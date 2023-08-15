mod imp {
    use std::cell::OnceCell;

    use adw::subclass::prelude::BinImpl;
    use adw::traits::BinExt;
    use gtk::glib::{self, Sender};
    use gtk::subclass::prelude::*;
    use gtk::traits::{BoxExt, ButtonExt, OrientableExt, WidgetExt};

    use crate::component::viewport::Action;

    // Object holding the state
    #[derive(Default)]
    pub struct Completed {
        pub sender: OnceCell<Sender<Action>>,
        container: gtk::Box,
        pub main_menu_button: gtk::Button,
        pub exit_button: gtk::Button,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Completed {
        const NAME: &'static str = "Completed";
        type Type = super::Completed;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for Completed {
        fn constructed(&self) {
            self.parent_constructed();
            let container = &self.container;
            container.set_orientation(gtk::Orientation::Vertical);
            container.set_spacing(64);
            container.set_vexpand(true);
            container.set_hexpand(true);
            container.set_halign(gtk::Align::Center);
            container.set_valign(gtk::Align::Center);
            append_message(container);
            append_menu(container, &self.main_menu_button, &self.exit_button);

            self.obj().set_child(Some(container));
        }
    }

    fn append_message(container: &gtk::Box) {
        let wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::Center)
            .spacing(8)
            .build();

        wrapper.append(
            &gtk::Label::builder()
                .css_classes(vec!["title-1"])
                .label("All done!")
                .build(),
        );
        container.append(&wrapper);
    }

    fn append_menu(
        container: &gtk::Box,
        main_menu_button: &gtk::Button,
        exit_button: &gtk::Button,
    ) {
        let wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .homogeneous(true)
            .spacing(8)
            .build();

        main_menu_button.set_css_classes(&["pill", "suggested-action"]);
        main_menu_button.set_label("Menu");

        wrapper.append(main_menu_button);

        exit_button.set_css_classes(&["pill"]);
        exit_button.set_label("Quit");
        wrapper.append(exit_button);

        container.append(&wrapper);
    }

    impl WidgetImpl for Completed {}
    impl BinImpl for Completed {}
}

use glib::Object;
use gtk::{
    glib::{self, clone, Sender},
    subclass::prelude::ObjectSubclassIsExt,
    traits::ButtonExt,
};

use crate::component::viewport::Action;

glib::wrapper! {
    pub struct Completed(ObjectSubclass<imp::Completed>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Completed {
    pub fn new(sender: Sender<Action>) -> Self {
        let slf = Object::builder::<Self>().build();
        let completed = slf.imp();

        let _ = completed.sender.set(sender);

        completed
            .main_menu_button
            .connect_clicked(clone!(@weak completed => move |_| {
                let _ = completed.sender.get().unwrap().send(Action::Landing(false));
            }));

        completed
            .exit_button
            .connect_clicked(clone!(@weak completed => move |_| {
                let _ = completed.sender.get().unwrap().send(Action::Exit);
            }));

        slf
    }
}
