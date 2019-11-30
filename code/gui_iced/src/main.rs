use iced::{Column, Element, Sandbox, Settings};

pub fn main() {
  Application::run(Settings::default())
}

#[derive(Default)]
struct Application {}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Sandbox for Application {
  type Message = Message;

  fn new() -> Self {
    Self::default()
  }

  fn title(&self) -> String {
    String::from("Space Engineers Calculator")
  }

  fn update(&mut self, _message: Message) {}

  fn view(&mut self) -> Element<Message> {
    Column::new().into()
  }
}
