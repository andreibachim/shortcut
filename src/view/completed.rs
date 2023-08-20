mod imp {
    use std::cell::OnceCell;

    use gtk::glib::subclass::InitializingObject;
    use gtk::glib::{self, Sender};
    use gtk::subclass::prelude::*;

    use gtk::CompositeTemplate;

    use crate::component::viewport::Action;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/completed.ui")]
    pub struct Completed {
        pub sender: OnceCell<Sender<Action>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Completed {
        const NAME: &'static str = "Completed";
        type Type = super::Completed;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("completed.main-menu", None, move |completed, _, _| {
                let imp = completed.imp();
                let _ = imp.sender.get().unwrap().send(Action::Landing(false));
            });
        }
        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Completed {}
    impl WidgetImpl for Completed {}
    impl BoxImpl for Completed {}
}

use glib::Object;
use gtk::{
    glib::{self, Sender},
    subclass::prelude::ObjectSubclassIsExt,
};

use crate::component::viewport::Action;

glib::wrapper! {
    pub struct Completed(ObjectSubclass<imp::Completed>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Completed {
    pub fn new(sender: Sender<Action>) -> Self {
        let slf = Object::builder::<Self>().build();
        let _ = slf.imp().sender.set(sender);
        slf
    }
}
