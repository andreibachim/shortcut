mod component;
mod model;
mod view;

use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::{BoxExt, GtkWindowExt},
};

const APP_ID: &str = "com.github.andreibachim.shortcut";

fn main() -> gtk::glib::ExitCode {
    gtk::gio::resources_register_include!("shortcut.gresource").expect("Could not load resources");

    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_window);
    app.run()
}

fn build_window(app: &adw::Application) {
    adw::ApplicationWindow::builder()
        .application(app)
        .default_width(600)
        .default_height(700)
        .content(&build_content())
        .title("Shortcut")
        .build()
        .present();
}

fn build_content() -> impl gtk::prelude::IsA<gtk::Widget> {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let headerbar = adw::HeaderBar::builder().build();
    content.append(&headerbar);

    let viewport = component::Viewport::new();
    content.append(&viewport);

    content
}
