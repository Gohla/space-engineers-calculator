use std::ops::Deref;

use iced::{Alignment, button, Element};

use crate::storage::Storage;
use crate::view::{button, col, h1, h3, row};

#[derive(Debug)]
pub struct Page {
  load_states: Vec<(String, button::State)>,
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
  pub fn new(calculator_storage: &Storage) -> Self {
    let load_states = calculator_storage.iter_saved_calculators().map(|(name, _)| (name.clone(), button::State::default())).collect();
    Self { load_states, cancel_button_state: button::State::default() }
  }

  pub fn update(&mut self, message: Message) -> Option<Action> {
    match message {
      Message::Load(name) => Some(Action::Load(name)),
      Message::Cancel => Some(Action::Cancel),
    }
  }

  pub fn view(&mut self) -> Element<Message> {
    let mut column = col()
      .spacing(10)
      .padding(10)
      .push(row()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(h1("Load"))
        .push(button(&mut self.cancel_button_state, "Cancel").on_press(Message::Cancel))
      )
      ;
    for (name, button_state) in &mut self.load_states {
      column = column.push(row()
        .spacing(10)
        .push(h3(name.deref()))
        .push(button(button_state, "Load").on_press(Message::Load(name.clone())))
      )
    }
    column.into()
  }
}
