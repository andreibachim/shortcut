mod imp {
    use std::cell::{OnceCell, RefCell};

    use adw::prelude::{
        GtkWindowExt, MessageDialogExt, MessageDialogExtManual, PreferencesGroupExt, WidgetExt,
    };
    use gtk::gio::Cancellable;
    use gtk::glib::subclass::InitializingObject;
    use gtk::glib::{clone, FromVariant, Sender};
    use gtk::prelude::{CastNone, StaticType};
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};

    use crate::component::viewport::Action;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/manage.ui")]
    pub struct Manage {
        pub sender: OnceCell<Sender<Action>>,
        #[template_child]
        pub app_list: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub show_all_switch: TemplateChild<gtk::Switch>,
        pub shortcuts: RefCell<Vec<gtk::Widget>>,
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
                let show_all = slf.imp().show_all_switch.is_active();
                let app_list = slf.imp().app_list.get();
                if show_all {
                    app_list.set_description(Some("All"));
                } else {
                    app_list.set_description(Some("Installed using Shortcut"));
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
                                    slf.load(slf.imp().show_all_switch.is_active());
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

use std::path::Path;
use std::path::PathBuf;

use adw::prelude::PreferencesGroupExt;
use adw::prelude::WidgetExt;
use freedesktop_entry_parser::parse_entry;
use glib::Object;
use gtk::prelude::ActionableExtManual;
use gtk::prelude::Cast;
use gtk::prelude::ToVariant;
use gtk::Builder;
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
        slf
    }

    pub fn set_icon(&self, image: &gtk::Image, icon_path: Option<&str>) {
        if let Some(icon_path) = icon_path {
            if PathBuf::from(icon_path).is_absolute() {
                if Path::new(icon_path).exists() {
                    image.set_from_file(Some(icon_path));
                } else {
                    self.set_placeholder_icon(image);
                }
            } else {
                let themed_icon = gtk::gio::ThemedIcon::from_names(&[icon_path]);
                image.set_gicon(Some(&themed_icon));
            }
        } else {
            self.set_placeholder_icon(image);
        };
    }

    pub fn set_placeholder_icon(&self, image: &gtk::Image) {
        let themed_icon = gtk::gio::ThemedIcon::from_names(&["application-x-executable"]);
        image.set_gicon(Some(&themed_icon));
    }

    pub fn load(&self, all: bool) {
        let imp = self.imp();

        imp.show_all_switch.set_active(all);

        let mut shortcuts = imp.shortcuts.borrow_mut();

        //Clear the current lists
        while !shortcuts.is_empty() {
            imp.app_list.remove(&shortcuts.remove(0));
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

                let builder =
                    Builder::from_resource("/io/github/andreibachim/shortcut/component/entry.ui");

                //Title
                let name = section.attr("Name").unwrap_or("");
                builder
                    .object::<gtk::Label>("title")
                    .unwrap()
                    .set_label(name);
                //Subtitle
                let subtitle = builder.object::<gtk::Label>("subtitle").unwrap();
                let subtitle_text = dir_entry.path().display().to_string();
                subtitle.set_label(&format!("<small>{}</small>", subtitle_text.trim()));
                subtitle.set_tooltip_text(Some(subtitle_text.trim()));

                let icon_path = section.attr("Icon");

                self.set_icon(
                    &builder.object::<gtk::Image>("app_icon").unwrap(),
                    icon_path,
                );

                //Delete button
                builder
                    .object::<gtk::Button>("delete_button")
                    .unwrap()
                    .set_action_target(Some(&(subtitle_text.trim(), name).to_variant()));

                //Edit button
                builder
                    .object::<gtk::Button>("edit_button")
                    .unwrap()
                    .set_action_target(Some(
                        &(
                            name,
                            section.attr("Icon").unwrap_or(""),
                            section.attr("Exec").unwrap_or(""),
                        )
                            .to_variant(),
                    ));

                //Get the entry and add it the list
                let entry: adw::PreferencesRow = builder.object("entry").unwrap();
                shortcuts.push(entry.dynamic_cast::<gtk::Widget>().unwrap());
            });

        //Push all the widget from the internal list into the view list
        shortcuts.iter().for_each(|widget| {
            imp.app_list.add(widget);
        });
    }
}
