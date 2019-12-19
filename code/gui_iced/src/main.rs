use iced::{Application, Settings};
use iced::settings::Window;

pub mod app;
pub mod data_bind;
pub mod view;
pub mod option_input;
pub mod block_input;
pub mod directional_block_input;

fn main() {
  app::App::run(Settings {
    window: Window {
      size: (2200, 1000),
      ..Window::default()
    },
    ..Settings::default()
  })
}

