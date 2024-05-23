use super::application::{exec_app, App};
use gtk4::{prelude::*, Label, ListBox, ListBoxRow};

use super::application;

pub fn return_pressed(
    desktop_entries_aux: &[App],
    result_listbox: &ListBox,
    entry_is_empty: bool,
) -> application::ExitApp {
    let selected_row_label: Label = match result_listbox.selected_row() {
        Some(row) => row.child().unwrap().dynamic_cast::<Label>().unwrap(),
        None => {
            if entry_is_empty {
                return application::ExitApp::DontExit;
            };

            match result_listbox.first_child() {
                None => return application::ExitApp::DontExit,
                Some(fst_row) => fst_row
                    .dynamic_cast::<ListBoxRow>()
                    .unwrap()
                    .child()
                    .unwrap()
                    .dynamic_cast::<Label>()
                    .unwrap(),
            }
        } // dont exit if the app list is empty
    };

    println!(
        "Return Hit, selected: {}",
        selected_row_label.text().as_str()
    );
    exec_app(desktop_entries_aux, selected_row_label.text().as_str());
    return application::ExitApp::Exit;
}
