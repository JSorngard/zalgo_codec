use cli_clipboard::set_contents;
use iced::{
    self, executor,
    widget::{button::Button, column, row, text::Text, text_input::TextInput, Space},
    Application, Command, Element, Length, Theme,
};
use rfd::FileDialog;
use zalgo_codec_common::{zalgo_decode, zalgo_encode, zalgo_wrap_python};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiButton {
    Encode,
    Decode,
    Copy,
    SaveAs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UserAction {
    EditedInputText(String),
    Pressed(GuiButton),
}

#[derive(Debug, Clone)]
enum ToplevelMessage {
    CodecFinished(String),
    User(UserAction),
}

#[derive(Debug)]
struct ZalgoCodecGui {
    input_field: String,
    output_field: String,
    error_messages: Vec<String>,
    working: bool,
}

impl ZalgoCodecGui {
    fn new() -> Self {
        Self {
            input_field: String::default(),
            output_field: String::default(),
            error_messages: Vec::default(),
            working: false,
        }
    }
}

impl Application for ZalgoCodecGui {
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;
    type Message = ToplevelMessage;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (ZalgoCodecGui::new(), Command::none())
    }

    fn title(&self) -> String {
        String::from("zalgo codec GUI")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Self::Message::CodecFinished(result) => {
                self.working = false;
                self.output_field = result;
                Command::none()
            }
            Self::Message::User(action) => match action {
                UserAction::EditedInputText(text) => {
                    self.input_field = text;
                    Command::none()
                }
                UserAction::Pressed(GuiButton::Encode) => {
                    let input = self.input_field.clone();
                    self.working = true;
                    Command::perform(async move { zalgo_encode(&input) }, |res| {
                        ToplevelMessage::CodecFinished(res.map_or_else(|e| e.to_string(), |ok| ok))
                    })
                }
                UserAction::Pressed(GuiButton::Decode) => {
                    let input = self.input_field.clone();
                    if input.is_empty() {
                        Command::perform(
                            async { String::from("the input string was empty") },
                            ToplevelMessage::CodecFinished,
                        )
                    } else {
                        self.working = true;
                        Command::perform(async move { zalgo_decode(&input) }, |res| {
                            ToplevelMessage::CodecFinished(
                                res.map_or_else(|e| e.to_string(), |ok| ok),
                            )
                        })
                    }
                }
                UserAction::Pressed(GuiButton::Copy) => {
                    if let Err(e) = set_contents(self.output_field.clone()) {
                        self.error_messages.push(e.to_string());
                    }
                    Command::none()
                }
                UserAction::Pressed(GuiButton::SaveAs) => {
                    if let Some(path) = FileDialog::new().set_file_name("zalgo.txt").save_file() {
                        if let Err(e) = std::fs::write(path, &self.output_field) {
                            self.error_messages.push(e.to_string());
                        }
                    }
                    Command::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        column![
            row![
                column![
                    TextInput::new("Type or paste text here!", &self.input_field)
                        .on_input(|s| ToplevelMessage::User(UserAction::EditedInputText(s)))
                        .on_paste(|s| ToplevelMessage::User(UserAction::EditedInputText(s))),
                    if self.working {
                        Button::new("Encode")
                    } else {
                        Button::new("Encode").on_press(ToplevelMessage::User(UserAction::Pressed(
                            GuiButton::Encode,
                        )))
                    },
                    if self.working {
                        Button::new("Decode")
                    } else {
                        Button::new("Decode").on_press(ToplevelMessage::User(UserAction::Pressed(
                            GuiButton::Decode,
                        )))
                    },
                ]
                .width(Length::FillPortion(3)),
                Space::with_width(Length::Fill),
                column![
                    Text::new(&self.output_field),
                    Button::new("Copy")
                        .on_press(ToplevelMessage::User(UserAction::Pressed(GuiButton::Copy))),
                    Button::new("Save as").on_press(ToplevelMessage::User(UserAction::Pressed(
                        GuiButton::SaveAs
                    ))),
                ]
                .width(Length::FillPortion(3)),
            ]
            .width(Length::Fill),
            Text::new(
                self.error_messages
                    .iter()
                    .rev()
                    .fold(String::new(), |toast, msg| format!("{toast}\n{msg}"))
            )
            .width(Length::Fill),
        ]
        .width(Length::Fill)
        .into()
    }
}

pub fn run_gui() -> ! {
    match ZalgoCodecGui::run(iced::Settings {
        window: iced::window::Settings {
            size: (500, 200),
            ..Default::default()
        },
        ..Default::default()
    }) {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("GUI failed with error: {e}");
            std::process::exit(1);
        }
    }
}
