use iced::{Align, button, Element};

use crate::view::{button, col, h1, row};

#[derive(Default, Debug)]
pub struct Page {
  load_button_state: button::State,
  cancel_button_state: button::State,
}

#[derive(Clone, Debug)]
pub enum Message {
  Load(String),
  Cancel,
}

#[derive(Debug)]
pub enum Action {
  Load(String),
  Cancel,
}

impl Page {
  pub fn new() -> Self { Self::default() }

  pub fn update(&mut self, message: Message) -> Option<Action> {
    match message {
      Message::Load(name) => Some(Action::Load(name)),
      Message::Cancel => Some(Action::Cancel),
    }
  }

  pub fn view(&mut self) -> Element<Message> {
    col()
      .padding(10)
      .push(row()
        .spacing(10)
        .align_items(Align::Center)
        .push(h1("Load"))
      )
      .push(row()
        .spacing(20)
        .push(button(&mut self.load_button_state, "Load").on_press(Message::Load("".to_string())))
        .push(button(&mut self.cancel_button_state, "Cancel").on_press(Message::Cancel))
      )
      .into()
  }
}
