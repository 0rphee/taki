use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, CssProvider, EntryBuffer, Label, ListBox,
    Text,
};
use nucleo::{
    self,
    pattern::{self},
};
use std::sync::{self, Arc, Mutex};

mod application {
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::env;
    use std::{ffi::OsStr, fs, path::PathBuf};

    pub struct DesktopEntry {
        pub exec: String,
        pub path: Option<PathBuf>,
        pub name: String,
        pub keywords: Vec<String>,
        pub desc: Option<String>,
        pub icon: String,
        pub term: bool,
        pub offset: i64,
    }

    const FIELD_CODE_LIST: &[&str] = &[
        "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m",
    ];

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
    impl DesktopEntry {
        fn from_dir_entry(entry: &fs::DirEntry, config: &Config) -> Vec<Self> {
            if entry.path().extension() == Some(OsStr::new("desktop")) {
                let content = match fs::read_to_string(entry.path()) {
                    Ok(content) => content,
                    Err(_) => return Vec::new(),
                };

                let lines = content.lines().collect::<Vec<_>>();

                let sections = lines
                    .split_inclusive(|line| line.starts_with('['))
                    .collect::<Vec<_>>();

                let mut line = None;
                let mut new_sections = Vec::new();

                for (i, section) in sections.iter().enumerate() {
                    if let Some(line) = line {
                        let mut section = section.to_vec();
                        section.insert(0, line);

                        // Only pop the last redundant entry if it isn't the last item
                        if i < sections.len() - 1 {
                            section.pop();
                        }
                        new_sections.push(section);
                    }
                    line = Some(section.last().unwrap_or(&""));
                }

                let mut ret = Vec::new();

                let entry = match new_sections.iter().find_map(|section| {
                    if section[0].starts_with("[Desktop Entry]") {
                        let mut map = HashMap::new();

                        for line in section.iter().skip(1) {
                            if let Some((key, val)) = line.split_once('=') {
                                map.insert(key, val);
                            }
                        }

                        if map.get("Type")? == &"Application"
                            && match map.get("NoDisplay") {
                                Some(no_display) => !no_display.parse::<bool>().unwrap_or(true),
                                None => true,
                            }
                        {
                            Some(DesktopEntry {
                                exec: {
                                    let mut exec = map.get("Exec")?.to_string();

                                    for field_code in FIELD_CODE_LIST {
                                        exec = exec.replace(field_code, "");
                                    }
                                    exec
                                },
                                path: map.get("Path").map(PathBuf::from),
                                name: map.get("Name")?.to_string(),
                                keywords: map
                                    .get("Keywords")
                                    .map(|keywords| {
                                        keywords
                                            .split(';')
                                            .map(|s| s.to_owned())
                                            .collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default(),
                                desc: None,
                                icon: map
                                    .get("Icon")
                                    .unwrap_or(&"application-x-executable")
                                    .to_string(),
                                term: map
                                    .get("Terminal")
                                    .map(|val| val.to_lowercase() == "true")
                                    .unwrap_or(false),
                                offset: 0,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }) {
                    Some(entry) => entry,
                    None => return Vec::new(),
                };

                if config.desktop_actions {
                    for (i, section) in new_sections.iter().enumerate() {
                        let mut map = HashMap::new();

                        for line in section.iter().skip(1) {
                            if let Some((key, val)) = line.split_once('=') {
                                map.insert(key, val);
                            }
                        }

                        if section[0].starts_with("[Desktop Action") {
                            ret.push(DesktopEntry {
                                exec: match map.get("Exec") {
                                    Some(exec) => {
                                        let mut exec = exec.to_string();

                                        for field_code in FIELD_CODE_LIST {
                                            exec = exec.replace(field_code, "");
                                        }
                                        exec
                                    }
                                    None => continue,
                                },
                                path: entry.path.clone(),
                                name: match map.get("Name") {
                                    Some(name) => name.to_string(),
                                    None => continue,
                                },
                                keywords: map
                                    .get("Keywords")
                                    .map(|keywords| {
                                        keywords
                                            .split(';')
                                            .map(|s| s.to_owned())
                                            .collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default(),
                                desc: Some(entry.name.clone()),
                                icon: entry.icon.clone(),
                                term: map
                                    .get("Terminal")
                                    .map(|val| val.to_lowercase() == "true")
                                    .unwrap_or(false),
                                offset: i as i64,
                            })
                        }
                    }
                }

                ret.push(entry);
                ret
            } else {
                Vec::new()
            }
        }
    }

    pub fn scrubber(
        config: &Config,
    ) -> Result<Vec<(DesktopEntry, u64)>, Box<dyn std::error::Error>> {
        // Create iterator over all the files in the XDG_DATA_DIRS
        // XDG compliancy is cool
        let user_path = match env::var("XDG_DATA_HOME") {
            Ok(data_home) => {
                format!("{}/applications/", data_home)
            }
            Err(_) => {
                format!(
                    "{}/.local/share/applications/",
                    env::var("HOME").expect("Unable to determine home directory!")
                )
            }
        };

        let mut entries: HashMap<String, DesktopEntry> = match env::var("XDG_DATA_DIRS") {
            Ok(data_dirs) => {
                // The vec for all the DirEntry objects
                let mut paths = Vec::new();
                // Parse the XDG_DATA_DIRS variable and list files of all the paths
                for dir in data_dirs.split(':') {
                    match fs::read_dir(format!("{}/applications/", dir)) {
                        Ok(dir) => {
                            paths.extend(dir);
                        }
                        Err(why) => {
                            eprintln!("Error reading directory {}: {}", dir, why);
                        }
                    }
                }
                // Make sure the list of paths isn't empty
                if paths.is_empty() {
                    return Err("No valid desktop file dirs found!".into());
                }

                // Return it
                paths
            }
            Err(_) => fs::read_dir("/usr/share/applications")?.collect(),
        }
        .into_iter()
        .filter_map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_why) => return None,
            };
            let entries = DesktopEntry::from_dir_entry(&entry, config);
            Some(
                entries
                    .into_iter()
                    .map(|entry| (format!("{}{}", entry.name, entry.icon), entry)),
            )
        })
        .flatten()
        .collect();

        // Go through user directory desktop files for overrides
        match fs::read_dir(&user_path) {
            Ok(dir_entries) => entries.extend(
                dir_entries
                    .into_iter()
                    .filter_map(|entry| {
                        let entry = match entry {
                            Ok(entry) => entry,
                            Err(_why) => return None,
                        };
                        let entries = DesktopEntry::from_dir_entry(&entry, config);
                        Some(
                            entries
                                .into_iter()
                                .map(|entry| (format!("{}{}", entry.name, entry.icon), entry)),
                        )
                    })
                    .flatten(),
            ),
            Err(why) => eprintln!("Error reading directory {}: {}", user_path, why),
        }

        Ok(entries
            .into_iter()
            .enumerate()
            .map(|(i, (_, entry))| (entry, i as u64))
            .collect())
    }
}

const APP_ID: &str = "com.orphee.toki";

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
    let desktop_entries = Box::leak(Box::new(Arc::new(
        application::scrubber(&desktop_entry_config)
            .unwrap()
            .into_iter()
            .map(|(de, _)| de)
            .collect::<Vec<_>>(),
    )));

    // matcher settings
    let mut matcher_config = nucleo::Config::DEFAULT;
    matcher_config.normalize = true;
    matcher_config.ignore_case = true;
    matcher_config.prefer_prefix = true;
    let matcher_notify_func = sync::Arc::new(|| {
        println!("hello");
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
