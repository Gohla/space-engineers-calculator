#![feature(try_trait)]
#![feature(clamp)]

use std::path::Path;
use std::rc::Rc;

use crate::data::Data;

pub mod calc;
pub mod data;
pub mod gui;

fn main() {
  let data = Rc::new(Data::from_se_dir(Path::new(r#"C:\Games\Steam\steamapps\common\SpaceEngineers\"#)));
  gui::run(data.clone());
}
