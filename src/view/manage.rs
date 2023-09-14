mod imp {
    use std::cell::OnceCell;

    use adw::prelude::{GtkWindowExt, MessageDialogExt, MessageDialogExtManual, WidgetExt};
    use gtk::gio::Cancellable;
    use gtk::glib::subclass::InitializingObject;
    use gtk::glib::{clone, FromVariant, Sender};
    use gtk::prelude::{CastNone, StaticType};
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};

    use crate::component::viewport::Action;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/manage.ui")]
    pub struct Manage {
        pub sender: OnceCell<Sender<Action>>,
        #[template_child]
        pub app_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub show_all: TemplateChild<gtk::Switch>,
        #[template_child]
        pub view_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub filter_entry: TemplateChild<gtk::SearchEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Manage {
        const NAME: &'static str = "Manage";
        type Type = super::Manage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("back", None, |slf, _, _| {
                let _ = slf.imp().sender.get().unwrap().send(Action::Back);
            });
            klass.install_action("toggle_show_all", None, |slf, _, _| {
                let show_all = slf.imp().show_all.get().is_active();
                let view_label = slf.imp().view_label.get();

                if show_all {
                    view_label.set_label("All");
                } else {
                    view_label.set_label("Managed by <i>Shortcut</i>");
                }

                slf.load(show_all);
            });
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
                    clone!(@weak slf => move |decision| {
                        if decision.eq("delete") {
                            match std::fs::remove_file(path) {
                                Ok(()) => {
                                    slf.load(false);
                                }
                                Err(e) => eprintln!("Could not delete file because of error {}", e),
                            }
                        }
                    }),
                );
            });
            klass.install_action("edit", Some("(sss)"), |slf, _, input| {
                let (name, icon_path, exec_path) =
                    <(String, String, String)>::from_variant(input.unwrap()).unwrap();
                let _ = slf.imp().sender.get().unwrap().send(Action::QuickFlow(
                    Some(name),
                    Some(icon_path),
                    Some(exec_path),
                ));
            });
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Manage {}

    impl WidgetImpl for Manage {}

    impl BoxImpl for Manage {}
}

use adw::prelude::EditableExt;
use adw::prelude::WidgetExt;
use freedesktop_entry_parser::parse_entry;
use glib::Object;
use gtk::glib::clone;
use gtk::prelude::Cast;
use gtk::{
    glib::{self, Sender},
    subclass::prelude::ObjectSubclassIsExt,
};

use crate::component::viewport::Action;

glib::wrapper! {
    pub struct Manage(ObjectSubclass<imp::Manage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Manage {
    pub fn new(sender: Sender<Action>) -> Self {
        let slf = Object::builder::<Self>().build();
        slf.set_sensitive(false);
        let _ = slf.imp().sender.set(sender);
        slf.imp().view_label.set_label("Managed by <i>Shortcut</i>");
        slf.setup_filter();
        slf
    }

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
            }
        }
    }

    pub fn load(&self, all: bool) {
        let imp = self.imp();

        imp.show_all.set_active(all);

        while imp.app_list.first_child().is_some() {
            imp.app_list.remove(&imp.app_list.first_child().unwrap());
        }

        //Load all the item widgets in the internal list
        let paths = std::fs::read_dir(gtk::glib::user_data_dir().join("applications")).unwrap();
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

        self.filter(imp.app_list.get(), &imp.filter_entry.text())
    }
}
