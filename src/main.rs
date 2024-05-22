use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, CssProvider, EntryBuffer, EventControllerKey,
    Label, ListBox, ScrolledWindow, Text,
};
use nucleo::{
    self,
    pattern::{self},
};
use std::sync::{self, Arc, Mutex};

mod application;

const APP_ID: &str = "com.orphee.taki";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let list_box_widg = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    list_box_widg.set_hexpand(true);
    list_box_widg.set_vexpand(true);
    let entry_box_widg = Text::builder()
        .buffer(&EntryBuffer::new(Some("Yahoa")))
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(5)
        .margin_start(5)
        .build();

    let result_list_widg = ListBox::builder().hexpand(true).vexpand(true).build();

    // #######################################################
    // TODO: MODIFICACIONES PARA EJECUTAR (ENTER)
    // let result_list_event_controller = gtk4::EventControllerKey::new();

    // // Connect key press event handler for ENTER key
    // result_list_event_controller.connect_key_pressed(
    //     move |_controller, keyval, _keycode, _modifier_type_state| {
    //         // Use pattern matching to handle the Option

    //         println!("Controller widget{}", _controller);
    //         if let gtk4::gdk::Key::Return = keyval {}

    //         gtk4::glib::Propagation::Stop
    //     },
    // );

    // result_list_widg.add_controller(result_list_event_controller);
    // #######################################################

    let scroll_widg = ScrolledWindow::builder()
        .child(&result_list_widg)
        .vexpand(true)
        .hexpand(true)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .build();

    list_box_widg.append(&entry_box_widg);
    list_box_widg.append(&scroll_widg);

    // MODIFICACIONES
    // let scroll_window = ListView::new();

    let window_widg = ApplicationWindow::builder()
        .application(app)
        .name("taki")
        .title("taki")
        .resizable(false)
        .default_height(250)
        .default_width(600)
        // .child(&list_box_widg.into())
        .child(&list_box_widg)
        .build();

    gtk4::style_context_add_provider_for_display(
        &WidgetExt::display(&window_widg),
        &CssProvider::new(),
        1,
    );

    let desktop_entry_config = application::Config::default();
    let desktop_entries: &mut Arc<Vec<application::App>> = Box::leak(Box::new(Arc::new(
        {
            let mut de: Vec<application::App> =
                application::AppL::scrubber(&desktop_entry_config).unwrap();

            #[cfg(target_os = "macos")]
            for app in de.iter_mut() {
                // Para remover el .app del nombre
                app.name.truncate(app.name.len() - 4);
            }
            de
        }, // .into_iter()
           // .map(|(de, _)| de)
           // .collect::<Vec<_>>()
    )));

    // matcher settings
    let mut matcher_config = nucleo::Config::DEFAULT;
    matcher_config.normalize = true;
    matcher_config.ignore_case = true;
    matcher_config.prefer_prefix = true;
    let matcher_notify_func = sync::Arc::new(|| {
        // println!("hello");
    });
    let matcher_thread_num = Some(2);

    let matcher: nucleo::Nucleo<&'static str> =
        nucleo::Nucleo::new(matcher_config, matcher_notify_func, matcher_thread_num, 1);

    let injector = matcher.injector();

    let mut label_widg_vec = Vec::new();

    let label_controller = EventControllerKey::new();

    label_controller.connect_key_pressed(
        move |_controller, keyval, _keycode, _modifier_type_state| {
            // Use pattern matching to handle the Option

            if let gtk4::gdk::Key::Return = keyval {
                println!("Controller widget{}", _controller.widget());
            }

            gtk4::glib::Propagation::Stop
        },
    );
    for entry in desktop_entries.iter() {
        let label_aux = Label::new(Some(&entry.name));
        // label_aux.add_controller(result_list_event_controller);
        label_aux.add_controller(label_controller.clone());
        label_widg_vec.push(label_aux);

        result_list_widg.append(label_widg_vec.last().unwrap());
        injector.push(entry.name.as_str(), |dst| {
            // clone????
            dst[0] = entry.name.clone().into();
        });
    }

    for i in desktop_entries.iter() {
        println!("Entry info{:?}", i);
    }

    for i in 0..injector.injected_items() {
        let it = injector.get(i).unwrap();
        println!("injected it {} {}", i, it.data);
    }

    let matcher = Arc::new(Mutex::new(matcher));

    let child = Label::new(Some("text"));
    result_list_widg.append(&child);

    entry_box_widg.connect_changed({
        move |new_entry_text| {
            let mut matcher = matcher.lock().unwrap();
            let new_entry_text = new_entry_text.text();
            let new_entry_str = new_entry_text.as_str();

            matcher.pattern.reparse(
                0,
                new_entry_str,
                pattern::CaseMatching::Ignore,
                pattern::Normalization::Never,
                false,
            );

            matcher.tick(10);

            let snapshot = matcher.snapshot();
            let matches = snapshot.matched_items(..);
            println!("total matches={}", matches.len());
            let mut ref_vec = Vec::new();

            for nucleo::Item { data, .. } in matches {
                let mut next = result_list_widg.first_child();
                let mut i = 0;

                while let Some(inner) = next {
                    let label = inner
                        .first_child()
                        .unwrap()
                        .dynamic_cast::<Label>()
                        .unwrap();

                    let name_str = label.text();
                    let name_str = name_str.as_str();

                    print!(
                        "checking i={} | name={:>22} data={:>20} ",
                        i, name_str, data
                    );
                    next = inner.next_sibling();
                    i += 1;
                    if *name_str == **data {
                        println!("matches!");
                        ref_vec.push(inner);
                    } else {
                        inner.set_visible(false);
                        println!();
                    }
                }
            }
            for widget in ref_vec {
                widget.set_visible(true);
            }
            println!("Change! current text is '{}'", new_entry_str);
        }
    });
    // MODIFICACIONES PARA ESC

    // #######################################################

    let window_widg_event_controller = EventControllerKey::new();
    let window_aux = window_widg.clone();
    // let desktop_aux = desktop_entries.clone();
    window_widg_event_controller.connect_key_pressed(
        move |_controller, keyval, _keycode, _modifier_type_state| {
            if keyval == gtk4::gdk::Key::Escape {
                // Check if the text entry field has focus
                if entry_box_widg.has_focus() {
                    // Close the application if the text field has focus
                    window_aux.close();
                } else {
                    // Give focus to the text field if it doesn't have it
                    entry_box_widg.grab_focus();
                }
            }
            // if keyval == gdk::Key::Tab {
            //     if entry_box_widg.has_focus() {
            //         result_list_widg.grab_focus();
            //         if let Some(first_child) = result_list_widg.first_child() {
            //             first_child.grab_focus();
            //         }
            //     } else if result_list_widg.has_focus() {
            //         if let Some(current) = result_list_widg.focus_child() {
            //             if let Some(next) = current.next_sibling() {
            //                 next.grab_focus();
            //             } else if let Some(first_child) = result_list_widg.first_child() {
            //                 first_child.grab_focus();
            //             }
            //         } else if let Some(first_child) = result_list_widg.first_child() {
            //             first_child.grab_focus();
            //         }
            //     }
            // }
            println!("{}", keyval);
            gtk4::glib::Propagation::Proceed
        },
    ); // #######################################################
    window_widg.add_controller(window_widg_event_controller);

    // test launch
    application::exec_app(&desktop_entries, "Visual Studio Code");

    window_widg.present();
}
