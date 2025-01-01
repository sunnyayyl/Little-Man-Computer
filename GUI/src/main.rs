use adw::glib::clone;
use adw::prelude::*;
use adw::{gio, glib, Application, HeaderBar};
use gtk::{Box, Button, Orientation};
fn main() -> glib::ExitCode {
    gio::resources_register_include!("compiled.gresource")
        .expect("Failed to register resources.");
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();
    app.connect_activate(build_ui);
    app.run()
}
fn build_ui(application: &Application) {
    let button = Button::builder()
        .label("Click me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let undo = Button::builder()
        .label("Undo")
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    button.connect_clicked(clone!(
        move |button| {
            button.set_label("Clicked!");
        }
    ));
    undo.connect_clicked(clone!(
        #[weak]
        button,
        move |_| {
            button.set_label("Click me!");
        }
    ));
    let body = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    body.append(&button);
    body.append(&undo);
    let content = Box::new(Orientation::Vertical, 0);
    content.append(&HeaderBar::new());
    content.append(&body);
    let window = adw::ApplicationWindow::builder()
        .application(application)
        .default_width(320)
        .default_height(200)
        .content(&content)
        .title("Hello, World!")
        .build();
    window.present();
}



