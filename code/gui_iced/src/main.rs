use iced::{Application, Settings};

pub mod data_bind;
pub mod app;

fn main() {
  app::App::run(Settings::default())
}
