mod imp {
    use std::{cell::RefCell, time::Duration};

    use adw::{subclass::prelude::*, traits::BinExt};
    use gtk::{
        glib::{self, clone},
        traits::{WidgetExt, ButtonExt},
    };

    use super::Action;

    pub struct Viewport {
        carousel: adw::Carousel,
        toast_overlay: gtk::Overlay,
    }

    impl Default for Viewport {
        fn default() -> Self {
            let (sender, r) = gtk::glib::MainContext::channel(gtk::glib::Priority::default());
            let receiver = RefCell::new(Some(r));

            let toast_overlay = gtk::Overlay::new();

            let carousel = adw::Carousel::builder()
                .interactive(false)
                .hexpand(true)
                .vexpand(true)
                .build();

            let landing_view = crate::view::Landing::new(sender.clone());
            carousel.append(&landing_view);
            
            let quick_flow_view = crate::view::QuickFlow::new(sender.clone());
            quick_flow_view.set_sensitive(false);
            carousel.append(&quick_flow_view);
            
            let confirmation = crate::view::Completed::new(sender.clone());
            confirmation.set_sensitive(false);
            carousel.append(&confirmation);

            receiver.borrow_mut().take().unwrap().attach(
                None,
                clone!(@strong carousel, @strong toast_overlay => move |action| {
                    let disable_all_children = || {
                        for view_index in 0..carousel.n_pages() {
                            carousel.nth_page(view_index).set_sensitive(false);
                        }
                    };
                    match action {
                        Action::Landing(scroll) => {
                            disable_all_children();
                            landing_view.set_sensitive(true);
                            carousel.scroll_to(&landing_view, scroll);
                        },
                        Action::QuickFlow => {
                            disable_all_children();
                            quick_flow_view.set_sensitive(true);
                            carousel.reorder(&quick_flow_view, 1);
                            carousel.scroll_to(&quick_flow_view, true);
                        },
                        Action::Completed => {
                            disable_all_children();
                            confirmation.set_sensitive(true);
                            carousel.reorder(&confirmation, (carousel.position() + 1.0) as i32);
                            carousel.scroll_to(&confirmation, true);
                        },
                        Action::ShowToast(toast) => {

                            let close_button = gtk::Button::builder()
                                .css_classes(vec!["flat", "circular"])
                                .icon_name("window-close-symbolic")
                                .build();

                            let toast_container = gtk::CenterBox::builder()
                                .css_classes(vec!["app-notification", "osd"])
                                .margin_start(32)
                                .margin_end(32)
                                .valign(gtk::Align::Start)
                                .hexpand(true)
                                .start_widget(
                                    &gtk::Image::builder()
                                        .css_classes(vec!["error"])
                                        .icon_name("emblem-important-symbolic")
                                        .margin_start(8)
                                        .pixel_size(24)
                                        .build(),
                                )
                                .end_widget(&close_button)
                                .build();
                
                            let toast_revealer = gtk::Revealer::builder()
                                .transition_type(gtk::RevealerTransitionType::SwingDown)
                                .transition_duration(400)
                                .focusable(false)
                                .child(&toast_container)
                                .build();
                
                            close_button.connect_clicked(
                                clone!(@weak toast_overlay, @weak toast_revealer => move |_| {
                                    toast_revealer.set_reveal_child(false);
                                    toast_revealer.connect_child_revealed_notify(clone!(@weak toast_overlay => move |revealer| {
                                        toast_overlay.remove_overlay(revealer);
                                    }));
                                }),
                            );
                            toast_container.set_center_widget(Some(&gtk::Label::new(Some(&toast))));

                            let (sender, receiver) = gtk::glib::MainContext::channel(gtk::glib::Priority::default());
                            gtk::gio::spawn_blocking(clone!(@strong sender => move || {
                                std::thread::sleep(Duration::from_secs(4));
                                let _ = sender.send(());
                            }));
                            receiver.attach(None, clone!(@strong toast_overlay, @strong toast_revealer => move |_:()| {
                                toast_revealer.set_reveal_child(false);
                                toast_revealer.connect_child_revealed_notify(clone!(@weak toast_overlay => move |revealer| {
                                    if revealer.parent().is_some() { revealer.unparent() }
                                }));

                                gtk::glib::ControlFlow::Continue
                            }));
                            toast_overlay.add_overlay(&toast_revealer);
                            toast_revealer.set_reveal_child(true);
                        }
                    }
                    gtk::glib::ControlFlow::Continue
                }),
            );

            Self {
                carousel,
                toast_overlay,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Viewport {
        const NAME: &'static str = "Viewport";
        type Type = super::Viewport;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for Viewport {
        fn constructed(&self) {
            self.parent_constructed();
            self.toast_overlay.set_child(Some(&self.carousel));
            self.obj().set_child(Some(&self.toast_overlay));
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
    QuickFlow,
    Completed,
    ShowToast(String),
}
