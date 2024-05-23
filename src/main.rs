use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, CssProvider, EventControllerKey, Label,
    ListBox, Text,
};
use nucleo::{self};
use std::sync::{self, Arc, Mutex};

use application::App;
use widget_builders::{
    build_box_widg, build_entry_box_widg, build_scroll_widg, build_window_widg, APP_ID,
};

use event_handlers::return_pressed;

mod application;
mod event_handlers;
mod widget_builders;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let desktop_entries: &mut Arc<Vec<application::App>> = build_desktop_entries();
    let box_widg = build_box_widg();
    let window_widg = build_window_widg(app);
    window_widg.set_child(Some(&box_widg));

    let result_list_widg = ListBox::builder().hexpand(true).vexpand(true).build();

    // ###################### MATCHER BEGIN ######################
    let matcher: nucleo::Nucleo<&'static str> = build_matcher();

    let injector: nucleo::Injector<&str> = matcher.injector();

    for entry in desktop_entries.iter() {
        let curr_label = Label::new(Some(&entry.name));
        result_list_widg.append(&curr_label);
        injector.push(entry.name.as_str(), |dst| {
            // clone????
            dst[0] = entry.name.clone().into();
        });

        println!("Entry info{:?}", &entry);
    }

    // for i in 0..injector.injected_items() {
    //     let it = injector.get(i).unwrap();
    //     println!("injected it {} {}", i, it.data);
    // }

    let matcher = Arc::new(Mutex::new(matcher));
    // ###################### MATCHER END ######################

    let entry_text_box_widg = build_entry_box_widg(
        desktop_entries.clone(),
        result_list_widg.clone(),
        window_widg.clone(),
        result_list_widg.clone(),
        matcher,
    );

    let scroll_widg = build_scroll_widg();
    scroll_widg.set_child(Some(&result_list_widg));

    box_widg.append(&entry_text_box_widg);
    box_widg.append(&scroll_widg);

    gtk4::style_context_add_provider_for_display(
        &WidgetExt::display(&window_widg),
        &CssProvider::new(),
        1,
    );

    // #######################################################

    let window_widg_event_controller = EventControllerKey::new();
    window_widg_event_controller.connect_key_pressed(build_event_controller_key_handler(
        entry_text_box_widg,
        desktop_entries.clone(),
    ));
    window_widg.add_controller(window_widg_event_controller);

    window_widg.present();
}

fn build_matcher() -> nucleo::Nucleo<&'static str> {
    // ###################### MATCHER BEGIN ######################
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
    return matcher;
}

fn build_desktop_entries() -> &'static mut Arc<Vec<applications::common::App>> {
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
    desktop_entries
}

fn build_event_controller_key_handler(
    entry_text_box_widg: Text,
    desktop_entries: Arc<Vec<App>>,
) -> impl for<'a> Fn(
    &'a EventControllerKey,
    gtk4::gdk::Key,
    u32,
    gtk4::gdk::ModifierType,
) -> glib::Propagation
       + 'static {
    return move |_controller, keyval, _keycode, _modifier_type_state| {
        println!("Key Pressed: {}", keyval);

        let local_window_widg: ApplicationWindow = _controller.widget().dynamic_cast().unwrap();
        let local_text_widg: Text = local_window_widg
            .first_child()
            .and_then(|this| this.first_child()?.dynamic_cast::<Text>().ok())
            .unwrap();
        let get_list_box = || -> ListBox {
            local_window_widg // GtkApplicationWindow
                .first_child() // GtkBox
                .and_then(|this| {
                    this.last_child()? // GtkScrolledWindow
                        .first_child()? // GtkViewport
                        .first_child()? // GtkListBox
                        .dynamic_cast() // Cast Window -> ListBox
                        .ok() // Convert Result<(), ListBox> to Ok<ListBox>
                })
                .unwrap_or_else(|| unreachable!())
        };

        match keyval {
            gtk4::gdk::Key::Escape => {
                // Check if the text entry field has focus
                if entry_text_box_widg.has_focus() {
                    // Close the application if the text field has focus
                    local_window_widg.close();
                } else {
                    // Give focus to the text field if it doesn't have it
                    entry_text_box_widg.grab_focus();
                }
            }
            gtk4::gdk::Key::Tab => {
                let result_list_widg = get_list_box();
                if entry_text_box_widg.has_focus() {
                    result_list_widg.grab_focus();
                    if let Some(first_child) = result_list_widg.first_child() {
                        first_child.grab_focus();
                    }
                } else if result_list_widg.has_focus() {
                    if let Some(current) = result_list_widg.focus_child() {
                        if let Some(next) = current.next_sibling() {
                            next.grab_focus();
                        } else if let Some(first_child) = result_list_widg.first_child() {
                            first_child.grab_focus();
                        }
                    } else if let Some(first_child) = result_list_widg.first_child() {
                        first_child.grab_focus();
                    }
                }
            }
            gtk4::gdk::Key::Return => {
                let result_listbox = get_list_box();
                match return_pressed(
                    &desktop_entries,
                    &result_listbox,
                    local_text_widg.text_length() == 0,
                ) {
                    application::ExitApp::Exit => {
                        local_window_widg.close();
                    }
                    application::ExitApp::DontExit => return gtk4::glib::Propagation::Proceed, // if the result listbox widget is empty (no rows)
                }
            }

            _ => (),
        };
        gtk4::glib::Propagation::Proceed
    };
}
