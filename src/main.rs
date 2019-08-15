#![feature(try_trait)]

use std::path::Path;

pub mod xml;
pub mod blocks;
pub mod components;
pub mod gas_properties;
pub mod localization;

fn main() {
  let data_dir = Path::new(r#"C:\Games\Steam\steamapps\common\SpaceEngineers\Content\Data\"#);
  let gas_properties = gas_properties::GasProperties::from_data(data_dir.join(r#"GasProperties.sbc"#));
  dbg!(gas_properties);
  let components = components::Components::from_data(data_dir.join(r#"Components.sbc"#));
  dbg!(components);
  let blocks = blocks::Blocks::from_data(data_dir.join(r#"CubeBlocks.sbc"#));
  dbg!(blocks);
  let localization = localization::Localization::from_data(data_dir.join(r#"Localization\MyTexts.resx"#));
  dbg!(localization);
}
