// use applications::get_apps;
use libc;
use serde::Deserialize;
use std::process::{Command, Stdio};
use std::sync::{self, Arc, Mutex};

#[cfg(target_os = "linux")]
pub type App = linux::DesktopEntry;
// #[cfg(target_os = "linux")]
// pub use linux::DesktopEntry;
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

pub fn exec_app(app_list: &Arc<Vec<App>>, name: String) {
    for entry in app_list.iter() {
        if entry.name == name {
            println!(
                "Path: {
                }",
                entry.exec
            );
            entry.exec.clone().push_str(" &");
            let mut command = Command::new(&entry.exec);

            command
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null());

            // Según entendí esto sirve para desvincular el proceso padre del proceso hijo
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.before_exec(|| {
                    unsafe { libc::setsid() };
                    Ok(())
                });
            }

            match command.spawn() {
                Ok(mut child) => {
                    println!("Aplicación lanzada exitosamente con PID: {}", child.id());
                    let _ = child.wait();
                }
                Err(e) => {
                    eprintln!("Error al lanzar la aplicación: {}", e)
                }
            }
        }
    }
}
