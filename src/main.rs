use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, CssProvider, EntryBuffer, Label, ListBox,
    Text,
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
    // let list_box_widg = gio::ListStore::new();
    let list_box_widg = ListBox::new();

    let entry_box_widg = Text::builder()
        .buffer(&EntryBuffer::new(Some("Yahoa")))
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(5)
        .margin_start(5)
        .build();
    // let result_list_widg = gio::ListStore::new();
    let result_list_widg = ListBox::new();

    list_box_widg.append(&entry_box_widg);
    list_box_widg.append(&result_list_widg);

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

    for entry in desktop_entries.iter() {
        label_widg_vec.push(Label::new(Some(&entry.name)));
        result_list_widg.append(label_widg_vec.last().unwrap());
        // result_list_widg.append(&cur_widg);
        // clone????
        injector.push(entry.name.as_str(), |dst| {
            dst[0] = entry.name.clone().into();
        });
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

    window_widg.present();
}
