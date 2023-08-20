mod imp {

    use crate::component::viewport::Action;

    use gtk::glib::subclass::InitializingObject;
    use gtk::glib::{self, Sender};
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use std::cell::OnceCell;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/landing.ui")]
    pub struct Landing {
        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Landing {
        const NAME: &'static str = "Landing";
        type Type = super::Landing;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("menu.quick_mode", None, move |landing, _, _| {
                let imp = landing.imp();
                let _ = imp.sender.get().unwrap().send(Action::QuickFlow);
            });
        }
        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Landing {}
    impl WidgetImpl for Landing {}
    impl BoxImpl for Landing {}
}

use glib::Object;
use gtk::{
    glib::{self, Sender},
    subclass::prelude::ObjectSubclassIsExt,
};

use crate::component::viewport::Action;

glib::wrapper! {
    pub struct Landing(ObjectSubclass<imp::Landing>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Landing {
    pub fn new(sender: Sender<Action>) -> Self {
        let slf = Object::builder::<Self>().build();
        let landing = slf.imp();
        landing.sender.set(sender).unwrap();
        slf
    }
}
