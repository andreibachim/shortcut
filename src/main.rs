mod component;
mod function;
mod model;
mod view;

use component::Menu;
use gtk::{
    glib::{clone, VariantTy},
    prelude::{
        ActionMapExtManual, ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt,
        GtkWindowExt, SettingsExt, SettingsExtManual, StaticType,
    },
};

use crate::glib::variant::FromVariant;
use crate::glib::variant::StaticVariantType;
use adw::prelude::ComboRowExt;
use adw::{prelude::AdwApplicationWindowExt, prelude::AdwDialogExt};
use gtk::glib;
use view::{Manage, QuickMode};

const APP_ID: &str = "io.github.andreibachim.shortcut";

fn main() -> gtk::glib::ExitCode {
    gtk::gio::resources_register_include!("shortcut.gresource").expect("Could not load resources");

    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_window);
    setup_actions(&app);
    app.run()
}

fn build_window(app: &adw::Application) {
    let settings = gtk::gio::Settings::new(APP_ID);
    set_color_scheme(settings.uint("color-scheme"));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(1000)
        .default_height(700)
        .icon_name(APP_ID)
        .title("Shortcut")
        .build();

    window.set_content(Some(&build_content(&window)));
    setup_toasts_action(&window);

    window.present();
}

fn build_content(window: &adw::ApplicationWindow) -> impl gtk::prelude::IsA<gtk::Widget> {
    Manage::static_type();
    QuickMode::static_type();
    Menu::static_type();

    let nav_view: adw::NavigationView =
        gtk::Builder::from_resource("/io/github/andreibachim/shortcut/component/nav_view.ui")
            .object("nav_view")
            .unwrap();

    setup_nav_actions(window, &nav_view);
    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&nav_view));
    toast_overlay
}

fn setup_nav_actions(window: &adw::ApplicationWindow, nav_view: &adw::NavigationView) {
    let load_quick_mode = gtk::gio::ActionEntry::builder("load_quick_mode")
        .parameter_type(Some(&<(String, String, String)>::static_variant_type()))
        .activate(clone!(
            #[weak]
            nav_view,
            move |_, _, params| {
                let (name, icon_path, exec_path) =
                    <(String, String, String)>::from_variant(params.unwrap()).unwrap();
                let quick_mode_page = nav_view
                    .find_page("quick_mode")
                    .unwrap()
                    .dynamic_cast::<crate::view::QuickMode>()
                    .unwrap();
                quick_mode_page.clear_data();
                quick_mode_page.edit_details(Some(name), Some(icon_path), Some(exec_path));
                nav_view.push_by_tag("quick_mode");
            }
        ))
        .build();

    window.add_action_entries([load_quick_mode]);
}

fn setup_toasts_action(window: &adw::ApplicationWindow) {
    let show_toast = gtk::gio::ActionEntry::builder("show_toast")
        .parameter_type(Some(VariantTy::STRING))
        .activate(|window: &adw::ApplicationWindow, _, message| {
            let message = String::from_variant(message.unwrap()).unwrap();
            let toast_overlay: adw::ToastOverlay = window
                .content()
                .unwrap()
                .dynamic_cast::<adw::ToastOverlay>()
                .unwrap();
            toast_overlay.add_toast(adw::Toast::new(&message));
        })
        .build();

    window.add_action_entries([show_toast]);
}

fn setup_actions(app: &adw::Application) {
    let quit = gtk::gio::ActionEntry::builder("quit")
        .activate(|app: &adw::Application, _, _| app.quit())
        .build();

    let preferences = gtk::gio::ActionEntry::builder("preferences")
        .activate(|app: &adw::Application, _, _| {
            let settings = gtk::gio::Settings::new(APP_ID);

            let preferences_builder =
                gtk::Builder::from_resource("/io/github/andreibachim/shortcut/preferences.ui");

            let create_enable_validation: gtk::Switch = preferences_builder
                .object("create_enable_validation")
                .unwrap();

            settings
                .bind(
                    "create-enable-validation",
                    &create_enable_validation,
                    "active",
                )
                .build();

            let color_scheme: adw::ComboRow = preferences_builder.object("color_scheme").unwrap();

            settings
                .bind("color-scheme", &color_scheme, "selected")
                .build();

            color_scheme.connect_selected_notify(move |a| {
                let selected = a.selected();
                set_color_scheme(selected);
            });

            let preferences_dialog: adw::PreferencesDialog =
                preferences_builder.object("preferences").unwrap();
            preferences_dialog.present(app.active_window().as_ref());
        })
        .build();

    let about = gtk::gio::ActionEntry::builder("about")
        .activate(|app: &adw::Application, _, _| {
            let window = app.active_window().unwrap();
            adw::AboutDialog::builder()
                .application_name("Shortcut")
                .application_icon(APP_ID)
                .website("https://github.com/andreibachim/shortcut")
                .issue_url("https://github.com/andreibachim/shortcut/issues")
                .version(env!("CARGO_PKG_VERSION"))
                .developers(["Andrei Achim <andreiachim@duck.com>"])
                .license_type(gtk::License::Gpl30)
                .copyright("© 2025 Andrei Achim")
                .build()
                .present(Some(&window));
        })
        .build();

    let keyboard_shortcuts = gtk::gio::ActionEntry::builder("shortcuts")
        .activate(|app: &adw::Application, _, _| {
            let shortcut_window: gtk::ShortcutsWindow = gtk::Builder::from_resource(
                "/io/github/andreibachim/shortcut/keyboard_shortcuts.ui",
            )
            .object("keyboard_shortcuts")
            .unwrap();
            shortcut_window.set_transient_for(app.active_window().as_ref());
            shortcut_window.present();
        })
        .build();

    app.set_accels_for_action("app.preferences", &["<ctrl>comma"]);
    app.set_accels_for_action("app.shortcuts", &["<ctrl>question"]);
    app.set_accels_for_action("app.quit", &["<ctrl>Q"]);
    app.add_action_entries([quit, preferences, keyboard_shortcuts, about]);
}

fn set_color_scheme(scheme: u32) {
    match scheme {
        1 => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark),
        2 => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceLight),
        _ => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default),
    }
}
