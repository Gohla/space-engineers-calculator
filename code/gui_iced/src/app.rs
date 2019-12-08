use std::fmt::Debug;
use std::sync::Mutex;

use iced::{Application, Color, Command, Element, Row, scrollable, Scrollable, Text};

use lazy_static::lazy_static;
use secalc_core::grid::GridCalculator;

use crate::data_bind::{DataBind, DataBindMessage};

lazy_static! {
  static ref CALC: Mutex<GridCalculator> = Mutex::new(GridCalculator::default());
}

pub struct App {
  input_scrollable: scrollable::State,
  result_scrollable: scrollable::State,
  gravity_multiplier: DataBind<f64>,
}

impl Default for App {
  fn default() -> Self {
    Self {
      input_scrollable: scrollable::State::default(),
      result_scrollable: scrollable::State::default(),
      gravity_multiplier: DataBind::new("Gravity Multiplier", 1.0, "1.0", Box::new(|v| CALC.lock().unwrap().gravity_multiplier = v)),
    }
  }
}

#[derive(Clone, Debug)]
pub enum Message {
  GravityMultiplier(DataBindMessage)
}

impl Application for App {
  type Message = Message;

  fn new() -> (Self, Command<Message>) {
    (Self::default(), Command::none())
  }

  fn title(&self) -> String {
    String::from("Space Engineers Calculator")
  }

  fn update(&mut self, message: Message) -> Command<Message> {
    match message {
      Message::GravityMultiplier(m) => self.gravity_multiplier.update(m),
    }
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    let input: Element<_> = Scrollable::new(&mut self.input_scrollable)
      .push(Text::new("Options"))
      .push(self.gravity_multiplier.view(Message::GravityMultiplier))
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
