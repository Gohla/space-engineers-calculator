#![allow(dead_code, unused_imports, unused_variables)]

use std::str::FromStr;

use iced::{Application, Color, Column, Command, Container, Element, Length, Row, scrollable, Scrollable, Settings, Text, text_input, TextInput};

use secalc_core::grid::GridCalculator;

#[derive(Debug)]
struct SECalc {
  calc: GridCalculator,
  input_scrollable: scrollable::State,
  result_scrollable: scrollable::State,
  gravity_multiplier: DataBind<f64>,
}

impl Default for SECalc {
  fn default() -> Self {
    Self {
      calc: GridCalculator::default(),
      input_scrollable: scrollable::State::default(),
      result_scrollable: scrollable::State::default(),
      gravity_multiplier: DataBind::new("Gravity Multiplier", 1.0, "1.0"),
    }
  }
}

#[derive(Clone, Debug)]
enum Message {
  InputMessage(DataBindMessage)
}

impl Application for SECalc {
  type Message = Message;

  fn new() -> (Self, Command<Self::Message>) {
    (Self::default(), Command::none())
  }

  fn title(&self) -> String {
    String::from("Space Engineers Calculator")
  }

  fn update(&mut self, message: Message) -> Command<Message> {
    match message {
      Message::InputMessage(m) => {
        self.gravity_multiplier.update(&mut self.calc.gravity_multiplier, m)
      },
    }
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    let input: Element<_> = Scrollable::new(&mut self.input_scrollable)
      .push(Text::new("Options"))
      .push(self.gravity_multiplier.view(&self.calc.gravity_multiplier))
      .push(Text::new("Storage"))
      .push(Text::new("Thrusters"))
      .push(Text::new("Power"))
      .push(Text::new("Hydrogen"))
      .into();
    let result: Element<_> = Scrollable::new(&mut self.result_scrollable)
      .push(Text::new("Mass"))
      .push(Text::new("Volume"))
      .push(Text::new("Items"))
      .push(Text::new("Force & Acceleration"))
      .push(Text::new("Power"))
      .push(Text::new("Hydrogen"))
      .into();

    let content: Element<_> = Row::new()
      .spacing(20)
      .padding(20)
      .push(input)
      .push(result)
      .into();
    content.explain(Color::BLACK)
  }
}

#[derive(Debug)]
struct DataBind<T> {
  label: String,
  default: T,
  placeholder: String,
  value: String,
  state: text_input::State,
}

#[derive(Clone, Debug)]
struct DataBindMessage(String);

impl<T: Copy + FromStr> DataBind<T> {
  fn new<L: Into<String>, P: Into<String>>(label: L, default: T, placeholder: P) -> Self {
    let label = label.into();
    let placeholder = placeholder.into();
    let value = String::new();
    let state = text_input::State::default();
    Self { label, default, placeholder, value, state }
  }

  fn update(&mut self, data: &mut T, message: DataBindMessage) {
    if let Ok(d) = T::from_str(&message.0) {
      *data = d
    } else {
      *data = self.default;
    }
    self.value = message.0;
  }

  fn view(&mut self, data: &T) -> Element<Message> {
    Row::new()
      .push(Text::new(&self.label))
      .push(
        TextInput::new(&mut self.state, &self.placeholder, &self.value, |s| Message::InputMessage(DataBindMessage(s)))
          .width(Length::Units(100))
      )
      .into()
  }
}

pub fn main() {
  SECalc::run(Settings::default())
}
