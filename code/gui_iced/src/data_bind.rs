use std::fmt::Debug;
use std::str::FromStr;

use iced::{Color, Element, Length, Row, Text, text_input, TextInput, VerticalAlignment};

use crate::util::Msg;

pub struct DataBind<T> {
  label: String,
  label_length: Length,

  input_default: T,
  input_placeholder: String,
  input_length: Length,

  value: String,
  error: bool,
  state: text_input::State,
}

#[derive(Clone, Debug)]
pub struct DataBindMessage(String);

impl<T: Copy + FromStr> DataBind<T> {
  pub fn new<L: Into<String>, P: Into<String>>(label: L, label_length: Length, input_default: T, input_placeholder: P, input_length: Length) -> Self {
    let label = label.into();
    let input_placeholder = input_placeholder.into();
    let value = String::new();
    let error = false;
    let state = text_input::State::default();
    Self { label, label_length, input_default, input_placeholder, input_length, value, error, state }
  }

  pub fn update(&mut self, message: DataBindMessage, val: &mut T) {
    let text = message.0;
    let (v, error) = match (text.is_empty(), T::from_str(&text)) {
      (true, _) => (self.input_default, false),
      (_, Err(_)) => (self.input_default, true),
      (false, Ok(v)) => (v, false)
    };
    *val = v;
    self.error = error;
    self.value = text;
  }

  pub fn view<M: Msg>(&mut self, on_change: impl 'static + Fn(DataBindMessage) -> M) -> Element<M> {
    let text = Text::new(&self.label)
      .width(self.label_length)
      .vertical_alignment(VerticalAlignment::Center);
    let text = if self.error {
      text.color(Color::from_rgb(0.8, 0.0, 0.0))
    } else {
      text
    };
    Row::new()
      .spacing(2)
      .padding(1)
      .push(text)
      .push(
        TextInput::new(&mut self.state, &self.input_placeholder, &self.value, move |s| on_change(DataBindMessage(s)))
          .width(self.input_length)
          .padding(1)
      )
      .into()
  }
}