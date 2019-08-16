#![feature(try_trait)]

use std::path::Path;

pub mod calc;
pub mod data;
pub mod gui;

fn main() {
  let data = data::Data::from_se_dir(Path::new(r#"C:\Games\Steam\steamapps\common\SpaceEngineers\"#));
  gui::run(&data);
}
