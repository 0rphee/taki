use gtk4::{
    glib, prelude::*, Application, ApplicationWindow, CssProvider, EntryBuffer, Label, ListBox,
    Text,
};

const APP_ID: &str = "com.orphee.toki";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let list_box = ListBox::new();

    let entry_buf = EntryBuffer::new(Some("Yahoa"));
    let text_static = Text::builder()
        .buffer(&entry_buf)
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(5)
        .margin_start(5)
        .build();

    let text_box = Text::builder().buffer(&entry_buf).margin_top(40).build();

    list_box.prepend(&text_box);
    list_box.prepend(&text_static);

    let window = ApplicationWindow::builder()
        .application(app)
        .name("taki")
        .title("taki")
        .resizable(false)
        .default_height(250)
        .default_width(600)
        .child(&list_box)
        .build();

    gtk4::style_context_add_provider_for_display(
        &WidgetExt::display(&window),
        &CssProvider::new(),
        1,
    );

    window.present();
}
