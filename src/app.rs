use std::sync::Arc;

use serde::Deserialize;

#[cfg(target_os = "linux")]
pub type App = linux::DesktopEntry;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub type App = macos::App;

#[cfg(target_os = "macos")]
mod macos;

#[derive(Deserialize)]
pub struct Config {
    desktop_actions: bool,
    // max_entries: usize,
    terminal: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            desktop_actions: false,
            // max_entries: 5,
            terminal: None,
        }
    }
}

pub trait AppL {
    fn launch(&self);
    fn scrubber(config: &Config) -> Result<Vec<Self>, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

pub fn exec_app(app_list: &[App], name: &str) {
    for entry in app_list.iter() {
        if entry.name == name {
            entry.launch();
            break;
        }
    }
}

pub enum ExitApp {
    Exit,
    DontExit,
}

pub fn build_desktop_entries() -> &'static mut Arc<Vec<applications::common::App>> {
    let desktop_entry_config = Config::default();
    let desktop_entries: &mut Arc<Vec<App>> = Box::leak(Box::new(Arc::new({
        AppL::scrubber(&desktop_entry_config).unwrap()
    })));
    desktop_entries
}
