use iced::{Align, button, Element};

use crate::view::{button, col, h1, row};

#[derive(Default, Debug)]
pub struct Page {
  cancel_button_state: button::State,
  discard_button_state: button::State,
}

#[derive(Clone, Debug)]
pub enum Message {
  Cancel,
  Discard,
}

#[derive(Debug)]
pub enum Action {
  Cancel,
  Discard,
}

impl Page {
  pub fn new() -> Self { Self::default() }

  pub fn update(&mut self, message: Message) -> Action {
    match message {
      Message::Cancel => Action::Cancel,
      Message::Discard => Action::Discard,
    }
  }

  pub fn view(&mut self) -> Element<Message> {
    col()
      .padding(10)
      .spacing(10)
      .push(row()
        .spacing(10)
        .align_items(Align::End)
        .push(h1("Unsaved changes - discard?"))
      )
      .push(row()
        .spacing(10)
        .push(button(&mut self.cancel_button_state, "Cancel").on_press(Message::Cancel))
        .push(button(&mut self.discard_button_state, "Discard unsaved changes")/*.background(danger_color())*/.on_press(Message::Discard))
      )
      .into()
  }
}
