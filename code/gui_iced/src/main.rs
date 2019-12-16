use iced::{Application, Settings};
use iced::settings::Window;

pub mod app;
pub mod data_bind;
pub mod widget;
pub mod option_input;
pub mod block_input;

fn main() {
  app::App::run(Settings {
    window: Window {
      size: (1300, 1000),
      ..Window::default()
    }
  })
}
