#![feature(try_trait)]
#![feature(clamp)]

#![windows_subsystem = "windows"] // Removes console window on windows.

use secalc_core::data::Data;

pub mod gui;

fn main() {
  let bytes: &[u8] = include_bytes!("../../../data/data.json");
  let data = Data::from_json(bytes).expect("Cannot read data");
  gui::run(data);
}
