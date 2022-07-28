use std::collections::HashMap;
use std::io;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::blocks::{BlockId, ThrusterType};
use crate::data::Data;

#[derive(Error, Debug)]
pub enum ReadError {
  #[error("Could not read grid from JSON")]
  FromJSON(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum WriteError {
  #[error("Could not write grid to JSON")]
  ToJSON(#[from] serde_json::Error),
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub enum Direction {
  Up,
  Down,
  Front,
  Back,
  Left,
  Right
}

impl Direction {
  pub fn iter() -> impl Iterator<Item=&'static Direction> {
    use self::Direction::*;
    static SIDES: [Direction; 6] = [Up, Down, Front, Back, Left, Right];
    SIDES.iter()
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GridCalculator {
  pub gravity_multiplier: f64,
  pub container_multiplier: f64,
  pub planetary_influence: f64,
  pub additional_mass: f64,
  pub ice_only_fill: f64,
  pub ore_only_fill: f64,
  pub any_fill_with_ice: f64,
  pub any_fill_with_ore: f64,
  pub any_fill_with_steel_plates: f64,
  pub blocks: HashMap<BlockId, u64>,
  pub directional_blocks: HashMap<Direction, HashMap<BlockId, u64>>,
}

impl Default for GridCalculator {
  fn default() -> Self {
    Self {
      gravity_multiplier: 1.0,
      container_multiplier: 1.0,
      planetary_influence: 1.0,
      ice_only_fill: 100.0,
      ore_only_fill: 100.0,
      any_fill_with_ice: 0.0,
      any_fill_with_ore: 0.0,
      any_fill_with_steel_plates: 0.0,
      additional_mass: 0.0,
      blocks: Default::default(),
      directional_blocks: {
        let mut map = HashMap::default();
        map.insert(Direction::Up, HashMap::default());
        map.insert(Direction::Down, HashMap::default());
        map.insert(Direction::Front, HashMap::default());
        map.insert(Direction::Back, HashMap::default());
        map.insert(Direction::Left, HashMap::default());
        map.insert(Direction::Right, HashMap::default());
        map
      },
    }
  }
}

impl GridCalculator {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_json<R: io::Read>(reader: R) -> Result<Self, ReadError> {
    let grid = serde_json::from_reader::<_, Self>(reader)?;
    Ok(grid)
  }

  pub fn to_json<W: io::Write>(&self, writer: W) -> Result<(), WriteError> {
    serde_json::to_writer_pretty(writer, self)?;
    Ok(())
  }

  pub fn iter_block_counts(&self) -> impl Iterator<Item=(&BlockId, &u64)> {
    self.blocks.iter()
  }

  pub fn calculate(&self, data: &Data) -> GridCalculated {
    let ice_weight_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ice_items_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ore_weight_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ore_items_per_volume = 1.0 / 0.37; // TODO: derive from data
    let steel_plate_weight_per_volume = 20.0 / 3.0; // TODO: derive from data
    let steel_plate_items_per_volume = 1.0 / 3.0; // TODO: derive from data

    let mut c = GridCalculated::default();

    let mut power_consumption_idle = 0.0;
    let mut power_consumption_misc = 0.0;
    let mut power_consumption_generator = 0.0;
    let power_consumption_jump_drive = 0.0;
    let mut power_consumption_thruster: HashMap<Direction, f64> = HashMap::default();
    let mut power_consumption_battery = 0.0;

    let mut hydrogen_consumption_idle = 0.0;
    let mut hydrogen_consumption_engine = 0.0;
    let mut hydrogen_consumption_thruster: HashMap<Direction, f64> = HashMap::default();

    c.total_mass_empty += self.additional_mass;

    // Containers.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.containers.get(id) {
        let count = *count as f64;
        c.total_mass_empty += block.mass(&data.components) * count;
        if block.store_any {
          let volume = block.details.capacity * count * self.container_multiplier;
          c.total_volume_any += volume;
          c.total_volume_ore += volume;
          c.total_volume_ice += volume;
        }
      }
    }
    // Cockpits.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.cockpits.get(id) {
        let count = *count as f64;
        c.total_mass_empty += block.mass(&data.components) * count;
        if block.has_inventory {
          let volume = block.details.capacity * count * self.container_multiplier;
          c.total_volume_any += volume;
          c.total_volume_ore += volume;
          c.total_volume_ice += volume;
        }
      }
    }
    // Thrusters.
    for (side, blocks) in self.directional_blocks.iter() {
      let mut a = AccelerationCalculated::default();
      for (id, count) in blocks {
        if let Some(block) = data.blocks.thrusters.get(id) {
          let count = *count as f64;
          let details = &block.details;
          c.total_mass_empty += block.mass(&data.components) * count;
          // Clamp planetary influence value.
          let planetary_influence = self.planetary_influence.clamp(details.min_planetary_influence, details.max_planetary_influence);
          // Slope-intercept form equation: y = mx + b
          // Calculate m: m = (y2 - y1) / (x2 - x1)
          let m = (details.effectiveness_at_min_influence - details.effectiveness_at_max_influence) / (details.min_planetary_influence - details.max_planetary_influence);
          // Calculate b: b = y + -mx (choose x,y on the line)
          let b = details.effectiveness_at_max_influence + (-1.0 * m * details.max_planetary_influence);
          // Calculate y: y = mx + b
          let effectiveness = m * planetary_influence + b;
          a.force += details.force * effectiveness * count;
          match details.ty {
            ThrusterType::Hydrogen => {
              hydrogen_consumption_idle += details.actual_min_consumption(&data.gas_properties) * count;
              let max_consumption = details.actual_max_consumption(&data.gas_properties) * count;
              hydrogen_consumption_thruster.entry(*side).and_modify(|c| *c += max_consumption).or_insert(max_consumption);
            },
            _ => {
              power_consumption_idle += details.actual_min_consumption(&data.gas_properties) * count;
              let max_consumption = details.actual_max_consumption(&data.gas_properties) * count;
              power_consumption_thruster.entry(*side).and_modify(|c| *c += max_consumption).or_insert(max_consumption);
            },
          }
        }
      }
      c.acceleration.insert(*side, a);
    }
    // Hydrogen Engines.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.hydrogen_engines.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.power_generation += details.max_power_generation * count;
        hydrogen_consumption_engine += details.max_fuel_consumption * count;
        c.hydrogen_capacity_engine += details.fuel_capacity * count;
      }
    }
    // Reactors.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.reactors.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.power_generation += details.max_power_generation * count;
        // TODO: fuel capacity/use
      }
    }
    // Batteries.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.batteries.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.power_generation += details.output * count;
        power_consumption_battery += details.input * count;
        c.power_capacity_battery += details.capacity * count;
      }
    }
    // Hydrogen Generators.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.generators.get(id) {
        let count = *count as f64;
        let details = &block.details;
        // Mass
        c.total_mass_empty += block.mass(&data.components) * count;
        // Volume
        c.total_volume_ice_only += details.inventory_volume_ice * count;
        // Power consumption
        power_consumption_idle += details.idle_power_consumption * count;
        power_consumption_generator += details.operational_power_consumption * count;
        c.hydrogen_generation += details.hydrogen_generation * count;
        // TODO: ice consumption
      }
    }
    // Hydrogen Tanks.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.hydrogen_tanks.get(id) {
        let count = *count as f64;
        let details = &block.details;
        // Mass
        c.total_mass_empty += block.mass(&data.components) * count;
        power_consumption_idle += details.idle_power_consumption * count;
        power_consumption_misc += details.operational_power_consumption * count;
        c.hydrogen_capacity_tank += details.capacity * count;
      }
    }

    // TODO: add jump drive block
    // TODO: add gyroscopes
    // TODO: add drills

    // Calculate filled volumes.
    let ice_only_volume = c.total_volume_ice_only * (self.ice_only_fill / 100.0);
    let ore_only_volume = c.total_volume_ore_only * (self.ore_only_fill / 100.0);
    let ice_in_any_volume = c.total_volume_any * (self.any_fill_with_ice / 100.0);
    let ore_in_any_volume = c.total_volume_any * (self.any_fill_with_ore / 100.0);
    let steel_plates_in_any_volume = c.total_volume_any * (self.any_fill_with_steel_plates / 100.0);

    // Calculate filled mass.
    // TODO: container multiplier increases volume but keeps mass the same!
    let ice_only_mass = ice_only_volume * ice_weight_per_volume;
    let ore_only_mass = ore_only_volume * ore_weight_per_volume;
    let any_mass = (ice_in_any_volume * ice_weight_per_volume) + (ore_in_any_volume * ore_weight_per_volume) + (steel_plates_in_any_volume * steel_plate_weight_per_volume);
    c.total_mass_filled = c.total_mass_empty + ice_only_mass + ore_only_mass + any_mass;

    // Calculate filled items.
    c.total_items_ore = (ore_only_volume + ore_in_any_volume) * ore_items_per_volume;
    c.total_items_ice = (ice_only_volume + ice_in_any_volume) * ice_items_per_volume;
    c.total_items_steel_plate = steel_plates_in_any_volume * steel_plate_items_per_volume;

    // Calculate Acceleration
    for a in c.acceleration.values_mut() {
      a.acceleration_empty_no_gravity = a.force / c.total_mass_empty;
      a.acceleration_filled_no_gravity = a.force / c.total_mass_filled;
      a.acceleration_empty_gravity = (a.force - (c.total_mass_empty * 9.81 * self.gravity_multiplier)) / c.total_mass_empty;
      a.acceleration_filled_gravity = (a.force - (c.total_mass_filled * 9.81 * self.gravity_multiplier)) / c.total_mass_filled;
    }

    {
      c.power_idle = c.power_resource(power_consumption_idle);
      let mut consumption = power_consumption_misc;
      c.power_misc = c.power_resource(consumption);
      consumption += power_consumption_jump_drive;
      c.power_upto_jump_drive = c.power_resource(consumption);
      consumption += power_consumption_generator;
      c.power_upto_generator = c.power_resource(consumption);
      consumption += Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Up, Direction::Down);
      c.power_upto_up_down_thruster = c.power_resource(consumption);
      consumption += Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Front, Direction::Back);
      c.power_upto_front_back_thruster = c.power_resource(consumption);
      consumption += Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Left, Direction::Right);
      c.power_upto_left_right_thruster = c.power_resource(consumption);
      consumption += power_consumption_battery;
      c.power_upto_battery = c.power_resource(consumption);
    }

    {
      c.hydrogen_idle = c.hydrogen_resource(hydrogen_consumption_idle);
      let mut consumption = hydrogen_consumption_engine;
      c.hydrogen_engine = c.hydrogen_resource(consumption);
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Up, Direction::Down);
      c.hydrogen_upto_up_down_thruster = c.hydrogen_resource(consumption);
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Front, Direction::Back);
      c.hydrogen_upto_front_back_thruster = c.hydrogen_resource(consumption);
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Left, Direction::Right);
      c.hydrogen_upto_left_right_thruster = c.hydrogen_resource(consumption);
    }

    c
  }

  fn thruster_consumption_peak(map: &HashMap<Direction, f64>, side_1: Direction, side2: Direction) -> f64 {
    let c1 = map.get(&side_1).map(|c| *c).unwrap_or(0.0);
    let c2 = map.get(&side2).map(|c| *c).unwrap_or(0.0);
    c1.max(c2)
  }
}


#[derive(Default)]
pub struct GridCalculated {
  pub total_volume_any: f64,
  pub total_volume_ore: f64,
  pub total_volume_ice: f64,
  pub total_volume_ore_only: f64,
  pub total_volume_ice_only: f64,
  pub total_mass_empty: f64,
  pub total_mass_filled: f64,
  pub total_items_ore: f64,
  pub total_items_ice: f64,
  pub total_items_steel_plate: f64,

  pub acceleration: HashMap<Direction, AccelerationCalculated>,

  pub power_generation: f64,
  pub power_capacity_battery: f64,
  pub power_idle: ResourceCalculated,
  pub power_misc: ResourceCalculated,
  pub power_upto_generator: ResourceCalculated,
  pub power_upto_jump_drive: ResourceCalculated,
  pub power_upto_up_down_thruster: ResourceCalculated,
  pub power_upto_front_back_thruster: ResourceCalculated,
  pub power_upto_left_right_thruster: ResourceCalculated,
  pub power_upto_battery: ResourceCalculated,

  pub hydrogen_generation: f64,
  pub hydrogen_capacity_tank: f64,
  pub hydrogen_capacity_engine: f64,
  pub hydrogen_idle: ResourceCalculated,
  pub hydrogen_engine: ResourceCalculated,
  pub hydrogen_upto_up_down_thruster: ResourceCalculated,
  pub hydrogen_upto_front_back_thruster: ResourceCalculated,
  pub hydrogen_upto_left_right_thruster: ResourceCalculated,
}

#[derive(Default)]
pub struct AccelerationCalculated {
  pub force: f64,
  pub acceleration_empty_no_gravity: f64,
  pub acceleration_empty_gravity: f64,
  pub acceleration_filled_no_gravity: f64,
  pub acceleration_filled_gravity: f64,
}

#[derive(Default)]
pub struct ResourceCalculated {
  pub consumption: f64,
  pub balance: f64,
  pub duration: f64,
}

impl ResourceCalculated {
  fn new(consumption: f64, generation: f64, capacity: f64, conversion_rate: f64) -> Self {
    let balance = generation - consumption;
    let duration = (capacity / consumption) * conversion_rate;
    ResourceCalculated { consumption, balance, duration }
  }
}

impl GridCalculated {
  fn power_resource(&self, consumption: f64) -> ResourceCalculated {
    ResourceCalculated::new(consumption, self.power_generation, self.power_capacity_battery, 60.0 /* MWh to mins */)
  }

  fn hydrogen_resource(&self, consumption: f64) -> ResourceCalculated {
    ResourceCalculated::new(consumption, self.hydrogen_generation, self.hydrogen_capacity_tank, 1.0 / 60.0 /* L/s to mins */)
  }
}
