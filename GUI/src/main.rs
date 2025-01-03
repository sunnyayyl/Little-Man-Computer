use adw::{gio, glib, Application};
use adw::prelude::*;
use crate::window::Window;

mod window;


fn main() -> glib::ExitCode {
    gio::resources_register_include!("compiled.gresource")
        .expect("Failed to register resources.");
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();
    app.connect_activate(build_ui);
    app.run()
}
pub fn build_ui(app: &Application){
    let window=Window::new(app);
    window.present();
}
