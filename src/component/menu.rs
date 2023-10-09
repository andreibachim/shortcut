mod imp {
    use adw::subclass::prelude::BinImpl;
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/component/menu.ui")]
    pub struct Menu {}

    #[glib::object_subclass]
    impl ObjectSubclass for Menu {
        const NAME: &'static str = "Menu";
        type Type = super::Menu;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Menu {}
    impl WidgetImpl for Menu {}
    impl BinImpl for Menu {}
}

use gtk::glib;

glib::wrapper! {
    pub struct Menu(ObjectSubclass<imp::Menu>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
