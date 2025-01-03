use adw::{ glib, ApplicationWindow};
use adw::glib::subclass::InitializingObject;
use adw::subclass::prelude::*;
use gtk::{Button, CompositeTemplate, TemplateChild};
use adw::prelude::*;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/sunnyayyl/gui/window.ui")]
pub struct Window {
    #[template_child]
    pub button: TemplateChild<Button>,
}
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "AppWindow";
    type Type = super::Window;
    type ParentType = ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Connect to "clicked" signal of `button`
        self.button.connect_clicked(move |button| {
            // Set the label to "Hello World!" after the button has been clicked on
            button.set_label("Hello World!");
        });
    }
}
impl WidgetImpl for Window {}

impl ApplicationImpl for Window {}
impl GtkApplicationImpl for Window {}

impl AdwApplicationImpl for Window {}

impl ApplicationWindowImpl for Window {}

impl WindowImpl for Window {}

impl AdwApplicationWindowImpl for Window {}