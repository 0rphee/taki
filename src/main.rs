use event_handlers::build_event_controller_key_handler;
use gtk4::{glib, prelude::*, Application, CssProvider, EventControllerKey, Label, ListBox};
use nucleo::{self};
use std::sync::{self, Arc, Mutex};

use widget_builders::{
    build_box_widg, build_entry_box_widg, build_scroll_widg, build_window_widg, APP_ID,
};

mod app;
mod event_handlers;
mod widget_builders;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let desktop_entries: &mut Arc<Vec<app::App>> = app::build_desktop_entries();
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

        #[cfg(debug_assertions)]
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
