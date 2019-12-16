use std::fmt::Debug;
use std::str::FromStr;

use iced::{Color, Element, Length, Row, Text, text_input, TextInput, VerticalAlignment};

pub struct DataBind<T> {
  label: Option<String>,
  label_width: Option<Length>,
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
    let label = Some(label.into());
    let label_width = Some(label_width);
    let input_placeholder = input_placeholder.into();
    let unit = unit.into();
    let value = String::new();
    let error = false;
    let state = text_input::State::default();
    Self { label, label_width, input_default, input_placeholder, input_width, unit, value, error, state }
  }

  pub fn new_without_label<P: Into<String>, U: Into<String>>(input_default: T, input_placeholder: P, input_width: Length, unit: U) -> Self {
    let input_placeholder = input_placeholder.into();
    let unit = unit.into();
    let value = String::new();
    let error = false;
    let state = text_input::State::default();
    Self { label: None, label_width: None, input_default, input_placeholder, input_width, unit, value, error, state }
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
    let label = self.label.as_ref().map(|l| {
      let label = Text::new(l)
        .width(self.label_width.unwrap_or(Length::Fill))
        .vertical_alignment(VerticalAlignment::Center);
      if self.error {
        label.color(Color::from_rgb(0.8, 0.0, 0.0))
      } else {
        label
      }
    });
    let input = TextInput::new(&mut self.state, &self.input_placeholder, &self.value, DataBindMessage)
      .padding(1)
      .width(self.input_width)
      ;
    let unit = Text::new(&self.unit)
      .width(Length::Shrink)
      ;

    let mut row = Row::new()
      .spacing(2)
      .padding(1)
      .width(Length::Shrink)
      ;
    if let Some(label) = label {
      row = row.push(label)
    }
    row
      .push(input)
      .push(unit)
      .into()
  }
}