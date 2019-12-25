use iced::{Application, Settings};
use iced::settings::Window;

pub mod app;
pub mod view;
pub mod storage;
pub mod page;
pub mod data_bind;

fn main() {
  let log_level = log::Level::Error;

  #[cfg(not(target_arch = "wasm32"))] {
    simple_logger::init_with_level(log_level)
      .unwrap_or_else(|e| eprintln!("Could not initialize logger: {:?}", e));
  }

  #[cfg(target_arch = "wasm32")] {
    console_log::init_with_level(log_level)
      .unwrap_or_else(|e| eprintln!("Could not initialize logger: {:?}", e));
  }

  app::App::run(Settings {
    window: Window {
      size: (2300, 1000),
      ..Window::default()
    },
    ..Settings::default()
  })
}
