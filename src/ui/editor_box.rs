use gtk::{gdk, glib};
use relm4::{gtk, gtk::prelude::*, prelude::*};

use crate::text::TextWithTags;

use super::{APP_BROKER, RootMsg};

#[derive(Clone, PartialEq, Debug)]
pub struct Editor {
    pub content: gtk::TextBuffer,
}

#[derive(Debug)]
pub struct GrabFocus;

#[derive(Debug)]
pub enum EditorMsg {
    RequestAddNoteFrom(DynamicIndex),
    RequestDeleteNoteFrom(DynamicIndex),
    ReuestFocusUpFrom(DynamicIndex),
    ReuestFocusDownFrom(DynamicIndex),
    TextChanged,
}

#[relm4::factory(pub)]
impl FactoryComponent for Editor {
    type Init = TextWithTags;
    type Input = GrabFocus;
    type Output = EditorMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_vexpand:false,
            add_css_class: "editor-container",
            #[name (text_view)]
            gtk::TextView {
                add_css_class: "editor-text-view",
                add_css_class: "editor-normal-text-view",
                set_hexpand: true,
                set_vexpand: true,
                set_editable: true,
                set_focusable: true,
                #[watch]
                grab_focus: (),
                set_wrap_mode: gtk::WrapMode::WordChar,
                set_buffer: Some(&self.content),
            }
        }
    }

    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: Self::Root,
        _: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widget = view_output!();

        add_css_class_by_focus(&widget.text_view);
        add_key_pressed_event(&widget.text_view, index.clone(), sender.clone());

        self.content.connect_changed(move |_| {
            sender.output(EditorMsg::TextChanged).unwrap();
        });

        widget.text_view.grab_focus();

        widget
    }

    fn init_model(content: Self::Init, _: &DynamicIndex, _: FactorySender<Self>) -> Self {
        Self {
            content: content.as_text_buffer(),
        }
    }

    fn update(&mut self, _: Self::Input, _sender: FactorySender<Self>) {}
}

#[derive(Debug)]
pub struct EditorBox {
    pub editors: FactoryVecDeque<Editor>,
}

impl EditorBox {
    pub fn get_text_with_tags(&self) -> Vec<TextWithTags> {
        self.editors
            .iter()
            .map(|e| TextWithTags::from(&e.content, e.content.start_iter(), e.content.end_iter()))
            .collect()
    }
}

#[relm4::component(pub)]
impl SimpleComponent for EditorBox {
    type Init = Vec<TextWithTags>;
    type Input = EditorMsg;
    type Output = ();

    view! {
        gtk::Box {
            #[local_ref]
            editor_box -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
            }
        }
    }

    fn init(
        mut text_with_tags: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let editors = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), std::convert::identity);

        if text_with_tags.is_empty() {
            let text = r#"Welcome to the illpad!
Ctrl + Enter           Add new block below the current block"#;

            text_with_tags.push(TextWithTags::from_str(text));
        }

        let mut model = EditorBox { editors };

        for text_with_tags in text_with_tags {
            model.editors.guard().push_back(text_with_tags);
        }

        let editor_box = model.editors.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            EditorMsg::TextChanged => {
                APP_BROKER.send(RootMsg::TextChanged);
            }
            EditorMsg::RequestAddNoteFrom(index) => {
                self.editors
                    .guard()
                    .insert(index.current_index() + 1, TextWithTags::default());
                APP_BROKER.send(RootMsg::EditorChanged);
            }
            EditorMsg::RequestDeleteNoteFrom(index) => {
                if self.editors.len() == 1 {
                    return;
                }
                let index = index.current_index();

                let mut editors = self.editors.guard();
                editors.remove(index);

                if index > 0 {
                    editors.send(index - 1, GrabFocus {});
                }
                APP_BROKER.send(RootMsg::EditorChanged);
            }
            EditorMsg::ReuestFocusUpFrom(index) => {
                if index.current_index() == 0 {
                    return;
                }

                let editors = self.editors.guard();
                let index = index.current_index().wrapping_sub(1);
                editors.send(index, GrabFocus {});
            }
            EditorMsg::ReuestFocusDownFrom(index) => {
                if index.current_index() == self.editors.len() - 1 {
                    return;
                }

                let editors = self.editors.guard();
                let index = index.current_index().wrapping_add(1);
                editors.send(index, GrabFocus {});
            }
        }
    }
}

pub fn add_key_pressed_event(
    text_view: &gtk::TextView,
    index: DynamicIndex,
    sender: FactorySender<Editor>,
) {
    let event_controller = gtk::EventControllerKey::new();

    let text_view_clone = text_view.clone();

    event_controller.connect_key_pressed(move |_, key, _, modifier| {
        return match key {
            gdk::Key::BackSpace => {
                if text_view_clone.buffer().end_iter().offset() == 0
                    || modifier.contains(gdk::ModifierType::CONTROL_MASK)
                {
                    sender
                        .output(EditorMsg::RequestDeleteNoteFrom(index.clone()))
                        .unwrap();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
            gdk::Key::Return => {
                if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                    sender
                        .output(EditorMsg::RequestAddNoteFrom(index.clone()))
                        .unwrap();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            gdk::Key::c => {
                if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                    if let Some((start, end)) = text_view_clone.buffer().selection_bounds() {
                        let clipboard_text =
                            TextWithTags::from(&text_view_clone.buffer(), start, end)
                                .clipboard_text();

                        text_view_clone.clipboard().set_text(&clipboard_text);
                        return glib::Propagation::Stop;
                    }
                }
                glib::Propagation::Proceed
            }
            gdk::Key::h => {
                if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                    if let Some((start, end)) = text_view_clone.buffer().selection_bounds() {
                        text_view_clone
                            .buffer()
                            .apply_tag_by_name("highlight", &start, &end);
                    }
                }
                glib::Propagation::Proceed
            }
            gdk::Key::b => {
                if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                    if let Some((start, end)) = text_view_clone.buffer().selection_bounds() {
                        text_view_clone
                            .buffer()
                            .apply_tag_by_name("bold", &start, &end);
                    }
                }
                glib::Propagation::Proceed
            }
            gdk::Key::j => {
                if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                    sender
                        .output(EditorMsg::ReuestFocusDownFrom(index.clone()))
                        .unwrap();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            gdk::Key::k => {
                if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
                    sender
                        .output(EditorMsg::ReuestFocusUpFrom(index.clone()))
                        .unwrap();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            _ => glib::Propagation::Proceed,
        };
    });

    text_view.add_controller(event_controller);
}

pub fn add_css_class_by_focus(text_view: &gtk::TextView) {
    text_view.connect_has_focus_notify(|text_view| {
        if text_view.has_focus() {
            text_view.remove_css_class("editor-normal-text-view");
            text_view.add_css_class("editor-focused-text-view");
        } else {
            text_view.remove_css_class("editor-focused-text-view");
            text_view.add_css_class("editor-normal-text-view");
        }
    });
}
