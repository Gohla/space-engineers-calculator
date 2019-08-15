#![feature(try_trait)]

use std::path::Path;

pub mod xml;
pub mod blocks;
pub mod components;
pub mod gas_properties;
pub mod localization;

fn main() {
  let data_dir = Path::new(r#"C:\Games\Steam\steamapps\common\SpaceEngineers\Content\Data\"#);

  let localization = localization::Localization::from_data(data_dir.join(r#"Localization\MyTexts.resx"#));

  let gas_properties = gas_properties::GasProperties::from_data(data_dir.join(r#"GasProperties.sbc"#));
  dbg!(&gas_properties);
  let components = components::Components::from_data(data_dir.join(r#"Components.sbc"#));
  dbg!(&components);

  let blocks = blocks::Blocks::from_data(data_dir.join(r#"CubeBlocks.sbc"#), data_dir.join(r#"EntityComponents.sbc"#));

  for thruster in &blocks.thrusters {
    dbg!(thruster);
    dbg!(thruster.name(&localization));
    dbg!(thruster.mass(&components));
  }

  for battery in &blocks.batteries {
    dbg!(battery);
    dbg!(battery.name(&localization));
    dbg!(battery.mass(&components));
  }

  for hydrogen_engine in &blocks.hydrogen_engines {
    dbg!(hydrogen_engine);
    dbg!(hydrogen_engine.name(&localization));
    dbg!(hydrogen_engine.mass(&components));
  }

  for reactor in &blocks.reactors {
    dbg!(reactor);
    dbg!(reactor.name(&localization));
    dbg!(reactor.mass(&components));
  }

  for generator in &blocks.generators {
    dbg!(generator);
    dbg!(generator.name(&localization));
    dbg!(generator.mass(&components));
  }

  for hydrogen_tank in &blocks.hydrogen_tanks {
    dbg!(hydrogen_tank);
    dbg!(hydrogen_tank.name(&localization));
    dbg!(hydrogen_tank.mass(&components));
  }

  for container in &blocks.containers {
    dbg!(container);
    dbg!(container.name(&localization));
    dbg!(container.mass(&components));
  }

  for cockpit in &blocks.cockpits {
    dbg!(cockpit);
    dbg!(cockpit.name(&localization));
    dbg!(cockpit.mass(&components));
  }
}
