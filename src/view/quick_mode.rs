mod imp {
    use std::{cell::RefCell, fs::File};

    use std::io::Write;
    use std::path::{Path, PathBuf};

    use adw::prelude::BinExt;
    use adw::prelude::EntryRowExt;
    use adw::subclass::prelude::NavigationPageImpl;

    use gtk::glib::subclass::InitializingObject;
    use gtk::glib::{self, Properties};
    use gtk::glib::{clone, closure};
    use gtk::prelude::{
        Cast, CastNone, FileExt, GObjectPropertyExpressionExt, ObjectExt, StaticType, ToVariant,
    };
    use gtk::prelude::{EditableExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtk::{ClosureExpression, CompositeTemplate};

    use crate::model::Desktop;

    #[derive(Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::QuickMode)]
    #[template(resource = "/io/github/andreibachim/shortcut/quick_mode.ui")]
    pub struct QuickMode {
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "exec", get, set, type = String, member = exec)]
        #[property(name = "icon", get, set, type = String, member = icon)]
        pub data: RefCell<Desktop>,

        #[property(get, set)]
        pub enable_validation: RefCell<bool>,

        pub old_name: RefCell<String>,

        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_input: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub name_preview: TemplateChild<gtk::Label>,
        #[template_child]
        pub exec_input: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub icon_input: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub icon_preview: TemplateChild<adw::Bin>,
    }

    #[gtk::template_callbacks]
    impl QuickMode {
        #[template_callback]
        fn save(&self) {
            let data = self.data.borrow();
            let old_name_binding = self.old_name.borrow();
            let old_name = old_name_binding.as_ref();

            if !data.name.eq(old_name) {
                let _ = std::fs::remove_file(self.get_file_path_from_name(old_name));
            }

            let target_dir = PathBuf::from(format!(
                "/home/{}/.local/share/applications",
                std::env::var("USER").unwrap()
            ));

            if !target_dir.exists() {
                let _ = std::fs::create_dir_all(&target_dir);
            }

            let file_path = target_dir.join(format!(
                "{}.desktop",
                data.name.replace(' ', "-").to_lowercase()
            ));

            let mut file = File::create(file_path).expect("Could not create a new file");
            match file.write_all(
                data.get_output()
                    .expect("Could not serialize desktop file for writing")
                    .as_bytes(),
            ) {
                Ok(()) => {
                    let _ = self
                        .obj()
                        .activate_action("navigation.pop", Some(&"manage".to_variant()));
                }
                Err(e) => {
                    eprintln!(
                        "Could not save file because of the following error: \n {:#?}",
                        e
                    );
                }
            }
        }

        fn get_file_path_from_name(&self, name: &str) -> PathBuf {
            PathBuf::from(format!(
                "/home/{}/.local/share/applications/{}.desktop",
                std::env::var("USER").unwrap(),
                name.replace(' ', "-").to_lowercase()
            ))
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QuickMode {
        const NAME: &'static str = "QuickMode";
        type Type = super::QuickMode;
        type ParentType = adw::NavigationPage;

        fn new() -> Self {
            Self {
                data: RefCell::new(Desktop::new()),
                ..Default::default()
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("pick_exec", None, move |quick_mode, _, _| {
                let imp = quick_mode.imp();

                let filters_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
                let executable_filter = gtk::FileFilter::new();
                executable_filter.set_name(Some("Executable files"));
                executable_filter.add_mime_type("application/x-executable");
                filters_store.append(&executable_filter);

                let all_filter = gtk::FileFilter::new();
                all_filter.add_pattern("*");
                all_filter.set_name(Some("All files"));
                filters_store.append(&all_filter);

                let main_window: adw::ApplicationWindow = quick_mode
                    .ancestor(adw::ApplicationWindow::static_type())
                    .unwrap()
                    .dynamic_cast()
                    .unwrap();

                let dialog = gtk::FileDialog::builder()
                    .filters(&filters_store)
                    .modal(true)
                    .title("Select Executable File")
                    .build();

                dialog.open(
                    Some(&main_window),
                    None::<&gtk::gio::Cancellable>,
                    clone!(
                        #[weak]
                        imp,
                        move |file| {
                            if let Ok(file) = file {
                                imp.exec_input.set_text(
                                    file.path()
                                        .expect("Invalid file path")
                                        .to_str()
                                        .expect("Path is not UTF-8 compliant"),
                                );
                                imp.exec_input.emit_by_name::<()>("apply", &[]);
                            }
                        }
                    ),
                );
            });
            klass.install_action("pick_icon", None, move |quick_mode, _, _| {
                let imp = quick_mode.imp();

                let filters_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
                let filter = gtk::FileFilter::new();
                filter.set_name(Some("Image files"));
                filter.add_mime_type("image/svg+xml");
                filter.add_mime_type("image/png");
                filters_store.append(&filter);

                let file_dialog = gtk::FileDialog::builder()
                    .filters(&filters_store)
                    .title("Select Icon File")
                    .modal(true)
                    .build();

                let main_window: adw::ApplicationWindow = quick_mode
                    .ancestor(adw::ApplicationWindow::static_type())
                    .unwrap()
                    .dynamic_cast()
                    .unwrap();

                file_dialog.open(
                    Some(&main_window),
                    None::<&gtk::gio::Cancellable>,
                    clone!(
                        #[weak]
                        imp,
                        move |file| {
                            if let Ok(file) = file {
                                imp.icon_input.set_text(
                                    file.path()
                                        .expect("Could not extract path from file")
                                        .to_str()
                                        .expect("Path is not UTF-8 compliant"),
                                );
                                imp.icon_input.emit_by_name::<()>("apply", &[]);
                            }
                        }
                    ),
                );
            });
        }
        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for QuickMode {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().init();
            bind_name_preview(self);
            setup_form_validation(self);
            self.icon_input
                .settings()
                .set_gtk_entry_select_on_focus(false);
            self.exec_input
                .settings()
                .set_gtk_entry_select_on_focus(false);
        }
    }

    fn bind_name_preview(slf: &QuickMode) {
        slf.name_input
            .bind_property("text", &slf.name_preview.get(), "label")
            .sync_create()
            .transform_to(|_, value: &str| -> Option<&str> {
                match value.is_empty() {
                    true => Some("Preview"),
                    false => Some(value),
                }
            })
            .build();
        slf.name_input
            .bind_property("text", &slf.name_preview.get(), "opacity")
            .sync_create()
            .transform_to(|_, value: &str| -> Option<f64> {
                match value.is_empty() {
                    true => Some(0.3),
                    false => Some(1.0),
                }
            })
            .build();
    }

    fn setup_form_validation(slf: &QuickMode) {
        slf.name_input
            .bind_property("text", slf.obj().as_ref(), "name")
            .bidirectional()
            .sync_create()
            .build();

        let show_error = |toast_text: &str, entry_row: &adw::EntryRow| {
            let window = entry_row
                .ancestor(adw::ApplicationWindow::static_type())
                .unwrap();
            let _ = window.activate_action("win.show_toast", Some(&toast_text.to_variant()));
            entry_row.set_css_classes(&["error"]);
            entry_row.grab_focus();
        };

        slf.exec_input.connect_apply(clone!(
            #[weak]
            slf,
            move |entry_row| {
                let text = entry_row.text();
                let path = Path::new(&text);
                let validate_form = *slf.enable_validation.borrow();

                if text.is_empty() {
                    show_error("The executable path is empty", entry_row);
                    return;
                }

                if !path.is_absolute() && validate_form {
                    show_error("Only absolute file paths are allowed", entry_row);
                    return;
                }

                if !path.exists() && validate_form {
                    show_error("The executable file does not exist", entry_row);
                    return;
                }

                if !path.is_file() && validate_form {
                    show_error("The selected file is a directory", entry_row);
                    return;
                }

                entry_row.set_css_classes(&[]);
                slf.obj().set_exec(text);
                slf.save_button.grab_focus();
            }
        ));

        slf.icon_input.connect_apply(clone!(
            #[weak]
            slf,
            move |entry_row| {
                let text = entry_row.text();
                let path = Path::new(&text);

                let validate_form = *slf.enable_validation.borrow();

                if text.is_empty() {
                    show_error("The icon path is empty", entry_row);
                    return;
                }

                if !path.is_absolute() && validate_form {
                    show_error("Only absolute file paths are allowed", entry_row);
                    return;
                }

                if !path.exists() && validate_form {
                    show_error("The icon file does not exist", entry_row);
                    return;
                }

                if !path.is_file() && validate_form {
                    show_error("The selected file is a directory", entry_row);
                    return;
                }

                entry_row.set_css_classes(&[]);
                let image = slf
                    .icon_preview
                    .child()
                    .and_dynamic_cast::<gtk::Image>()
                    .unwrap();
                image.set_from_file(Some(&text));
                slf.obj().set_icon(text);
                slf.exec_input.grab_focus();
            }
        ));

        let name_expression = slf.obj().property_expression("name");
        let exec_expression = slf.obj().property_expression("exec");
        let icon_expression = slf.obj().property_expression("icon");
        ClosureExpression::new::<bool>(
            [&name_expression, &exec_expression, &icon_expression],
            closure!(|_: <QuickMode as ObjectSubclass>::Type,
                      name: String,
                      exec: String,
                      icon: String| {
                !(name.is_empty() || exec.is_empty() || icon.is_empty())
            }),
        )
        .bind(
            &slf.save_button.get(),
            "sensitive",
            Some(slf.obj().as_ref()),
        );
    }

    impl WidgetImpl for QuickMode {}
    impl NavigationPageImpl for QuickMode {}
}

use adw::prelude::BinExt;
use adw::prelude::EntryRowExt;
use gtk::{
    glib::{self},
    prelude::{EditableExt, WidgetExt},
    prelude::{ObjectExt, SettingsExtManual},
    subclass::prelude::ObjectSubclassIsExt,
};

glib::wrapper! {
    pub struct QuickMode(ObjectSubclass<imp::QuickMode>)
    @extends adw::NavigationPage, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl QuickMode {
    pub fn init(&self) {
        let settings = gtk::gio::Settings::new("io.github.andreibachim.shortcut");
        settings
            .bind("create-enable-validation", self, "enable_validation")
            .build();
    }

    pub fn edit_details(
        &self,
        name: Option<String>,
        icon_path: Option<String>,
        exec_path: Option<String>,
    ) {
        if let Some(name) = name {
            *self.imp().old_name.borrow_mut() = name.clone();
            self.set_name(name);
        }

        if let Some(icon_path) = icon_path {
            if !icon_path.is_empty() {
                self.imp().icon_input.set_text(&icon_path);
                self.imp().icon_input.emit_by_name::<()>("apply", &[]);
            }
        }

        if let Some(exec_path) = exec_path {
            if !exec_path.is_empty() {
                self.imp().exec_input.set_text(&exec_path);
                self.imp().exec_input.emit_by_name::<()>("apply", &[]);
                self.imp().exec_input.get().set_show_apply_button(false);
                self.imp().exec_input.get().set_show_apply_button(true);
            }
        }
    }

    pub fn clear_data(&self) {
        let imp = self.imp();
        *imp.old_name.borrow_mut() = "".to_owned();
        self.set_name("");
        imp.name_input.get().delete_text(0, -1);

        imp.exec_input.set_text("");
        self.set_exec("");
        imp.exec_input.set_css_classes(&[]);

        imp.icon_input.set_text("");
        self.set_icon("");
        imp.icon_input.set_css_classes(&[]);

        imp.name_input.grab_focus();

        imp.icon_preview.set_child(Some(
            &gtk::Image::builder()
                .icon_name("preview-placeholder")
                .pixel_size(128)
                .build(),
        ));
    }
}
