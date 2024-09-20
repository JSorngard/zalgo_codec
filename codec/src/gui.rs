use cli_clipboard::set_contents;
use iced::{
    self,
    widget::{button::Button, column, row, text::Text, text_input::TextInput, Space},
    window::Settings,
    Element, Length, Size, Task,
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

#[derive(Debug, Default)]
struct ZalgoCodecGui {
    input_field: String,
    output_field: String,
    notifications: Vec<String>,
}

fn update(state: &mut ZalgoCodecGui, message: ToplevelMessage) -> Task<ToplevelMessage> {
    match message {
        ToplevelMessage::CodecFinished(result) => {
            state.output_field = result;
            Task::none()
        }
        ToplevelMessage::PushNotification(notification) => {
            state.notifications.push(notification);
            Task::perform(
                async { std::thread::sleep(std::time::Duration::from_secs(5)) },
                |_| ToplevelMessage::TimerFinised(TimedAction::PopNotification),
            )
        }
        ToplevelMessage::TimerFinised(action) => {
            match action {
                TimedAction::PopNotification => state.notifications.pop(),
            };
            Task::none()
        }
        ToplevelMessage::User(action) => match action {
            UserAction::EditedInputText(text) => {
                state.input_field = text;
                Task::none()
            }
            UserAction::Pressed(GuiButton::Encode) => {
                let input = state.input_field.clone();
                Task::perform(async move { zalgo_encode(&input) }, |res| match res {
                    Ok(encoded) => ToplevelMessage::CodecFinished(encoded),
                    Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                })
            }
            UserAction::Pressed(GuiButton::Decode) => {
                let input = state.input_field.clone();
                if input.is_empty() {
                    Task::perform(
                        async { String::from("the input string was empty") },
                        ToplevelMessage::PushNotification,
                    )
                } else {
                    Task::perform(async move { zalgo_decode(&input) }, |res| match res {
                        Ok(decoded) => ToplevelMessage::CodecFinished(decoded),
                        Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                    })
                }
            }
            UserAction::Pressed(GuiButton::Wrap) => {
                let input = state.input_field.clone();
                Task::perform(async move { zalgo_wrap_python(&input) }, |res| match res {
                    Ok(wrapped) => ToplevelMessage::CodecFinished(wrapped),
                    Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                })
            }
            UserAction::Pressed(GuiButton::Unwrap) => {
                let mut chars = state.input_field.chars();
                for _ in 0..3 {
                    chars.next();
                }
                for _ in 0..88 {
                    chars.next_back();
                }
                let encoded: String = chars.collect();
                Task::perform(async move { zalgo_decode(&encoded) }, |res| match res {
                    Ok(unwrapped) => ToplevelMessage::CodecFinished(unwrapped),
                    Err(e) => ToplevelMessage::PushNotification(e.to_string()),
                })
            }
            UserAction::Pressed(GuiButton::Copy) => {
                if let Err(e) = set_contents(state.output_field.clone()) {
                    let s = e.to_string();
                    Task::future(async { ToplevelMessage::PushNotification(s) })
                } else {
                    Task::none()
                }
            }
            UserAction::Pressed(GuiButton::SaveAs) => {
                if let Some(path) = FileDialog::new().set_file_name("zalgo.txt").save_file() {
                    if let Err(e) = std::fs::write(path, &state.output_field) {
                        let s = e.to_string();
                        Task::future(async { ToplevelMessage::PushNotification(s) })
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
        },
    }
}

fn view(state: &ZalgoCodecGui) -> Element<ToplevelMessage> {
    const BUTTON_WIDTH: f32 = 80.0;
    const SPACE_HEIGHT: f32 = 10.0;
    column![
        row![
            column![
                Space::with_height(Length::Fixed(SPACE_HEIGHT)),
                TextInput::new("Type or paste text here!", &state.input_field)
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
                Text::new(&state.output_field),
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
        Space::with_height(SPACE_HEIGHT),
        Text::new(
            state
                .notifications
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

/// Runs the GUI version of the application and then exits.
pub fn run_gui() -> ! {
    match iced::application("zalgo codec GUI", update, view)
        .window(Settings {
            size: Size {
                width: 500.0,
                height: 300.0,
            },
            ..Default::default()
        })
        .run()
    {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("GUI failed with error: {e}");
            std::process::exit(1);
        }
    }
}
