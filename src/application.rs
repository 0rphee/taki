use serde::Deserialize;

#[cfg(target_os = "linux")]
pub type App = linux::DesktopEntry;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub type App = macos::App;
// #[cfg(target_os = "macos")]
// pub use macos::App;
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
        }
    }
}
