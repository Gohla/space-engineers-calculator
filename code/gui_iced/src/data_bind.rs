use std::fmt::Debug;
use std::str::FromStr;

use iced::{Color, Element, Length, Row, Text, text_input, TextInput, VerticalAlignment};

pub struct DataBind<T> {
  label: String,
  default: T,
  placeholder: String,
  value: String,
  error: bool,
  state: text_input::State,
}

#[derive(Clone, Debug)]
pub struct DataBindMessage(String);

impl<T: Copy + FromStr> DataBind<T> {
  pub fn new<L: Into<String>, P: Into<String>>(label: L, default: T, placeholder: P) -> Self {
    let label = label.into();
    let placeholder = placeholder.into();
    let value = String::new();
    let error = false;
    let state = text_input::State::default();
    Self { label, default, placeholder, value, error, state }
  }

  pub fn update(&mut self, message: DataBindMessage, val: &mut T) {
    let text = message.0;
    let (v, error) = match (text.is_empty(), T::from_str(&text)) {
      (true, _) => (self.default, false),
      (_, Err(_)) => (self.default, true),
      (false, Ok(v)) => (v, false)
    };
    *val = v;
    self.error = error;
    self.value = text;
  }

  pub fn view<M: 'static + Clone + Debug>(&mut self, on_change: impl 'static + Fn(DataBindMessage) -> M) -> Element<M> {
    let text = Text::new(&self.label)
      .width(Length::Shrink)
      .vertical_alignment(VerticalAlignment::Center);
    let text = if self.error {
      text.color(Color::from_rgb(0.8, 0.0, 0.0))
    } else {
      text
    };
    Row::new()
      .spacing(4)
      .padding(4)
      .push(text)
      .push(
        TextInput::new(&mut self.state, &self.placeholder, &self.value, move |s| on_change(DataBindMessage(s)))
          .width(Length::Units(50))
          .padding(1)
      )
      .into()
  }
}