use std::fmt::Debug;
use std::str::FromStr;

use iced::{Align, Color, Element, Length, text_input, TextInput};

use crate::view::{lbl, row, TXT_SIZE};

pub struct DataBind<T> {
  input_default: T,
  input_placeholder: String,
  input_width: Length,
  unit: String,
  value: String,
  error: bool,
  state: text_input::State,
}

#[derive(Clone, Debug)]
pub struct DataBindMessage(String);

impl<T: Copy + FromStr + PartialEq> DataBind<T> {
  pub fn new<P: Into<String>, U: Into<String>>(input_default: T, input_placeholder: P, input_width: Length, unit: U) -> Self {
    let input_placeholder = input_placeholder.into();
    let unit = unit.into();
    let value = String::new();
    let error = false;
    let state = text_input::State::default();
    Self { input_default, input_placeholder, input_width, unit, value, error, state }
  }

  pub fn update(&mut self, message: DataBindMessage, val: &mut T) {
    let DataBindMessage(text) = message;
    let (v, error) = match (text.is_empty(), T::from_str(&text)) {
      (true, _) => (self.input_default, false),
      (_, Err(_)) => (self.input_default, true),
      (false, Ok(v)) => (v, false)
    };
    *val = v;
    self.value = text;
    self.error = error;
  }

  pub fn reload(&mut self, val: String) {
    if let Ok(parsed) = T::from_str(&val) {
      if parsed == self.input_default {
        self.value = "".to_owned();
      } else {
        self.value = val;
      }
      self.error = false;
    } else {
      self.value = val;
      self.error = true;
    }
  }

  pub fn view(&mut self) -> Element<DataBindMessage> {
    let input = TextInput::new(&mut self.state, &self.input_placeholder, &self.value, DataBindMessage)
      .width(self.input_width)
      .padding(1)
      .size(TXT_SIZE)
      ;
    let unit = lbl(&self.unit)
      .color(if self.error { Color::from_rgb(0.8, 0.0, 0.0) } else { Color::BLACK })
      ;
    row()
      .spacing(2)
      .padding(1)
      .align_items(Align::Center)
      .push(input)
      .push(unit)
      .into()
  }
}
