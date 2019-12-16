use std::fmt::Debug;
use std::str::FromStr;

use iced::{Color, Element, Length, Row, Text, text_input, TextInput, VerticalAlignment};

pub struct DataBind<T> {
  label: String,
  label_width: Length,
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

impl<T: Copy + FromStr> DataBind<T> {
  pub fn new<L: Into<String>, P: Into<String>, U: Into<String>>(label: L, label_width: Length, input_default: T, input_placeholder: P, input_width: Length, unit: U) -> Self {
    let label = label.into();
    let input_placeholder = input_placeholder.into();
    let unit = unit.into();
    let value = String::new();
    let error = false;
    let state = text_input::State::default();
    Self { label, label_width, input_default, input_placeholder, input_width, unit, value, error, state }
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

  pub fn view(&mut self) -> Element<DataBindMessage> {
    let label = {
      let label = Text::new(&self.label)
        .width(self.label_width)
        .vertical_alignment(VerticalAlignment::Center);
      if self.error {
        label.color(Color::from_rgb(0.8, 0.0, 0.0))
      } else {
        label
      }
    };
    let input = TextInput::new(&mut self.state, &self.input_placeholder, &self.value, DataBindMessage)
      .padding(1)
      .width(self.input_width)
      ;
    let unit = Text::new(&self.unit)
      .width(Length::Shrink)
      ;

    Row::new()
      .spacing(2)
      .padding(1)
      .push(label)
      .push(input)
      .push(unit)
      .into()
  }
}