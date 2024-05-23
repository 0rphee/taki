pub use applications::common::App;
use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

impl super::AppL for applications::common::App {
    fn launch(&self) {
        println!("Exec path: {:?}", self.app_path_exe);

        let mut command = Command::new(OsStr::new("open"));

        command.args(["-a", self.app_path_exe.to_str().unwrap()]);

        command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null());

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

    fn scrubber(_config: &super::Config) -> Result<Vec<Self>, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        Result::Ok(
            applications::get_apps()
                .into_iter()
                .map(|mut app| {
                    // Para remover el .app del nombre
                    app.name.truncate(app.name.len() - 4);
                    app
                })
                .collect(),
        )
    }
}
