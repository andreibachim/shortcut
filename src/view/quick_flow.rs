mod imp {
    use std::cell::{OnceCell, RefCell};

    use std::path::Path;

    use adw::traits::{BinExt, EntryRowExt, PreferencesRowExt};

    use gtk::glib::{self, clone, closure, Properties, Sender};
    use gtk::prelude::{FileExt, GObjectPropertyExpressionExt, ObjectExt};
    use gtk::subclass::prelude::*;
    use gtk::traits::{BoxExt, ButtonExt, EditableExt, ListBoxRowExt, OrientableExt, WidgetExt};
    use gtk::ClosureExpression;

    use crate::component::viewport::Action;
    use crate::model::Desktop;

    #[derive(Properties)]
    #[properties(wrapper_type = super::QuickFlow)]
    pub struct QuickFlow {
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "exec", get, set, type = String, member = exec)]
        #[property(name = "icon", get, set, type = String, member = icon)]
        pub data: RefCell<Desktop>,
        pub sender: OnceCell<Sender<Action>>,
        pub navbar: gtk::CenterBox,
        pub back_button: gtk::Button,
        pub save_button: gtk::Button,
        pub preview: gtk::Box,
        pub name_input: adw::EntryRow,
        pub name_preview: gtk::Label,
        pub form: adw::Clamp,
        pub icon_location: adw::EntryRow,
        pub icon_preview: adw::Bin,
        pub exec_location: adw::EntryRow,
    }

    impl Default for QuickFlow {
        fn default() -> Self {
            let sender = OnceCell::new();

            let navbar = gtk::CenterBox::new();
            let back_button = gtk::Button::builder()
                // .css_classes(vec!["flat"])
                .label("Back")
                .build();
            let save_button = gtk::Button::builder()
                .css_classes(vec!["suggested-action"])
                .label("Save")
                .build();
            config_navbar(&navbar, &back_button, &save_button);

            let preview = gtk::Box::builder().build();
            let icon_preview = adw::Bin::new();
            let name_preview = gtk::Label::new(Some("Application name"));

            let name_input = adw::EntryRow::new();
            configure_preview(&preview, &icon_preview, &name_preview, &name_input);

            let form = adw::Clamp::new();
            let icon_location = adw::EntryRow::new();
            let exec_location = adw::EntryRow::new();
            configure_form(
                &form,
                &name_input,
                &exec_location,
                &icon_location,
                &icon_preview,
            );

            let data = RefCell::new(Desktop::new());

            Self {
                data,
                sender,
                navbar,
                back_button,
                save_button,
                preview,
                name_preview,
                icon_preview,
                form,
                name_input,
                icon_location,
                exec_location,
            }
        }
    }

    fn config_navbar(
        navbar: &gtk::CenterBox,
        back_button: &gtk::Button,
        save_button: &gtk::Button,
    ) {
        navbar.set_css_classes(&["toolbar", "osd"]);
        navbar.set_margin_bottom(16);
        navbar.set_margin_start(32);
        navbar.set_margin_end(32);
        navbar.set_valign(gtk::Align::End);

        navbar.set_start_widget(Some(back_button));
        navbar.set_end_widget(Some(save_button));
    }

    fn configure_preview(
        preview: &gtk::Box,
        icon_preview: &adw::Bin,
        name_preview: &gtk::Label,
        name_input: &adw::EntryRow,
    ) {
        preview.set_orientation(gtk::Orientation::Vertical);
        preview.set_spacing(16);

        icon_preview.set_child(Some(
            &gtk::Image::builder()
                .icon_name("image-missing-symbolic")
                .pixel_size(128)
                .build(),
        ));
        name_preview.set_css_classes(&["title-2"]);
        name_input
            .bind_property("text", name_preview, "label")
            .sync_create()
            .transform_to(|_, value: &str| -> Option<&str> {
                match value.is_empty() {
                    true => Some("Application name"),
                    false => Some(value),
                }
            })
            .build();
        name_input
            .bind_property("text", name_preview, "opacity")
            .sync_create()
            .transform_to(|_, value: &str| -> Option<f64> {
                match value.is_empty() {
                    true => Some(0.3),
                    false => Some(1.0),
                }
            })
            .build();
        preview.append(icon_preview);
        preview.append(name_preview);
    }

    fn configure_form(
        form: &adw::Clamp,
        name_input: &adw::EntryRow,
        exec_location: &adw::EntryRow,
        icon_location: &adw::EntryRow,
        icon_preview: &adw::Bin,
    ) {
        form.set_maximum_size(480);
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(16)
            .build();

        container.append(&build_name_input(name_input));
        container.append(&build_exec_input(exec_location));
        container.append(&build_icon_input(icon_location, icon_preview));

        form.set_child(Some(&container));
    }

    fn build_name_input(name_input: &adw::EntryRow) -> gtk::ListBox {
        let wrapper = gtk::ListBox::builder()
            .css_classes(vec!["boxed-list"])
            .build();
        name_input.set_title("Application Name");
        wrapper.append(name_input);
        wrapper
    }

    fn build_exec_input(exec_location: &adw::EntryRow) -> gtk::ListBox {
        let input_wrapper = gtk::ListBox::builder()
            .css_classes(vec!["boxed-list"])
            .build();
        exec_location.set_title("Executable Path");
        exec_location.set_show_apply_button(true);
        input_wrapper.append(exec_location);
        input_wrapper.set_hexpand(true);
        input_wrapper.set_halign(gtk::Align::Fill);

        let upload_button = gtk::Button::builder()
            .css_classes(vec!["circular", "flat"])
            .icon_name("document-open-symbolic")
            .valign(gtk::Align::Center)
            .build();

        upload_button.connect_clicked(clone!(@weak exec_location => move |_| {
            let filters_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
            let executable_filter = gtk::FileFilter::new();
            executable_filter.set_name(Some("Executable files"));
            executable_filter.add_mime_type("application/x-executable");
            filters_store.append(&executable_filter);

            let all_filter = gtk::FileFilter::new();
            all_filter.add_pattern("*");
            all_filter.set_name(Some("All files"));
            filters_store.append(&all_filter);

            let dialog = gtk::FileDialog::builder().filters(&filters_store).modal(true).title("Select Executable File").build();
            dialog.open(None::<&gtk::Window>, None::<&gtk::gio::Cancellable>, move |file| {
                if let Ok(file) = file {
                    exec_location.set_text(file.path().expect("Invalid file path").to_str().expect("Path is not UTF-8 compliant"));
                    exec_location.emit_by_name::<()>("apply", &[]);
                }
            });

        }));
        exec_location.add_suffix(&upload_button);
        exec_location.set_selectable(false);

        input_wrapper
    }

    fn build_icon_input(icon_location: &adw::EntryRow, icon_preview: &adw::Bin) -> gtk::ListBox {
        let wrapper = gtk::ListBox::builder()
            .css_classes(vec!["boxed-list"])
            .build();
        icon_location.set_selectable(false);
        icon_location.set_show_apply_button(true);
        icon_location.set_title("Icon Location");

        let upload_button = gtk::Button::builder()
            .css_classes(vec!["circular", "flat"])
            .icon_name("document-open-symbolic")
            .valign(gtk::Align::Center)
            .build();

        upload_button.connect_clicked(clone!(@weak icon_location, @weak icon_preview => move |_| {
            let filters_store = gtk::gio::ListStore::new::<gtk::FileFilter>();
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Image files"));
            filter.add_mime_type("image/svg+xml");
            filter.add_mime_type("image/png");
            filters_store.append(&filter);

            let file_dialog = gtk::FileDialog::builder().filters(&filters_store).title("Select Icon File").modal(true).build();
            file_dialog.open(None::<&gtk::Window>, None::<&gtk::gio::Cancellable>, move |file| {
                if let Ok(file) = file {
                    icon_location.set_text(file.path().expect("Could not extract path from file").to_str().expect("Path is not UTF-8 compliant"));
                    icon_location.emit_by_name::<()>("apply", &[]);
                }
            });
        }));

        icon_location.add_suffix(&upload_button);
        wrapper.append(icon_location);
        wrapper
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QuickFlow {
        const NAME: &'static str = "QuickFlow";
        type Type = super::QuickFlow;
        type ParentType = gtk::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for QuickFlow {
        fn constructed(&self) {
            self.parent_constructed();

            let layout = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .hexpand(true)
                .vexpand(true)
                .build();

            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(64)
                .vexpand(true)
                .vexpand(true)
                .valign(gtk::Align::Center)
                .build();

            container.append(&self.preview);
            container.append(&self.form);

            layout.append(&container);
            layout.append(&self.navbar);

            self.obj().append(&layout);

            self.name_input
                .bind_property("text", self.obj().as_ref(), "name")
                .sync_create()
                .build();

            self.exec_location
                .connect_apply(clone!(@weak self as slf => move |entry_row| {
                    let text = entry_row.text();
                    let path = Path::new(&text);
                    if path.exists() && path.is_file() {
                        entry_row.set_css_classes(&[]);
                        slf.obj().set_exec(text);
                    } else {
                        let _ = slf.sender.get().expect("Could not get sender").send(Action::ShowToast("The executable path is not valid".to_owned()));
                        entry_row.set_css_classes(&["error"]);
                    }
                }));

            self.icon_location
                .connect_apply(clone!(@weak self as slf => move |entry_row| {
                    let text = entry_row.text();
                    let path = Path::new(&text);
                    if path.exists() && path.is_file() {
                        entry_row.set_css_classes(&[]);
                        slf.obj().set_icon(text);
                        slf.icon_preview.set_child(
                            Some(&gtk::Image::builder().file(entry_row.text()).pixel_size(128).css_classes(vec!["icon-dropshadow"]).build())
                        );
                    } else {
                        let _ = slf.sender.get().expect("Could not get sender").send(Action::ShowToast("The icon path is not valid".to_owned()));
                        entry_row.set_css_classes(&["error"]);
                    }
                }));

            let name_expression = self.obj().property_expression("name");
            let exec_expression = self.obj().property_expression("exec");
            let icon_expression = self.obj().property_expression("icon");
            ClosureExpression::new::<bool>(
                [&name_expression, &exec_expression, &icon_expression],
                closure!(|_: Self::Type, name: String, exec: String, icon: String| {
                    !(name.is_empty() || exec.is_empty() || icon.is_empty())
                }),
            )
            .bind(&self.save_button, "sensitive", Some(self.obj().as_ref()));
        }
    }
    impl WidgetImpl for QuickFlow {}
    impl BoxImpl for QuickFlow {}
}

use std::{fs::File, io::Write};

use glib::Object;
use gtk::{
    glib::{self, clone, Sender},
    subclass::prelude::ObjectSubclassIsExt,
    traits::ButtonExt,
};

use crate::component::viewport::Action;

glib::wrapper! {
    pub struct QuickFlow(ObjectSubclass<imp::QuickFlow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl QuickFlow {
    pub fn new(sender: Sender<Action>) -> Self {
        let slf = Object::builder::<Self>()
            .property("hexpand", true)
            .property("vexpand", true)
            .build();
        let quick_flow = slf.imp();
        quick_flow.sender.set(sender).expect("Could not set sender");
        quick_flow
            .back_button
            .connect_clicked(clone!(@weak quick_flow => move |_| {
                let _ = quick_flow.sender.get().expect("Could not get sender").send(Action::Landing(true));
            }));

        quick_flow
            .save_button
            .connect_clicked(clone!(@weak quick_flow => move |_| {
                let data = quick_flow.data.borrow();
                let file_path = dirs::home_dir().expect("Could not get the home directory").join(format!(".local/share/applications/{}.desktop", data.name.replace(' ', "-").to_lowercase()));
                let mut file = File::create(file_path).expect("Could not create a new file");
                file.write_all(data.get_output().expect("Could not serialize desktop file for writing").as_bytes())
                    .expect("Could not write to .desktop file.");
                let _ = quick_flow.sender.get().expect("Could not get sender").send(Action::Completed);
            }));

        slf
    }
}
