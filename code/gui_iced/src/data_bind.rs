use std::fmt::Debug;
use std::str::FromStr;

use iced::{Color, Element, Length, Row, Text, text_input, TextInput};

pub struct DataBind<T> {
  label: String,
  default: T,
  placeholder: String,
  update: Box<dyn Fn(T) -> ()>,
  value: String,
  error: bool,
  state: text_input::State,
}

#[derive(Clone, Debug)]
pub struct DataBindMessage(String);

impl<T: Copy + FromStr> DataBind<T> {
  pub fn new<L: Into<String>, P: Into<String>>(label: L, default: T, placeholder: P, update: Box<dyn Fn(T) -> ()>) -> Self {
    let label = label.into();
    let placeholder = placeholder.into();
    let value = String::new();
    let error = false;
    let state = text_input::State::default();
    Self { label, default, placeholder, update, value, error, state }
  }

  pub fn update(&mut self, message: DataBindMessage) {
    if message.0.is_empty() {
      (self.update)(self.default);
      self.error = false;
    } else {
      if let Ok(v) = T::from_str(&message.0) {
        (self.update)(v);
        self.error = false;
      } else {
        (self.update)(self.default);
        self.error = true;
      }
    }
    self.value = message.0;
  }

  pub fn view<M: 'static + Clone + Debug>(&mut self, on_change: impl 'static + Fn(DataBindMessage) -> M) -> Element<M> {
    let text = Text::new(&self.label);
    let text = if self.error {
      text.color(Color::from_rgb(0.8, 0.0, 0.0))
    } else {
      text
    };
    Row::new()
      .push(text)
      .push(
        TextInput::new(&mut self.state, &self.placeholder, &self.value, move |s| on_change(DataBindMessage(s)))
          .width(Length::Units(100))
      )
      .into()
  }
}