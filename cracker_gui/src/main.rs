mod tab;

use crate::tab::bedrock::bedrock_tab::{BdrkTab};
use crate::tab::controls::{ControlMenu, ControlMessage};
use iced::{keyboard, Application, Element, Settings, Theme};
use iced_native::{event, subscription, Command, Event, Subscription};

fn main() -> iced::Result {
    <State as Application>::run(Settings::default())
}

#[derive(Debug, Default)]
struct State {
    bedrock_menu: ControlMenu<BdrkTab>,
}

#[derive(Debug, Clone)]
enum Message {
    ControlMessage(ControlMessage),
    TabPressed { shift: bool },
}

impl Application for State {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        let bedrock_menu = ControlMenu::new();

        (State { bedrock_menu }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Bedrock cracker")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::TabPressed { shift } => {
                if shift {
                    iced::widget::focus_previous()
                } else {
                    iced::widget::focus_next()
                }
            }
            Message::ControlMessage(message) => self
                .bedrock_menu
                .update(message)
                .map(Message::ControlMessage),
        }
    }

    fn view(&self) -> Element<Message> {
        self.bedrock_menu.view().map(Message::ControlMessage)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let cracker_sub = self
            .bedrock_menu
            .subscription()
            .map(Message::ControlMessage);

        let tab_sub = subscription::events_with(|event, status| match (event, status) {
            (
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers,
                    ..
                }),
                event::Status::Ignored,
            ) => Some(Message::TabPressed {
                shift: modifiers.shift(),
            }),
            _ => None,
        });
        Subscription::batch(vec![cracker_sub, tab_sub])
    }
}
