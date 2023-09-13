mod imp {
    use adw::subclass::preferences_row::PreferencesRowImpl;
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/component/entry.ui")]
    pub struct Entry {
        #[template_child]
        pub app_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle: TemplateChild<gtk::Label>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub edit_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Entry {
        const NAME: &'static str = "Entry";
        type Type = super::Entry;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Entry {}
    impl WidgetImpl for Entry {}
    impl ListBoxRowImpl for Entry {}
    impl PreferencesRowImpl for Entry {}
}

use adw::prelude::WidgetExt;
use glib::Object;
use gtk::{
    glib,
    prelude::{ActionableExtManual, ToVariant},
    subclass::prelude::ObjectSubclassIsExt,
};

glib::wrapper! {
    pub struct Entry(ObjectSubclass<imp::Entry>)
        @extends adw::PreferencesRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Entry {
    pub fn new(
        title: Option<&str>,
        subtitle: Option<&str>,
        icon_path: Option<&str>,
        exec_path: Option<&str>,
    ) -> Self {
        let slf = Object::builder::<Self>().build();
        let imp = slf.imp();

        let name = title.unwrap_or("").trim();
        let subtitle_text = subtitle.unwrap_or("").trim();
        let icon_path_str = icon_path.unwrap_or("");
        let exec_path_str = exec_path.unwrap_or("");

        crate::function::set_icon(&imp.app_icon.get(), icon_path);

        imp.title.set_label(title.unwrap_or(""));
        imp.subtitle
            .set_label(&format!("<small>{}</small>", subtitle_text));
        imp.subtitle.set_tooltip_text(Some(subtitle_text));

        imp.delete_button
            .set_action_target(Some(&(subtitle_text, name).to_variant()));
        imp.edit_button
            .set_action_target(Some(&(name, icon_path_str, exec_path_str).to_variant()));

        slf
    }

    pub fn get_name(&self) -> String {
        self.imp().title.text().to_string()
    }
}
