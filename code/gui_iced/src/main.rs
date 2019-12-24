use iced::{Application, Settings};
use iced::settings::Window;

pub mod app;
pub mod view;
pub mod storage;
pub mod page;
pub mod data_bind;

fn main() {
  app::App::run(Settings {
    window: Window {
      size: (2300, 1000),
      ..Window::default()
    },
    ..Settings::default()
  })
}
