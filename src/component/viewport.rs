mod imp {
    use std::{cell::RefCell, sync::Arc, time::Duration};

    use adw::{
        prelude::{ButtonExt, WidgetExt},
        subclass::prelude::*,
    };
    use gtk::{
        glib::{self, clone, subclass::InitializingObject},
        prelude::CastNone,
        CompositeTemplate,
    };

    use super::Action;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/andreibachim/shortcut/component/viewport.ui")]
    pub struct Viewport {
        #[template_child]
        toast_overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        toast_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        toast_label: TemplateChild<gtk::Label>,

        current_toast: Arc<RefCell<i64>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Viewport {
        const NAME: &'static str = "Viewport";
        type Type = super::Viewport;
        type ParentType = adw::Bin;

        fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("close_toast", None, move |viewport, _, _| {
                let imp = viewport.imp();
                imp.toast_revealer.set_reveal_child(false);
            })
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Viewport {
        fn constructed(&self) {
            self.parent_constructed();

            let (sender, r) = gtk::glib::MainContext::channel(gtk::glib::Priority::default());
            let receiver = RefCell::new(Some(r));

            let carousel = self.carousel.get();
            let toast_overlay = self.toast_overlay.get();
            let toast_revealer = self.toast_revealer.get();
            let toast_label = self.toast_label.get();
            let current_toast = self.current_toast.clone();

            let landing_view = crate::view::Landing::new(sender.clone());
            carousel.append(&landing_view);
            let quick_mode_view = crate::view::QuickMode::new(sender.clone());
            carousel.append(&quick_mode_view);
            let completed_view = crate::view::Completed::new(sender.clone());
            carousel.append(&completed_view);
            let manage_view = crate::view::Manage::new(sender.clone());
            carousel.append(&manage_view);

            receiver.borrow_mut().take().unwrap().attach(
                None,
                clone!(@strong carousel,
                    @strong toast_overlay,
                    @strong toast_revealer,
                    @strong toast_label,
                    @strong current_toast => move |action| {
                    let disable_focus_on_all_children = || {
                        for view_index in 0..carousel.n_pages() {
                            carousel.nth_page(view_index).set_sensitive(false);
                        }
                    };
                    toast_revealer.set_reveal_child(false);
                    match action {
                        Action::Back => {
                            disable_focus_on_all_children();
                            let previous_view = carousel.nth_page((carousel.position() as u32) - 1);
                            previous_view.set_sensitive(true);
                            carousel.scroll_to(&previous_view, true);
                        }
                        Action::Landing(scroll) => {
                            disable_focus_on_all_children();
                            landing_view.set_sensitive(true);
                            carousel.scroll_to(&landing_view, scroll);
                        },
                        Action::QuickFlow(name, icon_path, exec_path) => {
                            disable_focus_on_all_children();
                            quick_mode_view.set_sensitive(true);
                            carousel.reorder(&quick_mode_view, (carousel.position() as i32) + 1);
                            quick_mode_view.clear_data();
                            quick_mode_view.edit_details(name,
                                icon_path,
                                exec_path);
                            carousel.scroll_to(&quick_mode_view, true);
                        },
                        Action::Completed => {
                            disable_focus_on_all_children();
                            completed_view.set_sensitive(true);
                            carousel.reorder(&completed_view, (carousel.position() as i32) + 1);
                            carousel.scroll_to(&completed_view, true);
                        },
                        Action::Manage => {
                            disable_focus_on_all_children();
                            manage_view.set_sensitive(true);
                            carousel.reorder(&manage_view, (carousel.position() as i32) + 1);
                            carousel.scroll_to(&manage_view, true);
                            manage_view.load(false);
                        },
                        Action::ShowToast(toast, widget) => {
                            toast_revealer.child().and_dynamic_cast::<gtk::CenterBox>()
                                .expect("Child of toast revelear is not a GtkCenterBox")
                                .end_widget().and_dynamic_cast::<gtk::Button>().expect("End child of revealer box is not a GtkButton")
                                .connect_clicked(move |_| {
                                    widget.grab_focus();
                                });

                            let current_time = gtk::glib::real_time();
                            *current_toast.borrow_mut() = current_time;

                            let (sender, receiver) = gtk::glib::MainContext::channel(gtk::glib::Priority::default());

                            gtk::gio::spawn_blocking(clone!(@strong sender => move || {
                                std::thread::sleep(Duration::from_secs(4));
                                let _ = sender.send(());
                            }));

                            receiver.attach(None, clone!(@strong toast_revealer, @strong current_toast => move |_: ()| {
                                if current_time == *current_toast.borrow() { toast_revealer.set_reveal_child(false); };
                                gtk::glib::ControlFlow::Continue
                            }));

                            toast_label.set_label(&toast);
                            toast_revealer.set_reveal_child(true);
                        }
                    }
                    gtk::glib::ControlFlow::Continue
                }),
            );
        }
    }
    impl WidgetImpl for Viewport {}
    impl BinImpl for Viewport {}
}

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct Viewport(ObjectSubclass<imp::Viewport>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Viewport {
    pub fn new() -> Self {
        Object::builder::<Self>().build()
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
pub enum Action {
    Landing(bool),
    Back,
    QuickFlow(Option<String>, Option<String>, Option<String>),
    Completed,
    Manage,
    ShowToast(String, gtk::Widget),
}
