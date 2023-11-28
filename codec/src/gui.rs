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
    Wrap,
    Unwrap,
    Copy,
    SaveAs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UserAction {
    EditedInputText(String),
    Pressed(GuiButton),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimedAction {
    PopNotification,
}

#[derive(Debug, Clone)]
enum ToplevelMessage {
    CodecFinished(String),
    TimerFinised(TimedAction),
    PushNotification(String),
    User(UserAction),
}

#[derive(Debug)]
struct ZalgoCodecGui {
    input_field: String,
    output_field: String,
    notifications: Vec<String>,
}

impl ZalgoCodecGui {
    fn new() -> Self {
        Self {
            input_field: String::default(),
            output_field: String::default(),
            notifications: Vec::default(),
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
                self.output_field = result;
                Command::none()
            }
            Self::Message::PushNotification(notification) => {
                self.notifications.push(notification);
                Command::perform(
                    async { std::thread::sleep(std::time::Duration::from_secs(5)) },
                    |_| ToplevelMessage::TimerFinised(TimedAction::PopNotification),
                )
            }
            Self::Message::TimerFinised(action) => {
                match action {
                    TimedAction::PopNotification => self.notifications.pop(),
                };
                Command::none()
            }
            Self::Message::User(action) => match action {
                UserAction::EditedInputText(text) => {
                    self.input_field = text;
                    Command::none()
                }
                UserAction::Pressed(GuiButton::Encode) => {
                    let input = self.input_field.clone();
                    Command::perform(async move { zalgo_encode(&input) }, |res| match res {
                        Ok(encoded) => ToplevelMessage::CodecFinished(encoded),
                        Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                    })
                }
                UserAction::Pressed(GuiButton::Decode) => {
                    let input = self.input_field.clone();
                    if input.is_empty() {
                        Command::perform(
                            async { String::from("the input string was empty") },
                            |s| ToplevelMessage::PushNotification(s),
                        )
                    } else {
                        Command::perform(async move { zalgo_decode(&input) }, |res| match res {
                            Ok(decoded) => ToplevelMessage::CodecFinished(decoded),
                            Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                        })
                    }
                }
                UserAction::Pressed(GuiButton::Wrap) => {
                    let input = self.input_field.clone();
                    Command::perform(async move { zalgo_wrap_python(&input) }, |res| match res {
                        Ok(wrapped) => ToplevelMessage::CodecFinished(wrapped),
                        Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                    })
                }
                UserAction::Pressed(GuiButton::Unwrap) => {
                    let mut chars = self.input_field.chars();
                    for _ in 0..3 {
                        chars.next();
                    }
                    for _ in 0..88 {
                        chars.next_back();
                    }
                    let encoded: String = chars.collect();
                    Command::perform(async move { zalgo_decode(&encoded) }, |res| match res {
                        Ok(unwrapped) => ToplevelMessage::CodecFinished(unwrapped),
                        Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                    })
                }
                UserAction::Pressed(GuiButton::Copy) => {
                    if let Err(e) = set_contents(self.output_field.clone()) {
                        let s = e.to_string();
                        Command::perform(async {}, |_| ToplevelMessage::PushNotification(s))
                    } else {
                        Command::none()
                    }
                }
                UserAction::Pressed(GuiButton::SaveAs) => {
                    if let Some(path) = FileDialog::new().set_file_name("zalgo.txt").save_file() {
                        if let Err(e) = std::fs::write(path, &self.output_field) {
                            let s = e.to_string();
                            Command::perform(async {}, |_| ToplevelMessage::PushNotification(s))
                        } else {
                            Command::none()
                        }
                    } else {
                        Command::none()
                    }
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        const BUTTON_WIDTH: f32 = 70.0;
        const SPACE_HEIGHT: f32 = 10.0;
        column![
            row![
                column![
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    TextInput::new("Type or paste text here!", &self.input_field)
                        .on_input(|s| ToplevelMessage::User(UserAction::EditedInputText(s)))
                        .on_paste(|s| ToplevelMessage::User(UserAction::EditedInputText(s))),
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    Button::new("Encode")
                        .on_press(ToplevelMessage::User(UserAction::Pressed(
                            GuiButton::Encode,
                        )))
                        .width(Length::Fixed(BUTTON_WIDTH)),
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    Button::new("Decode")
                        .on_press(ToplevelMessage::User(UserAction::Pressed(
                            GuiButton::Decode,
                        )))
                        .width(Length::Fixed(BUTTON_WIDTH)),
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    Button::new("Wrap")
                        .on_press(ToplevelMessage::User(UserAction::Pressed(GuiButton::Wrap)))
                        .width(Length::Fixed(BUTTON_WIDTH)),
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    Button::new("Unwrap")
                        .on_press(ToplevelMessage::User(UserAction::Pressed(
                            GuiButton::Unwrap
                        )))
                        .width(Length::Fixed(BUTTON_WIDTH)),
                ]
                .width(Length::FillPortion(3)),
                Space::with_width(Length::Fill),
                column![
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    Text::new(&self.output_field),
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    Button::new("Copy")
                        .on_press(ToplevelMessage::User(UserAction::Pressed(GuiButton::Copy)))
                        .width(Length::Fixed(BUTTON_WIDTH)),
                    Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                    Button::new("Save as")
                        .on_press(ToplevelMessage::User(UserAction::Pressed(
                            GuiButton::SaveAs
                        )))
                        .width(Length::Fixed(BUTTON_WIDTH)),
                ]
                .width(Length::FillPortion(3)),
            ]
            .width(Length::Fill),
            Text::new(
                self.notifications
                    .iter()
                    .rev()
                    .fold(String::new(), |toast, msg| format!("{toast}\n{msg}"))
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .width(Length::Fill)
        .into()
    }
}

/// Runs the GUI version of the application and then exits.
pub fn run_gui() -> ! {
    match ZalgoCodecGui::run(iced::Settings {
        window: iced::window::Settings {
            size: (500, 250),
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
