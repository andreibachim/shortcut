mod component;
mod model;
mod view;

use gtk::{
    prelude::{ActionMapExtManual, ApplicationExt, ApplicationExtManual},
    traits::{BoxExt, GtkApplicationExt, GtkWindowExt},
};

const APP_ID: &str = "io.github.andreibachim.shortcut";

fn main() -> gtk::glib::ExitCode {
    gtk::gio::resources_register_include!("shortcut.gresource").expect("Could not load resources");

    let app = adw::Application::builder().application_id(APP_ID).build();
    app.set_flags(gtk::gio::ApplicationFlags::HANDLES_OPEN);
    app.connect_activate(build_window);
    set_up_actions(&app);
    app.run()
}

fn build_window(app: &adw::Application) {
    adw::ApplicationWindow::builder()
        .application(app)
        .default_width(600)
        .default_height(700)
        .content(&build_content())
        .icon_name(APP_ID)
        .title("Shortcut")
        .build()
        .present();
}

fn build_content() -> impl gtk::prelude::IsA<gtk::Widget> {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    set_up_headerbar(&content);

    let viewport = component::Viewport::new();
    content.append(&viewport);

    content
}

fn set_up_headerbar(content: &gtk::Box) {
    let headerbar = adw::HeaderBar::builder().build();

    let menu = gtk::gio::Menu::new();
    let shortcuts_item = gtk::gio::MenuItem::new(Some("Keyboard shortcuts"), Some("app.shortcuts"));
    menu.append_item(&shortcuts_item);
    let about_item = gtk::gio::MenuItem::new(Some("About Shortcut"), Some("app.about"));
    menu.append_item(&about_item);

    let menu_button = gtk::MenuButton::builder()
        .tooltip_text("Menu")
        .menu_model(&menu)
        .hexpand(true)
        .halign(gtk::Align::End)
        .icon_name("open-menu-symbolic")
        .build();

    let window_title = gtk::Label::builder()
        .use_markup(true)
        .label("<b>Shortcut</b>")
        .build();

    headerbar.set_title_widget(Some(
        &gtk::CenterBox::builder()
            .hexpand(true)
            .center_widget(&window_title)
            .end_widget(&menu_button)
            .build(),
    ));

    content.append(&headerbar);
}

fn set_up_actions(app: &adw::Application) {
    let quit = gtk::gio::ActionEntry::builder("quit")
        .activate(|app: &adw::Application, _, _| app.quit())
        .build();

    let about = gtk::gio::ActionEntry::builder("about")
        .activate(|app: &adw::Application, _, _| {
            let window = app.active_window().unwrap();
            adw::AboutWindow::builder()
                .application_name("Shortcut")
                .application_icon(APP_ID)
                .website("https://github.com/andreibachim/shortcut")
                .issue_url("https://github.com/andreibachim/shortcut/issues")
                .version("0.1.0")
                .developers(["Andrei Achim <andreiachim@duck.com>"])
                .license_type(gtk::License::Gpl30)
                .modal(true)
                .transient_for(&window)
                .build()
                .present();
        })
        .build();

    let keyboard_shortcuts = gtk::gio::ActionEntry::builder("shortcuts")
        .activate(|app: &adw::Application, _, _| {
            let shortcut_window: gtk::ShortcutsWindow = gtk::Builder::from_resource(
                "/io/github/andreibachim/shortcut/ui/keyboard-shortcuts.ui",
            )
            .object("keyboard_shortcuts")
            .unwrap();
            shortcut_window.set_transient_for(app.active_window().as_ref());
            shortcut_window.present();
        })
        .build();

    app.set_accels_for_action("app.shortcuts", &["<ctrl>question"]);
    app.set_accels_for_action("app.quit", &["<ctrl>Q"]);
    app.add_action_entries([quit, keyboard_shortcuts, about]);
}
