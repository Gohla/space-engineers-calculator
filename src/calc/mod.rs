use std::collections::HashMap;
use std::io;

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::data::blocks::{BlockId, ThrusterType};
use crate::data::Data;

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Could not read calculator from JSON: {}", source))]
  FromJSON { source: serde_json::Error, },
  #[snafu(display("Could not write calculator to JSON: {}", source))]
  ToJSON { source: serde_json::Error, },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub enum ThrusterSide {
  Up,
  Down,
  Front,
  Back,
  Left,
  Right
}

impl ThrusterSide {
  pub fn iter() -> impl Iterator<Item=&'static ThrusterSide> {
    use self::ThrusterSide::*;
    static SIDES: [ThrusterSide; 6] = [Up, Down, Front, Back, Left, Right];
    SIDES.iter()
  }
}

#[derive(Serialize, Deserialize)]
pub struct Calculator {
  pub gravity_multiplier: f64,
  pub container_multiplier: f64,
  pub planetary_influence: f64,
  pub additional_mass: f64,
  pub ice_only_fill: f64,
  pub ore_only_fill: f64,
  pub any_fill_with_ice: f64,
  pub any_fill_with_ore: f64,
  pub any_fill_with_steel_plates: f64,

  pub containers: HashMap<BlockId, u64>,
  pub cockpits: HashMap<BlockId, u64>,
  // TODO: drills

  pub thrusters: HashMap<ThrusterSide, HashMap<BlockId, u64>>,

  pub hydrogen_engines: HashMap<BlockId, u64>,
  pub reactors: HashMap<BlockId, u64>,
  pub batteries: HashMap<BlockId, u64>,

  pub generators: HashMap<BlockId, u64>,
  pub hydrogen_tanks: HashMap<BlockId, u64>,
}

impl Calculator {
  pub fn new() -> Self {
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

      containers: Default::default(),
      cockpits: Default::default(),

      thrusters: {
        let mut map = HashMap::default();
        map.insert(ThrusterSide::Up, HashMap::default());
        map.insert(ThrusterSide::Down, HashMap::default());
        map.insert(ThrusterSide::Front, HashMap::default());
        map.insert(ThrusterSide::Back, HashMap::default());
        map.insert(ThrusterSide::Left, HashMap::default());
        map.insert(ThrusterSide::Right, HashMap::default());
        map
      },

      hydrogen_engines: Default::default(),
      reactors: Default::default(),
      batteries: Default::default(),

      generators: Default::default(),
      hydrogen_tanks: Default::default(),
    }
  }

  pub fn from_json<R: io::Read>(reader: R) -> Result<Self> {
    serde_json::from_reader(reader).context(self::FromJSON)
  }

  pub fn to_json<W: io::Write>(&self, writer: W) -> Result<()> {
    serde_json::to_writer_pretty(writer, self).context(self::ToJSON)?;
    Ok(())
  }

  pub fn iter_block_counts(&self) -> impl Iterator<Item=(&BlockId, &u64)> {
    vec![
      self.containers.iter(),
      self.cockpits.iter(),
      self.hydrogen_engines.iter(),
      self.reactors.iter(),
      self.batteries.iter(),
      self.generators.iter(),
      self.hydrogen_tanks.iter(),
    ].into_iter().flat_map(|it| it)
  }

  pub fn calculate(&self, data: &Data) -> Calculated {
    let ice_weight_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ice_items_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ore_weight_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ore_items_per_volume = 1.0 / 0.37; // TODO: derive from data
    let steel_plate_weight_per_volume = 20.0 / 3.0; // TODO: derive from data
    let steel_plate_items_per_volume = 1.0 / 3.0; // TODO: derive from data

    let mut c = Calculated::default();

    let mut power_consumption_idle = 0.0;
    let mut power_consumption_misc = 0.0;
    let mut power_consumption_generator = 0.0;
    let power_consumption_jump_drive = 0.0;
    let mut power_consumption_thruster: HashMap<ThrusterSide, f64> = HashMap::default();
    let mut power_consumption_battery = 0.0;

    let mut hydrogen_consumption_idle = 0.0;
    let mut hydrogen_consumption_engine = 0.0;
    let mut hydrogen_consumption_thruster: HashMap<ThrusterSide, f64> = HashMap::default();

    c.total_mass_empty += self.additional_mass;

    // Containers.
    for (id, count) in self.containers.iter() {
      let block = data.blocks.containers.get(id).unwrap();
      let count = *count as f64;
      c.total_mass_empty += block.mass(&data.components) * count;
      if block.store_any {
        let volume = block.details.capacity * count * self.container_multiplier;
        c.total_volume_any += volume;
        c.total_volume_ore += volume;
        c.total_volume_ice += volume;
      }
    }
    // Cockpits.
    for (id, count) in self.cockpits.iter() {
      let block = data.blocks.cockpits.get(id).unwrap();
      let count = *count as f64;
      c.total_mass_empty += block.mass(&data.components) * count;
      if block.has_inventory {
        let volume = block.details.capacity * count * self.container_multiplier;
        c.total_volume_any += volume;
        c.total_volume_ore += volume;
        c.total_volume_ice += volume;
      }
    }
    // Thrusters.
    for (side, thrusters) in self.thrusters.iter() {
      let mut a = AccelerationCalculated::default();
      for (id, count) in thrusters {
        let block = data.blocks.thrusters.get(id).unwrap();
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
      c.acceleration.insert(*side, a);
    }
    // Hydrogen Engines.
    for (id, count) in self.hydrogen_engines.iter() {
      let block = data.blocks.hydrogen_engines.get(id).unwrap();
      let count = *count as f64;
      let details = &block.details;
      c.total_mass_empty += block.mass(&data.components) * count;
      c.power_generation += details.max_power_generation * count;
      hydrogen_consumption_engine += details.max_fuel_consumption * count;
      c.hydrogen_capacity_engine += details.fuel_capacity * count;
    }
    // Reactors.
    for (id, count) in self.reactors.iter() {
      let block = data.blocks.reactors.get(id).unwrap();
      let count = *count as f64;
      let details = &block.details;
      c.total_mass_empty += block.mass(&data.components) * count;
      c.power_generation += details.max_power_generation * count;
      // TODO: fuel capacity/use
    }
    // Batteries.
    for (id, count) in self.batteries.iter() {
      let block = data.blocks.batteries.get(id).unwrap();
      let count = *count as f64;
      let details = &block.details;
      c.total_mass_empty += block.mass(&data.components) * count;
      c.power_generation += details.output * count;
      power_consumption_battery += details.input * count;
      c.power_capacity_battery += details.capacity * count;
    }
    // Hydrogen Generators.
    for (id, count) in self.generators.iter() {
      let block = data.blocks.generators.get(id).unwrap();
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
    // Hydrogen Tanks.
    for (id, count) in self.hydrogen_tanks.iter() {
      let block = data.blocks.hydrogen_tanks.get(id).unwrap();
      let count = *count as f64;
      let details = &block.details;
      // Mass
      c.total_mass_empty += block.mass(&data.components) * count;
      power_consumption_idle += details.idle_power_consumption * count;
      power_consumption_misc += details.operational_power_consumption * count;
      c.hydrogen_capacity_engine += details.capacity * count;
    }

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
    c.total_items_ice = (ice_only_volume + ice_in_any_volume) * ice_items_per_volume;
    c.total_items_ore = (ore_only_volume + ore_in_any_volume) * ore_items_per_volume;
    c.total_items_steel_plate = steel_plates_in_any_volume * steel_plate_items_per_volume;

    // Calculate Acceleration
    for a in c.acceleration.values_mut() {
      a.acceleration_empty_no_gravity = a.force / c.total_mass_empty;
      a.acceleration_filled_no_gravity = a.force / c.total_mass_filled;
      a.acceleration_empty_gravity = (a.force - (c.total_mass_empty * 9.81 * self.gravity_multiplier)) / c.total_mass_empty;
      a.acceleration_filled_gravity = (a.force - (c.total_mass_filled * 9.81 * self.gravity_multiplier)) / c.total_mass_filled;
    }

    {
      let gen = c.power_generation;
      c.power_balance_idle = gen - power_consumption_idle;
      let mut consumption = power_consumption_misc;
      c.power_balance_misc = gen - consumption;
      consumption += power_consumption_jump_drive;
      c.power_balance_upto_jump_drive = gen - consumption;
      consumption += power_consumption_generator;
      c.power_balance_upto_generator = gen - consumption;
      consumption += Self::thruster_consumption_peak(&power_consumption_thruster, ThrusterSide::Up, ThrusterSide::Down);
      c.power_balance_upto_up_down_thruster = gen - consumption;
      consumption += Self::thruster_consumption_peak(&power_consumption_thruster, ThrusterSide::Front, ThrusterSide::Back);
      c.power_balance_upto_front_back_thruster = gen - consumption;
      consumption += Self::thruster_consumption_peak(&power_consumption_thruster, ThrusterSide::Left, ThrusterSide::Right);
      c.power_balance_upto_left_right_thruster = gen - consumption;
      consumption += power_consumption_battery;
      c.power_balance_upto_battery = gen - consumption;
    }

    {
      let gen = c.hydrogen_generation;
      c.hydrogen_balance_idle = gen - hydrogen_consumption_idle;
      let mut consumption = hydrogen_consumption_engine;
      c.hydrogen_balance_engine = gen - consumption;
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, ThrusterSide::Up, ThrusterSide::Down);
      c.hydrogen_balance_upto_up_down_thruster = gen - consumption;
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, ThrusterSide::Front, ThrusterSide::Back);
      c.hydrogen_balance_upto_front_back_thruster = gen - consumption;
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, ThrusterSide::Left, ThrusterSide::Right);
      c.hydrogen_balance_upto_left_right_thruster = gen - consumption;
    }

    c
  }

  fn thruster_consumption_peak(map: &HashMap<ThrusterSide, f64>, side_1: ThrusterSide, side2: ThrusterSide) -> f64 {
    let c1 = map.get(&side_1).map(|c| *c).unwrap_or(0.0);
    let c2 = map.get(&side2).map(|c| *c).unwrap_or(0.0);
    c1.max(c2)
  }
}


#[derive(Default)]
pub struct Calculated {
  pub total_volume_any: f64,
  pub total_volume_ore: f64,
  pub total_volume_ice: f64,
  pub total_volume_ore_only: f64,
  pub total_volume_ice_only: f64,
  pub total_mass_empty: f64,
  pub total_mass_filled: f64,
  pub total_items_ice: f64,
  pub total_items_ore: f64,
  pub total_items_steel_plate: f64,

  pub acceleration: HashMap<ThrusterSide, AccelerationCalculated>,

  pub power_generation: f64,
  pub power_balance_idle: f64,
  pub power_balance_misc: f64,
  pub power_balance_upto_generator: f64,
  // TODO: add jump drive block
  pub power_balance_upto_jump_drive: f64,
  // TODO: gyros?
  // TODO: hover balance
  pub power_balance_upto_up_down_thruster: f64,
  pub power_balance_upto_front_back_thruster: f64,
  pub power_balance_upto_left_right_thruster: f64,
  pub power_balance_upto_battery: f64,
  pub power_capacity_battery: f64,

  pub hydrogen_generation: f64,
  pub hydrogen_balance_idle: f64,
  pub hydrogen_balance_engine: f64,
  // TODO: hover balance
  pub hydrogen_balance_upto_up_down_thruster: f64,
  pub hydrogen_balance_upto_front_back_thruster: f64,
  pub hydrogen_balance_upto_left_right_thruster: f64,
  pub hydrogen_capacity_tank: f64,
  pub hydrogen_capacity_engine: f64,
}

#[derive(Default)]
pub struct AccelerationCalculated {
  pub force: f64,
  pub acceleration_empty_no_gravity: f64,
  pub acceleration_empty_gravity: f64,
  pub acceleration_filled_no_gravity: f64,
  pub acceleration_filled_gravity: f64,
}