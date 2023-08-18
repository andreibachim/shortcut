mod imp {

    use crate::component::viewport::Action;
    use adw::subclass::prelude::BinImpl;
    use adw::traits::BinExt;
    use gtk::glib::{self, Sender};
    use gtk::subclass::prelude::*;
    use gtk::traits::BoxExt;
    use std::cell::OnceCell;

    pub struct Landing {
        pub sender: OnceCell<Sender<Action>>,
        container: gtk::Box,
        pub quick_flow_button: gtk::Button,
    }

    impl Default for Landing {
        fn default() -> Self {
            let sender: OnceCell<Sender<Action>> = Default::default();
            let container = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(64)
                .hexpand(true)
                .vexpand(true)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build();

            container.append(&logo());

            let menu_clamp = adw::Clamp::builder()
                .orientation(gtk::Orientation::Horizontal)
                .maximum_size(240)
                .build();

            let quick_flow_button = gtk::Button::builder()
                .css_classes(vec!["suggested-action"])
                .child(
                    &gtk::Label::builder()
                        .margin_top(8)
                        .margin_bottom(8)
                        .label("Quick flow")
                        .build(),
                )
                .build();

            let expert_flow_button = gtk::Button::builder()
                .child(
                    &gtk::Label::builder()
                        .margin_top(8)
                        .margin_bottom(8)
                        .label("Expert flow")
                        .build(),
                )
                .visible(false)
                .build();

            let menu = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(16)
                .build();
            menu.append(&quick_flow_button);
            menu.append(&expert_flow_button);

            menu_clamp.set_child(Some(&menu));
            container.append(&menu_clamp);

            Self {
                container,
                sender,
                quick_flow_button,
            }
        }
    }

    fn logo() -> gtk::Box {
        let logo = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(16)
            .hexpand(true)
            .vexpand(true)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::End)
            .build();
        let logo_image = gtk::Image::builder()
            .icon_name("io.github.andreibachim.shortcut")
            .pixel_size(128)
            .build();
        let logo_subtitle = gtk::Label::builder()
            .css_classes(vec!["dim-label"])
            .use_markup(true)
            .label("Make .desktop files")
            .build();
        logo.append(&logo_image);
        logo.append(&logo_subtitle);
        logo
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Landing {
        const NAME: &'static str = "Landing";
        type Type = super::Landing;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for Landing {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_child(Some(&self.container));
        }
    }
    impl WidgetImpl for Landing {}
    impl BinImpl for Landing {}
}

use glib::Object;
use gtk::{
    glib::{self, clone, Sender},
    subclass::prelude::ObjectSubclassIsExt,
    traits::ButtonExt,
};

use crate::component::viewport::Action;

glib::wrapper! {
    pub struct Landing(ObjectSubclass<imp::Landing>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Landing {
    pub fn new(sender: Sender<Action>) -> Self {
        let slf = Object::builder::<Self>()
            .property("hexpand", true)
            .property("vexpand", true)
            .build();
        let landing = slf.imp();
        landing.sender.set(sender).unwrap();
        landing
            .quick_flow_button
            .connect_clicked(clone!(@weak landing => move |_| {
                let _ = landing.sender.get().unwrap().send(Action::QuickFlow);
            }));
        slf
    }
}
