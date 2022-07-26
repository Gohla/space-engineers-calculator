use iced::{Alignment, button, Element};

use crate::view::{button, col, h1, row};

#[derive(Debug)]
pub struct Page {
  name: String,
  cancel_button_state: button::State,
  overwrite_button_state: button::State,
}

#[derive(Clone, Debug)]
pub enum Message {
  Cancel,
  Overwrite,
}

#[derive(Debug)]
pub enum Action {
  Cancel,
  Overwrite(String),
}

impl Page {
  pub fn new(name: String) -> Self {
    Self {
      name,
      cancel_button_state: Default::default(),
      overwrite_button_state: Default::default()
    }
  }

  pub fn update(&mut self, message: Message) -> Action {
    match message {
      Message::Cancel => Action::Cancel,
      Message::Overwrite => Action::Overwrite(self.name.clone()),
    }
  }

  pub fn view(&mut self) -> Element<Message> {
    col()
      .padding(10)
      .spacing(10)
      .push(row()
        .spacing(10)
        .align_items(Alignment::End)
        .push(h1("Overwrite?"))
      )
      .push(row()
        .spacing(10)
        .push(button(&mut self.cancel_button_state, "Cancel").on_press(Message::Cancel))
        .push(button(&mut self.overwrite_button_state, "Overwrite")/*.background(danger_color())*/.on_press(Message::Overwrite))
      )
      .into()
  }
}
