#![allow(dead_code, unused_imports, unused_variables)]

use iced::{Column, Element, Row, Sandbox, Settings, Text, text_input, TextInput};

pub fn main() {
  State::run(Settings::default())
}

#[derive(Default)]
struct State {}

#[derive(Debug, Clone)]
enum Message {}

impl Sandbox for State {
  type Message = Message;

  fn new() -> Self {
    Self::default()
  }

  fn title(&self) -> String {
    String::from("Space Engineers Calculator")
  }

  fn update(&mut self, message: Message) {}

  fn view(&mut self) -> Element<Message> {
    Column::new().into()
  }
}
