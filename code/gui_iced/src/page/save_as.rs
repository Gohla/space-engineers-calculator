use iced::{Alignment, button, Element, Length, text_input};

use crate::view::{button, col, danger_color, foreground_color, h1, lbl, row, text_input};

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
      Message::Save if !self.name.is_empty() => Some(Action::Save(self.name.clone())),
      Message::Cancel => Some(Action::Cancel),
      _ => None,
    }
  }

  pub fn view(&mut self) -> Element<Message> {
    col()
      .padding(10)
      .spacing(10)
      .push(row()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(h1("Save as"))
        .push(button(&mut self.cancel_button_state, "Cancel").on_press(Message::Cancel))
      )
      .push(row()
        .spacing(10)
        .push(lbl("Name: ").color(if self.name.is_empty() { danger_color() } else { foreground_color() }))
        .push(text_input(Length::Units(250), &mut self.name_input_state, "", &self.name, Message::SetName))
      )
      .push(row()
        .spacing(10)
        .push(button(&mut self.save_button_state, "Save").on_press(Message::Save))
      )
      .into()
  }
}
