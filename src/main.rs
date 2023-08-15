mod component;
mod model;
mod view;

use gtk::{
    prelude::{ActionMapExtManual, ApplicationExt, ApplicationExtManual},
    traits::{BoxExt, GtkApplicationExt, GtkWindowExt},
};

const APP_ID: &str = "com.github.andreibachim.shortcut";

fn main() -> gtk::glib::ExitCode {
    gtk::gio::resources_register_include!("shortcut.gresource").expect("Could not load resources");

    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_window);

    let about = gtk::gio::ActionEntry::builder("about")
        .activate(|app: &adw::Application, _, _| {
            let window = app.active_window().unwrap();
            adw::AboutWindow::builder()
                .application_name("Shortcut")
                .application_icon(APP_ID)
                .website("https://github.com/andreibachim/shortcut")
                .issue_url("https://github.com/andreibachim/shortcut/issues")
                .version("0.1.0")
                .license_type(gtk::License::Gpl30)
                .modal(true)
                .transient_for(&window)
                .build()
                .present()
        })
        .build();

    app.add_action_entries([about]);
    app.set_accels_for_action("app.about", &["<Control>q"]);
    app.run()
}

fn build_window(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(600)
        .default_height(700)
        .content(&build_content())
        .icon_name(APP_ID)
        .title("Shortcut")
        .build();

    gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap())
        .add_resource_path("/com/github/andreibachim/shortcut/icons/");

    window.present();
}

fn build_content() -> impl gtk::prelude::IsA<gtk::Widget> {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let headerbar = adw::HeaderBar::builder().build();
    content.append(&headerbar);

    let menu = gtk::gio::Menu::new();
    let about_item = gtk::gio::MenuItem::new(Some("About Shortcut"), Some("app.about"));
    menu.append_item(&about_item);

    let menu_button = gtk::MenuButton::builder()
        .tooltip_text("Menu")
        .menu_model(&menu)
        .hexpand(true)
        .halign(gtk::Align::End)
        .icon_name("open-menu-symbolic")
        .build();

    headerbar.set_title_widget(Some(&menu_button));

    let viewport = component::Viewport::new();
    content.append(&viewport);

    content
}
