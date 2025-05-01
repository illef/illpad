use app::App;
use relm4::{MessageBroker, RelmApp};

pub mod app;
pub mod editor_box;
pub mod file_writer;

#[derive(Debug)]
pub enum RootMsg {
    TextChanged,
    EditorChanged,
    SaveComplete,
    AutoSaveTickTriggered,
    ExitTriggered,
}

pub(crate) static APP_BROKER: MessageBroker<RootMsg> = MessageBroker::new();

pub fn run_app() {
    let app = RelmApp::new("illef.illpad")
        .with_args(vec![])
        .with_broker(&APP_BROKER);
    app.run::<App>(());
}
