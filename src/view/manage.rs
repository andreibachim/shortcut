mod imp {

    use adw::prelude::{GtkWindowExt, MessageDialogExt, MessageDialogExtManual, WidgetExt};
    use adw::subclass::prelude::NavigationPageImpl;
    use gtk::gio::Cancellable;
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::glib::FromVariant;
    use gtk::prelude::{CastNone, StaticType, ToVariant};
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/manage.ui")]
    pub struct Manage {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub app_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub list_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub filter_entry: TemplateChild<gtk::SearchEntry>,
    }

    #[gtk::template_callbacks]
    impl Manage {
        #[template_callback]
        fn load(&self) {
            self.obj().load(false);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Manage {
        const NAME: &'static str = "Manage";
        type Type = super::Manage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("delete", Some("(ss)"), |slf, _, param| {
                let (path, name) = <(String, String)>::from_variant(param.unwrap()).unwrap();
                let binding = slf.ancestor(gtk::Window::static_type());
                let window = binding.and_dynamic_cast_ref::<gtk::Window>().unwrap();

                let confirm_dialog = adw::MessageDialog::builder()
                    .heading("Delete")
                    .body(format!(
                        "Are you sure you want to delete the '{}' shortcut?",
                        name
                    ))
                    .default_response("cancel")
                    .close_response("cancel")
                    .modal(true)
                    .transient_for(window)
                    .build();

                confirm_dialog.add_responses(&[("cancel", "_Cancel"), ("delete", "_Delete")]);

                confirm_dialog
                    .set_response_appearance("delete", adw::ResponseAppearance::Destructive);

                confirm_dialog.present();
                confirm_dialog.choose(
                    Cancellable::NONE,
                    clone!(@weak slf, @weak window => move |decision| {
                        if decision.eq("delete") {
                            match std::fs::remove_file(path) {
                                Ok(()) => slf.load(false),
                                Err(e) => {
                                   let _ = window.activate_action("win.show_toast",
                                       Some(&"Could not delete the shortcut".to_variant()));
                                   eprintln!("Could not delete the shortcut: {:#?}", e);
                                },
                            }
                        }
                    }),
                )
            });

            klass.install_action("edit", Some("(sss)"), |slf, _, params| {
                let _ = slf.activate_action("win.load_quick_mode", params);
            });

            klass.install_action("reload_apps", None, |slf, _, _| {
                slf.load(false);
            });
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Manage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_filter();
            self.obj().load(false);
        }
    }

    impl WidgetImpl for Manage {}

    impl NavigationPageImpl for Manage {}
}

use adw::prelude::EditableExt;
use adw::prelude::WidgetExt;
use freedesktop_entry_parser::parse_entry;
use gtk::glib::clone;
use gtk::prelude::Cast;
use gtk::{
    glib::{self},
    subclass::prelude::ObjectSubclassIsExt,
};

glib::wrapper! {
    pub struct Manage(ObjectSubclass<imp::Manage>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Manage {
    fn setup_filter(&self) {
        let filter_entry = self.imp().filter_entry.get();
        let action = gtk::CallbackAction::new(|filter_entry, _| {
            filter_entry.grab_focus();
            true
        });
        let trigger = gtk::ShortcutTrigger::parse_string("<ctrl>F").unwrap();
        let shortcut = gtk::Shortcut::builder()
            .action(&action)
            .trigger(&trigger)
            .build();

        let shortcut_controller = gtk::ShortcutController::new();
        shortcut_controller.set_scope(gtk::ShortcutScope::Managed);
        shortcut_controller.add_shortcut(shortcut);
        filter_entry.add_controller(shortcut_controller);

        let app_list = self.imp().app_list.get();
        self.imp().filter_entry.connect_changed(
            clone!(@weak app_list, @weak self as slf => move |filter_entry| {
                let filter_criteria = filter_entry.text();
                slf.filter(app_list, &filter_criteria)
            }),
        );
    }

    fn filter(&self, app_list: gtk::ListBox, filter_criteria: &str) {
        let apps = app_list.observe_children();

        if apps.into_iter().count() == 0 {
            self.imp()
                .status_page
                .set_description(Some("You have not created any shortcuts"));

            self.imp().list_window.set_visible(false);
            self.imp().status_page.set_visible(true);
            return;
        }

        self.imp()
            .status_page
            .set_description(Some("Try a different search term."));

        let mut visible_apps: usize = 0;
        for app in app_list.observe_children().into_iter() {
            let app = app
                .unwrap()
                .dynamic_cast::<crate::component::Entry>()
                .unwrap();
            if !app
                .get_name()
                .to_lowercase()
                .contains(filter_criteria.to_lowercase().as_str())
            {
                app.set_visible(false);
            } else {
                app.set_visible(true);
                visible_apps += 1;
            }
        }
        self.imp().list_window.set_visible(visible_apps > 0);
        self.imp().status_page.set_visible(visible_apps == 0);
    }

    pub fn load(&self, all: bool) {
        let imp = self.imp();

        imp.filter_entry.set_text("");

        while imp.app_list.first_child().is_some() {
            imp.app_list.remove(&imp.app_list.first_child().unwrap());
        }

        //Load all the item widgets in the internal list
        let paths = std::fs::read_dir(format!(
            "/home/{}/.local/share/applications",
            std::env::var("USER").unwrap()
        ));

        if let Ok(paths) = paths {
            paths
                .into_iter()
                .filter_map(|entry_result| entry_result.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension.eq("desktop"))
                })
                .filter_map(|entry| {
                    parse_entry(entry.path())
                        .ok()
                        .map(|desktop| (entry, desktop))
                        .filter(|(_, desktop)| {
                            desktop
                                .section("Desktop Entry")
                                .attr("X-Shortcut-App")
                                .is_some()
                                || all
                        })
                })
                .for_each(|(dir_entry, desktop)| {
                    let section = desktop.section("Desktop Entry");
                    let entry = crate::component::Entry::new(
                        section.attr("Name"),
                        dir_entry.path().to_str(),
                        section.attr("Icon"),
                        section.attr("Exec"),
                    );
                    imp.app_list.append(&entry);
                });

            self.filter(imp.app_list.get(), &imp.filter_entry.text());
        }
    }
}
