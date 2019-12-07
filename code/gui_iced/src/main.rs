#![allow(dead_code, unused_imports, unused_variables)]

use std::str::FromStr;

use iced::{Application, Color, Column, Command, Container, Element, Length, Row, scrollable, Scrollable, Settings, Text, text_input, TextInput};

use secalc_core::grid::GridCalculator;

#[derive(Debug)]
struct SECalc {
  calc: GridCalculator,
  input_scrollable: scrollable::State,
  result_scrollable: scrollable::State,
  gravity_multiplier: Input,
}

impl Default for SECalc {
  fn default() -> Self {
    Self {
      calc: GridCalculator::default(),
      input_scrollable: scrollable::State::default(),
      result_scrollable: scrollable::State::default(),
      gravity_multiplier: Input::new("Gravity Multiplier", "1.0"),
    }
  }
}

#[derive(Clone, Debug)]
enum Message {
  InputMessage(InputMessage)
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
        self.gravity_multiplier.update(&mut self.calc.gravity_multiplier, 1.0, m)
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
struct Input {
  label: String,
  placeholder: String,
  state: text_input::State,
}

#[derive(Clone, Debug)]
struct InputMessage(String);

impl Input {
  fn new<S1: Into<String>, S2: Into<String>>(label: S1, placeholder: S2) -> Self {
    let label = label.into();
    let placeholder = placeholder.into();
    let state = text_input::State::default();
    Self { label, placeholder, state }
  }

  fn update<T: FromStr>(&mut self, data: &mut T, default: T, message: InputMessage) {
    if let Ok(d) = T::from_str(&message.0) {
      *data = d
    } else {
      *data = default;
    }
  }

  fn view<T: ToString>(&mut self, data: &T) -> Element<Message> {
    Row::new()
      .push(Text::new(&self.label))
      .push(
        TextInput::new(&mut self.state, &self.placeholder, &T::to_string(data), |s| Message::InputMessage(InputMessage(s)))
          .width(Length::Units(100))
      )
      .into()
  }
}

pub fn main() {
  SECalc::run(Settings::default())
}
