use std::path::PathBuf;

use iced::{self, Application, Element, Theme, executor::{self, Executor}, Command, widget::{text::Text, row}};
use zalgo_codec_common::{ZalgoString, zalgo_decode};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
enum InputSource {
    TextField(String),
    File(PathBuf),
}

#[derive(Debug, Clone)]
enum OutputDestination {
    TextField,
    File(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodecActionKind {
    Encode,
    Decode,
    Wrap,
    Unwrap,
}

#[derive(Debug, Clone)]
struct CodecAction {
    input_source: InputSource,
    output_dest: OutputDestination,
    action_kind: CodecActionKind,
}

#[derive(Debug)]
enum ToplevelMessage {
    BackendAction(CodecAction),
    GuiAction,
}

struct ZalgoCodecGui();

impl Application for ZalgoCodecGui {
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;
    type Message = ToplevelMessage;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (ZalgoCodecGui(), Command::none())
    }

    fn title(&self) -> String {
        String::from("zalgo codec GUI")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        row![Text::new("Hello, world!")].into()
    }
}

fn main() {
    let is = iced::Settings::default();
    ZalgoCodecGui::run(is).unwrap()
}