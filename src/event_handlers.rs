use super::app::{exec_app, App};

use super::app;
use gtk4::{
    glib, prelude::*, ApplicationWindow, EventControllerKey, Label, ListBox, ListBoxRow, Text,
};
use std::sync::Arc;

pub fn return_pressed(
    desktop_entries_aux: &[App],
    result_listbox: &ListBox,
    entry_is_empty: bool,
) -> app::ExitApp {
    let selected_row_label: Label = match result_listbox.selected_row() {
        Some(row) => row.child().unwrap().dynamic_cast::<Label>().unwrap(),
        None => {
            if entry_is_empty {
                return app::ExitApp::DontExit;
            };

            match result_listbox.first_child() {
                None => return app::ExitApp::DontExit,
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

    #[cfg(debug_assertions)]
    println!(
        "Return Hit, selected: {}",
        selected_row_label.text().as_str()
    );

    exec_app(desktop_entries_aux, selected_row_label.text().as_str());
    return app::ExitApp::Exit;
}

pub fn build_event_controller_key_handler(
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
        #[cfg(debug_assertions)]
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
                    app::ExitApp::Exit => {
                        local_window_widg.close();
                    }
                    app::ExitApp::DontExit => return gtk4::glib::Propagation::Proceed, // if the result listbox widget is empty (no rows)
                }
            }
            gtk4::gdk::Key::Escape => {
                // Check if the text entry field has focus
                if entry_text_box_widg.has_focus() {
                    // Close the application if the text field has focus
                    local_window_widg.close();
                } else {
                    // Give focus to the text field if it doesn't have it
                    entry_text_box_widg.grab_focus_without_selecting();
                }
            }

            _ => {
                // Check if the text entry field has focus
                if !entry_text_box_widg.has_focus() {
                    // Give focus to the text field if it doesn't have it
                    entry_text_box_widg.grab_focus_without_selecting();
                }
            }
        };
        gtk4::glib::Propagation::Proceed
    };
}
