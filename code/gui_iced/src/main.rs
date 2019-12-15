#![feature(trait_alias)]

use iced::{Application, Settings};

pub mod data_bind;
pub mod app;
pub mod util;

fn main() {
  app::App::run(Settings::default())
}
