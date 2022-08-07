use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::blocks::{BlockId, ThrusterType};
use crate::data::Data;

// Direction

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum Direction {
  #[default] Up,
  Down,
  Front,
  Back,
  Left,
  Right
}

impl Direction {
  #[inline]
  pub fn items() -> impl IntoIterator<Item=Self> {
    use Direction::*;
    const ITEMS: [Direction; 6] = [Up, Down, Front, Back, Left, Right];
    ITEMS.into_iter()
  }

  #[inline]
  pub const fn into_index(self) -> usize {
    use Direction::*;
    match self {
      Up => 0,
      Down => 1,
      Front => 2,
      Back => 3,
      Left => 4,
      Right => 5,
    }
  }
}

impl Display for Direction {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Direction::Up => f.write_str("Up"),
      Direction::Down => f.write_str("Down"),
      Direction::Front => f.write_str("Front"),
      Direction::Back => f.write_str("Back"),
      Direction::Left => f.write_str("Left"),
      Direction::Right => f.write_str("Right"),
    }
  }
}


// Per-direction data

#[repr(transparent)]
#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct PerDirection<T>([T; 6]);

impl<T> PerDirection<T> {
  #[inline]
  pub const fn get(&self, direction: Direction) -> &T { &self.0[direction.into_index()] }
  #[inline]
  pub fn get_mut(&mut self, direction: Direction) -> &mut T { &mut self.0[direction.into_index()] }

  #[inline]
  pub const fn up(&self) -> &T { self.get(Direction::Up) }
  #[inline]
  pub const fn down(&self) -> &T { self.get(Direction::Down) }
  #[inline]
  pub const fn front(&self) -> &T { self.get(Direction::Front) }
  #[inline]
  pub const fn back(&self) -> &T { self.get(Direction::Back) }
  #[inline]
  pub const fn left(&self) -> &T { self.get(Direction::Left) }
  #[inline]
  pub const fn right(&self) -> &T { self.get(Direction::Right) }

  #[inline]
  pub fn up_mut(&mut self) -> &mut T { self.get_mut(Direction::Up) }
  #[inline]
  pub fn down_mut(&mut self) -> &mut T { self.get_mut(Direction::Down) }
  #[inline]
  pub fn front_mut(&mut self) -> &mut T { self.get_mut(Direction::Front) }
  #[inline]
  pub fn back_mut(&mut self) -> &mut T { self.get_mut(Direction::Back) }
  #[inline]
  pub fn left_mut(&mut self) -> &mut T { self.get_mut(Direction::Left) }
  #[inline]
  pub fn right_mut(&mut self) -> &mut T { self.get_mut(Direction::Right) }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item=&T> { self.0.iter() }
  #[inline]
  pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> { self.0.iter_mut() }

  #[inline] //noinspection RsBorrowChecker
  pub fn iter_with_direction(&self) -> impl Iterator<Item=(Direction, &T)> {
    Direction::items().into_iter().map(|d| (d, &self[d]))
  }
}

impl<T> Index<Direction> for PerDirection<T> {
  type Output = T;
  #[inline]
  fn index(&self, index: Direction) -> &Self::Output {
    &self.0[index.into_index()]
  }
}

impl<T> IndexMut<Direction> for PerDirection<T> {
  #[inline]
  fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
    &mut self.0[index.into_index()]
  }
}

pub type CountPerDirection = PerDirection<u64>;


// Calculator

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum BatteryMode {
  #[default] Auto,
  Recharge,
  Discharge,
  Off,
}

impl BatteryMode {
  #[inline]
  pub fn items() -> impl IntoIterator<Item=Self> {
    use BatteryMode::*;
    const ITEMS: [BatteryMode; 4] = [Auto, Recharge, Discharge, Off];
    ITEMS.into_iter()
  }

  #[inline]
  pub fn is_charging(&self) -> bool {
    use BatteryMode::*;
    match self { Auto => true, Recharge => true, _ => false }
  }

  #[inline]
  pub fn is_discharging(&self) -> bool {
    use BatteryMode::*;
    match self { Auto => true, Discharge => true, _ => false }
  }
}

impl Display for BatteryMode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    use BatteryMode::*;
    match self {
      Auto => f.write_str("Auto"),
      Recharge => f.write_str("Recharge"),
      Discharge => f.write_str("Discharge"),
      Off => f.write_str("Off"),
    }
  }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct GridCalculator {
  /// Gravity multiplier 0-* (g)
  pub gravity_multiplier: f64,
  /// Container multiplier 0-*
  pub container_multiplier: f64,
  /// Planetary influence 0-1
  pub planetary_influence: f64,
  /// Battery mode
  pub battery_mode: BatteryMode,
  /// Engines enabled?
  pub engine_enabled: bool,
  /// Are railguns charging?
  pub railgun_charging: bool,
  /// Are jump drives charging?
  pub jump_drive_charging: bool,
  /// Thruster power 0-100%
  pub thruster_power: f64,
  /// Wheel power 0-100%
  pub wheel_power: f64,
  /// Additional mass (kg)
  pub additional_mass: f64,
  /// Ice only fill 0-100%
  pub ice_only_fill: f64,
  /// Ore only fill 0-100%
  pub ore_only_fill: f64,
  /// Any fill with ice 0-100%
  pub any_fill_with_ice: f64,
  /// Any fill with ore 0-100%
  pub any_fill_with_ore: f64,
  /// Any fill with steel plates 0-100%
  pub any_fill_with_steel_plates: f64,
  /// Block counts
  pub blocks: HashMap<BlockId, u64>,
  /// Block counts per direction.
  pub directional_blocks: HashMap<BlockId, CountPerDirection>,
}

impl Default for GridCalculator {
  fn default() -> Self {
    Self {
      gravity_multiplier: 1.0,
      container_multiplier: 1.0,
      planetary_influence: 1.0,
      battery_mode: Default::default(),
      engine_enabled: true,
      railgun_charging: true,
      jump_drive_charging: true,
      thruster_power: 100.0,
      wheel_power: 100.0,
      ice_only_fill: 100.0,
      ore_only_fill: 100.0,
      any_fill_with_ice: 0.0,
      any_fill_with_ore: 0.0,
      any_fill_with_steel_plates: 0.0,
      additional_mass: 0.0,
      blocks: Default::default(),
      directional_blocks: Default::default(),
    }
  }
}

impl GridCalculator {
  pub fn new() -> Self {
    Self::default()
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
    let mut power_consumption_railgun = 0.0;
    let mut power_consumption_utility = 0.0;
    let mut power_consumption_wheel_suspension = 0.0;
    let mut power_consumption_jump_drive = 0.0;
    let mut power_consumption_generator = 0.0;
    let mut power_consumption_thruster: PerDirection<f64> = PerDirection::default();
    let mut power_consumption_battery = 0.0;

    let mut hydrogen_consumption_idle = 0.0;
    let mut hydrogen_consumption_engine = 0.0;
    let mut hydrogen_consumption_thruster: PerDirection<f64> = PerDirection::default();

    c.total_mass_empty += self.additional_mass;

    // Containers.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.containers.get(id) {
        let count = *count as f64;
        c.total_mass_empty += block.mass(&data.components) * count;
        if block.store_any {
          let volume = block.details.inventory_volume_any * count * self.container_multiplier;
          c.total_volume_any += volume;
          c.total_volume_ore += volume;
          c.total_volume_ice += volume;
        }
      }
    }
    // Connectors.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.connectors.get(id) {
        let count = *count as f64;
        c.total_mass_empty += block.mass(&data.components) * count;
        let volume = block.details.inventory_volume_any * count * self.container_multiplier;
        c.total_volume_any += volume;
        c.total_volume_ore += volume;
        c.total_volume_ice += volume;
      }
    }
    // Cockpits.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.cockpits.get(id) {
        let count = *count as f64;
        c.total_mass_empty += block.mass(&data.components) * count;
        if block.has_inventory {
          let volume = block.details.inventory_volume_any * count * self.container_multiplier;
          c.total_volume_any += volume;
          c.total_volume_ore += volume;
          c.total_volume_ice += volume;
        }
      }
    }
    { // Thrusters
      let power_ratio = self.thruster_power / 100.0;
      for (id, count_per_direction) in self.directional_blocks.iter() {
        for (direction, count) in count_per_direction.iter_with_direction() {
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
            c.thruster_acceleration[direction].force += details.force * power_ratio * effectiveness * count;
            match details.ty {
              ThrusterType::Hydrogen => {
                hydrogen_consumption_idle += details.actual_min_consumption(&data.gas_properties) * count;
                let max_consumption = details.actual_max_consumption(&data.gas_properties) * power_ratio * count; // TODO: should this be multiplied with `effectiveness`?
                hydrogen_consumption_thruster[direction] += max_consumption;
              },
              _ => {
                power_consumption_idle += details.actual_min_consumption(&data.gas_properties) * count;
                let max_consumption = details.actual_max_consumption(&data.gas_properties) * power_ratio * count; // TODO: should this be multiplied with `effectiveness`?
                power_consumption_thruster[direction] += max_consumption;
              },
            }
          }
        }
      }
    }
    { // Wheel suspensions
      let power_ratio = self.wheel_power / 100.0;
      for (id, count) in self.blocks.iter() {
        if let Some(block) = data.blocks.wheel_suspensions.get(id) {
          let count = *count as f64;
          let details = &block.details;
          c.total_mass_empty += block.mass(&data.components) * count;
          c.wheel_force += details.force * count * power_ratio;
          power_consumption_idle += details.idle_power_consumption * count;
          power_consumption_wheel_suspension += details.operational_power_consumption * count * power_ratio;
        }
      }
    }
    // Hydrogen Engines.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.hydrogen_engines.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        if self.engine_enabled {
          c.power_generation += details.max_power_generation * count;
          hydrogen_consumption_engine += details.max_fuel_consumption * count;
          c.hydrogen_engine_enabled = true;
        }
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
        // TODO: inventory - uranium ingot only
        // TODO: fuel capacity/use
      }
    }
    // Batteries.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.batteries.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        if self.battery_mode.is_discharging() {
          c.power_generation += details.output * count;
          c.power_battery_discharging = true;
        }
        if self.battery_mode.is_charging() {
          power_consumption_battery += details.input * count;
        }
        c.power_capacity_battery += details.capacity * count;
      }
    }
    // Jump drives
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.jump_drives.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        if self.jump_drive_charging {
          power_consumption_jump_drive += details.operational_power_consumption * count;
        }
      }
    }
    // Railguns
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.railguns.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        power_consumption_idle += details.idle_power_consumption * count;
        if self.railgun_charging {
          power_consumption_railgun += details.operational_power_consumption * count;
        }
      }
    }
    // Hydrogen Generators.
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.generators.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.total_volume_ice_only += details.inventory_volume_ice * count;
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
        c.total_mass_empty += block.mass(&data.components) * count;
        power_consumption_idle += details.idle_power_consumption * count;
        power_consumption_utility += details.operational_power_consumption * count;
        c.hydrogen_capacity_tank += details.capacity * count;
      }
    }
    // Drills
    for (id, count) in self.blocks.iter() {
      if let Some(block) = data.blocks.drills.get(id) {
        let count = *count as f64;
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.total_volume_ore_only += details.inventory_volume_ore * count;
        power_consumption_idle += details.idle_power_consumption * count;
        power_consumption_utility += details.operational_power_consumption * count;
      }
    }

    // TODO: add jump drive block
    // TODO: add gyroscopes

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
    let has_mass_empty = c.total_mass_empty != 0.0;
    let has_mass_filled = c.total_mass_filled != 0.0;
    for a in c.thruster_acceleration.iter_mut() {
      a.acceleration_empty_no_gravity = has_mass_empty.then(|| a.force / c.total_mass_empty);
      a.acceleration_filled_no_gravity = has_mass_filled.then(|| a.force / c.total_mass_filled);
      a.acceleration_empty_gravity = has_mass_empty.then(|| (a.force - (c.total_mass_empty * 9.81 * self.gravity_multiplier)) / c.total_mass_empty);
      a.acceleration_filled_gravity = has_mass_filled.then(|| (a.force - (c.total_mass_filled * 9.81 * self.gravity_multiplier)) / c.total_mass_filled);
    }

    {
      c.power_idle = c.power_resource(power_consumption_idle, power_consumption_idle);
      let mut total_consumption = power_consumption_railgun;
      c.power_railgun = c.power_resource(power_consumption_railgun, total_consumption);
      total_consumption += power_consumption_utility;
      c.power_upto_utility = c.power_resource(power_consumption_utility, total_consumption);
      total_consumption += power_consumption_wheel_suspension;
      c.power_upto_wheel_suspension = c.power_resource(power_consumption_wheel_suspension, total_consumption);
      total_consumption += power_consumption_jump_drive;
      c.power_upto_jump_drive = c.power_resource(power_consumption_jump_drive, total_consumption);
      total_consumption += power_consumption_generator;
      c.power_upto_generator = c.power_resource(power_consumption_generator, total_consumption);
      let up_down_consumption = Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Up, Direction::Down);
      total_consumption += up_down_consumption;
      c.power_upto_up_down_thruster = c.power_resource(up_down_consumption, total_consumption);
      let front_left_consumption = Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Front, Direction::Back);
      total_consumption += front_left_consumption;
      c.power_upto_front_back_thruster = c.power_resource(front_left_consumption, total_consumption);
      let left_right_consumption = Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Left, Direction::Right);
      total_consumption += left_right_consumption;
      c.power_upto_left_right_thruster = c.power_resource(left_right_consumption, total_consumption);
      total_consumption += power_consumption_battery;
      c.power_upto_battery = c.power_resource(power_consumption_battery, total_consumption);
    }

    {
      c.hydrogen_idle = c.hydrogen_resource(hydrogen_consumption_idle);
      let mut consumption = hydrogen_consumption_engine;
      c.hydrogen_engine = c.hydrogen_resource(consumption);
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Up, Direction::Down);
      c.hydrogen_upto_up_down_thruster = c.hydrogen_tank_only_resource(consumption);
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Front, Direction::Back);
      c.hydrogen_upto_front_back_thruster = c.hydrogen_tank_only_resource(consumption);
      consumption += Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Left, Direction::Right);
      c.hydrogen_upto_left_right_thruster = c.hydrogen_tank_only_resource(consumption);
    }

    c
  }

  fn thruster_consumption_peak(per_direction: &PerDirection<f64>, direction_a: Direction, direction_b: Direction) -> f64 {
    per_direction[direction_a].max(per_direction[direction_b])
  }
}


// Calculated data

#[derive(Default)]
pub struct GridCalculated {
  /// Total volume available in inventories that accept any item (L)
  pub total_volume_any: f64,
  /// Total volume available for ore in inventories that accept any item (L)
  pub total_volume_ore: f64,
  /// Total volume available for ice in inventories that accept any item (L)
  pub total_volume_ice: f64,
  /// Total volume available for ore in inventories that accept only ore (L)
  pub total_volume_ore_only: f64,
  /// Total volume available for ore in inventories that accept only ice (L)
  pub total_volume_ice_only: f64,
  /// Total mass without items (kg)
  pub total_mass_empty: f64,
  /// Total mass when fully filled with items (kg)
  pub total_mass_filled: f64,
  /// Total number of ore that can are stored
  pub total_items_ore: f64,
  /// Total number of ice that can are stored
  pub total_items_ice: f64,
  /// Total number of steel plates that can are stored
  pub total_items_steel_plate: f64,

  pub thruster_acceleration: PerDirection<ThrusterAccelerationCalculated>,
  /// Force (N)
  pub wheel_force: f64,

  /// Total power generation (MW)
  pub power_generation: f64,
  /// Total power capacity in batteries (MWh)
  pub power_capacity_battery: f64,
  /// Whether batteries are discharging
  pub power_battery_discharging: bool,
  /// Idle power calculation
  pub power_idle: PowerCalculated,
  /// Railgun (charging) power calculation
  pub power_railgun: PowerCalculated,
  /// + Utility power calculation
  pub power_upto_utility: PowerCalculated,
  /// + Wheel suspension power calculation
  pub power_upto_wheel_suspension: PowerCalculated,
  /// + Jump drive (charging) power calculation
  pub power_upto_jump_drive: PowerCalculated,
  /// + Generator power calculation
  pub power_upto_generator: PowerCalculated,
  /// + Up/down thruster power calculation
  pub power_upto_up_down_thruster: PowerCalculated,
  /// + Front/back thruster power calculation
  pub power_upto_front_back_thruster: PowerCalculated,
  /// + Left/right thruster power calculation
  pub power_upto_left_right_thruster: PowerCalculated,
  /// + Battery (charging) power calculation
  pub power_upto_battery: PowerCalculated,

  /// Total hydrogen generation (L/s)
  pub hydrogen_generation: f64,
  /// Total hydrogen capacity in tanks (L)
  pub hydrogen_capacity_tank: f64,
  /// Total hydrogen capacity in engines (L)
  pub hydrogen_capacity_engine: f64,
  /// Whether hydrogen engines are enabled
  pub hydrogen_engine_enabled: bool,
  /// Idle hydrogen calculation
  pub hydrogen_idle: HydrogenCalculated,
  /// + Engine (filling) hydrogen calculation
  pub hydrogen_engine: HydrogenCalculated,
  /// + Up/down thruster hydrogen calculation
  pub hydrogen_upto_up_down_thruster: HydrogenCalculated,
  /// + Front/back thruster hydrogen calculation
  pub hydrogen_upto_front_back_thruster: HydrogenCalculated,
  /// + Left/right thruster hydrogen calculation
  pub hydrogen_upto_left_right_thruster: HydrogenCalculated,
}

#[derive(Default)]
pub struct ThrusterAccelerationCalculated {
  /// Force (N)
  pub force: f64,
  /// Acceleration when empty and outside of gravity (m/s^2)
  pub acceleration_empty_no_gravity: Option<f64>,
  /// Acceleration when empty and inside of gravity (m/s^2)
  pub acceleration_empty_gravity: Option<f64>,
  /// Acceleration when filled and outside of gravity (m/s^2)
  pub acceleration_filled_no_gravity: Option<f64>,
  /// Acceleration when filled and outside of gravity (m/s^2)
  pub acceleration_filled_gravity: Option<f64>,
}

#[derive(Default)]
pub struct PowerCalculated {
  /// Power consumption of this group (MW)
  pub consumption: f64,
  /// Total power consumption upto this group (MW)
  pub total_consumption: f64,
  /// Power balance upto this group (+-MW)
  pub balance: f64,
  /// Duration until batteries are empty when discharging (min), or None if there are no batteries
  /// or they are not discharging.
  pub duration_battery: Option<f64>,
}

impl PowerCalculated {
  fn new(consumption: f64, total_consumption: f64, generation: f64, capacity_battery: f64, battery_discharging: bool, conversion_rate: f64) -> Self {
    let balance = generation - total_consumption;
    let duration_battery = (total_consumption != 0.0 && capacity_battery != 0.0 && battery_discharging).then(|| (capacity_battery / total_consumption) * conversion_rate);
    PowerCalculated { consumption, total_consumption, balance, duration_battery }
  }
}

#[derive(Default)]
pub struct HydrogenCalculated {
  /// Total hydrogen consumption upto this group (L/s)
  pub total_consumption: f64,
  /// Hydrogen balance upto this group (+-L/s)
  pub balance: f64,
  /// Duration until hydrogen tanks are empty when discharging (min), or None if there are no 
  /// hydrogen tanks or they are stockpiling. TODO: stockpiling behaviour
  pub duration_tank: Option<f64>,
  /// TODO: remove, this one makes no sense, as engines don't make their hydrogen available.
  pub duration_engine: Option<f64>,
}

impl HydrogenCalculated {
  fn new(consumption: f64, generation: f64, capacity_tanks: f64, capacity_engines: Option<f64>, engine_enabled: bool, conversion_rate: f64) -> Self {
    let balance = generation - consumption;
    let has_consumption = consumption != 0.0;
    let duration_tank = (has_consumption && capacity_tanks != 0.0).then(|| (capacity_tanks / consumption) * conversion_rate);
    let has_capacity_engines = capacity_engines.map_or(false, |c| c != 0.0);
    let duration_engine = (has_consumption && has_capacity_engines && engine_enabled).then(|| consumption).and_then(|_| capacity_engines.map(|c| (c / consumption) * conversion_rate));
    HydrogenCalculated { total_consumption: consumption, balance, duration_tank, duration_engine }
  }
}

impl GridCalculated {
  fn power_resource(&self, consumption: f64, total_consumption: f64) -> PowerCalculated {
    PowerCalculated::new(consumption, total_consumption, self.power_generation, self.power_capacity_battery, self.power_battery_discharging, 60.0 /* MWh to mins */)
  }

  fn hydrogen_resource(&self, consumption: f64) -> HydrogenCalculated {
    HydrogenCalculated::new(consumption, self.hydrogen_generation, self.hydrogen_capacity_tank, Some(self.hydrogen_capacity_engine), self.hydrogen_engine_enabled, 1.0 / 60.0 /* L/s to mins */)
  }

  fn hydrogen_tank_only_resource(&self, consumption: f64) -> HydrogenCalculated {
    HydrogenCalculated::new(consumption, self.hydrogen_generation, self.hydrogen_capacity_tank, None, self.hydrogen_engine_enabled, 1.0 / 60.0 /* L/s to mins */)
  }
}


// JSON serde

impl GridCalculator {
  pub fn from_json<R: io::Read>(reader: R) -> Result<Self, ReadError> {
    let grid = serde_json::from_reader::<_, Self>(reader)?;
    Ok(grid)
  }

  pub fn to_json<W: io::Write>(&self, writer: W) -> Result<(), WriteError> {
    serde_json::to_writer_pretty(writer, self)?;
    Ok(())
  }
}

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