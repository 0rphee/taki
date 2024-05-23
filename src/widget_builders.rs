use super::app::{App, ExitApp};
use super::event_handlers::return_pressed;
use gtk4::{
    prelude::*, Application, ApplicationWindow, EntryBuffer, Label, ListBox, ScrolledWindow, Text,
};
use nucleo::{
    self,
    pattern::{self},
};
use std::sync::{Arc, Mutex};

pub const APP_ID: &str = "com.orphee.taki";
const APP_AND_WINDOW_NAME: &str = "taki";

pub fn build_box_widg() -> gtk4::Box {
    return gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
}

pub fn build_window_widg(app: &Application) -> ApplicationWindow {
    let window_widg = ApplicationWindow::builder()
        .application(app)
        .name(APP_AND_WINDOW_NAME)
        .title(APP_AND_WINDOW_NAME)
        .resizable(false)
        .default_height(250)
        .default_width(600)
        .build();
    window_widg
}

pub fn build_entry_box_widg(
    desktop_entries: Arc<Vec<App>>,
    local_result_list_widg: ListBox,
    window_widg_aux: ApplicationWindow,
    result_list_widg: ListBox,
    matcher: Arc<Mutex<nucleo::Nucleo<&str>>>,
) -> Text {
    let entry_text_box_widg = Text::builder()
        .buffer(&EntryBuffer::new(Option::<&str>::None))
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(5)
        .margin_start(5)
        .overwrite_mode(false)
        .placeholder_text("Write your app name here!")
        .build();
    entry_text_box_widg.connect_activate(move |local_text_widg| {
        match return_pressed(
            &desktop_entries,
            &local_result_list_widg,
            local_text_widg.text_length() == 0,
        ) {
            ExitApp::Exit => window_widg_aux.close(),
            ExitApp::DontExit => (),
        };
    });

    entry_text_box_widg.connect_changed({
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

    entry_text_box_widg
}

pub fn build_scroll_widg() -> ScrolledWindow {
    let scroll_widg = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .build();
    scroll_widg
}
