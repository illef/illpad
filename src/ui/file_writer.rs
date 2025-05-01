use std::path::{Path, PathBuf};

use log::trace;
use relm4::{ComponentSender, Worker};

use crate::text::TextWithTags;

pub struct FileWriter {
    path: PathBuf,
}

#[derive(Debug)]
pub enum FileWriterMsg {
    SaveComplete,
}

impl FileWriter {
    pub fn save(path: &Path, input: Vec<TextWithTags>) {
        trace!("FileWriter::save start");

        let dir = dirs::home_dir().unwrap().join(".cache/illpad");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).unwrap();
        }

        if !path.exists() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
            }
        }

        if let Ok(json) = serde_json::to_string_pretty(&input) {
            std::fs::write(path, json).unwrap();
        }

        trace!("FileWriter::save finish");
    }
}

impl Worker for FileWriter {
    type Init = PathBuf;
    type Input = Vec<TextWithTags>;
    type Output = FileWriterMsg;

    fn init(path: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self { path }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        Self::save(self.path.as_path(), input);
        sender.output(FileWriterMsg::SaveComplete).unwrap();
    }
}
