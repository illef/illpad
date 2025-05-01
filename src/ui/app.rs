use crate::ui::{
    APP_BROKER, RootMsg,
    editor_box::EditorBox,
    file_writer::{FileWriter, FileWriterMsg},
};
use gtk::{gdk, glib};
use log::trace;
use relm4::{
    WorkerController,
    gtk::{CssProvider, prelude::*},
    prelude::*,
    tokio,
};
use std::path::PathBuf;

pub struct App {
    editor_box: Controller<EditorBox>,
    file_writer: WorkerController<FileWriter>,
    save_file_path: PathBuf,
    text_changed: bool,
    editor_changed: bool,
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = RootMsg;
    type Output = ();

    view! {
        #[name = "window"]
        gtk::ApplicationWindow {
            set_title: Some("illpad"),
            // set_default_width: 700,
            set_decorated: false,
            set_expand: true,
            set_vexpand: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "ui"]
                append = &gtk::ScrolledWindow {
                    set_vexpand: true,

                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        append = model.editor_box.widget(),

                        // NOTE: hack for scroll adjustment
                        append = &gtk::Label {
                            set_size_request: (-1, 50),
                        }
                    }
                },

                append = &gtk::Label {
                    add_css_class: "status-label",
                    set_label: "",
                    set_size_request: (-1, 20),
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let file_path = dirs::home_dir().unwrap().join(".cache/illpad/notes.json");

        let mut text_with_tags = vec![];

        if file_path.exists() {
            let text = std::fs::read_to_string(&file_path).unwrap();
            if let Ok(loaded) = serde_json::from_str(&text) {
                text_with_tags = loaded;
            }
        }

        let model = App {
            editor_box: EditorBox::builder().launch(text_with_tags).detach(),
            save_file_path: file_path.clone(),
            file_writer: FileWriter::builder().detach_worker(file_path).forward(
                sender.input_sender(),
                |msg| match msg {
                    FileWriterMsg::SaveComplete => RootMsg::SaveComplete,
                },
            ),
            text_changed: false,
            editor_changed: false,
        };

        sender.command(move |_out, shutdown| {
            shutdown
                .register(async move {
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
                    loop {
                        interval.tick().await;
                        APP_BROKER.send(RootMsg::AutoSaveTickTriggered);
                    }
                })
                .drop_on_shutdown()
        });

        let widgets = view_output!();

        // widgets.ui.vadjustment().connect_changed(|v| {
        //     if v.value() < v.upper() {
        //         v.set_value(v.upper());
        //     }
        // });

        let css_provider = CssProvider::new();
        css_provider.load_from_data(include_str!("default.css"));

        gtk::style_context_add_provider_for_display(
            &widgets.ui.display(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        add_key_pressed_event(&widgets.window);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        trace!("App Message received {:?}", msg);
        match msg {
            RootMsg::TextChanged => {
                self.text_changed = true;
            }
            RootMsg::EditorChanged => {
                self.editor_changed = true;
            }
            RootMsg::SaveComplete => {}
            RootMsg::ExitTriggered => {
                if self.text_changed || self.editor_changed {
                    FileWriter::save(
                        &self.save_file_path,
                        self.editor_box.model().get_text_with_tags(),
                    );
                }
                relm4::main_application().quit();
            }
            RootMsg::AutoSaveTickTriggered => {
                if self.text_changed || self.editor_changed {
                    self.text_changed = false;
                    self.editor_changed = false;
                    self.file_writer
                        .emit(self.editor_box.model().get_text_with_tags());
                }
            }
        }
    }
}

fn add_key_pressed_event(window: &gtk::ApplicationWindow) {
    let event_controller = gtk::EventControllerKey::new();

    event_controller.connect_key_pressed(|_, key, _, _| {
        match key {
            gdk::Key::Escape => {
                APP_BROKER.send(RootMsg::ExitTriggered);
            }
            _ => (),
        }
        glib::Propagation::Proceed
    });

    window.add_controller(event_controller);
}
