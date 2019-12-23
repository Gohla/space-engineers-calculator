use iced::{Align, button, Element, text_input, TextInput};

use crate::view::{button, col, h1, row};

#[derive(Debug)]
pub struct Page {
  name: String,
  name_input_state: text_input::State,
  save_button_state: button::State,
  cancel_button_state: button::State,
}

#[derive(Clone, Debug)]
pub enum Message {
  SetName(String),
  Save,
  Cancel,
}

#[derive(Debug)]
pub enum Action {
  Save(String),
  Cancel,
}

impl Page {
  pub fn new(name: Option<String>) -> Self {
    Self {
      name: name.unwrap_or("".to_string()),
      name_input_state: Default::default(),
      save_button_state: Default::default(),
      cancel_button_state: Default::default()
    }
  }

  pub fn update(&mut self, message: Message) -> Option<Action> {
    match message {
      Message::SetName(name) => {
        self.name = name;
        None
      },
      Message::Save => Some(Action::Save(self.name.clone())),
      Message::Cancel => Some(Action::Cancel),
    }
  }

  pub fn view(&mut self) -> Element<Message> {
    col()
      .padding(10)
      .push(row()
        .spacing(10)
        .align_items(Align::Center)
        .push(h1("Save as"))
      )
      .push(row()
        .spacing(20)
        .push(TextInput::new(&mut self.name_input_state, "", &self.name, Message::SetName))
        .push(button(&mut self.save_button_state, "Save").on_press(Message::Save))
        .push(button(&mut self.cancel_button_state, "Cancel").on_press(Message::Cancel))
      )
      .into()
  }
}
