use iced::{Application, Settings};

mod data_bind;
mod app;

fn main() {
  app::App::run(Settings::default())
}
