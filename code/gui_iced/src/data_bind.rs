use std::fmt::Debug;
use std::str::FromStr;

use iced::{Alignment, Element, Length, text_input};

use crate::view::{danger_color, foreground_color, lbl, row, text_input};

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
  pub fn new<P: Into<String>, U: Into<String>, V: Into<String>>(input_default: T, input_placeholder: P, input_width: Length, unit: U, value: V) -> Self {
    let input_placeholder = input_placeholder.into();
    let unit = unit.into();
    let (value, error) = Self::value_and_error(value.into(), input_default);
    let state = text_input::State::default();
    Self { input_default, input_placeholder, input_width, unit, value, error, state }
  }

  fn value_and_error(val: String, default: T) -> (String, bool) {
    if let Ok(parsed) = T::from_str(&val) {
      if parsed == default {
        ("".to_owned(), false)
      } else {
        (val, false)
      }
    } else {
      (val, true)
    }
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
    let (value, error) = Self::value_and_error(val, self.input_default);
    self.value = value;
    self.error = error;
  }

  pub fn view(&mut self) -> Element<DataBindMessage> {
    let input = text_input(self.input_width, &mut self.state, &self.input_placeholder, &self.value, DataBindMessage)
      .padding(1)
      ;
    let unit = lbl(&self.unit)
      .color(if self.error { danger_color() } else { foreground_color() })
      ;
    row()
      .spacing(2)
      .padding(1)
      .align_items(Alignment::Center)
      .push(input)
      .push(unit)
      .into()
  }
}
