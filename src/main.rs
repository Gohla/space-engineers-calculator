#![feature(try_trait)]
#![feature(clamp)]

use std::path::Path;

use crate::data::Data;

pub mod calc;
pub mod data;
pub mod gui;
pub mod error;

fn main() {
  let data = Data::from_se_dir(Path::new(r#"C:\Games\Steam\steamapps\common\SpaceEngineers\"#)).expect("Cannot read data");
  gui::run(data);
}
