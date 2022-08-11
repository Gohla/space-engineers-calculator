use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use direction::PerDirection;

use crate::data::blocks::{BlockId, ThrusterType};
use crate::data::Data;
use crate::grid::direction::{CountPerDirection, Direction};
use crate::grid::duration::Duration;

pub mod direction;
pub mod duration;

// Battery mode

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum BatteryMode {
  Auto,
  Recharge,
  #[default] Discharge,
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


// Hydrogen tank mode

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum HydrogenTankMode {
  #[default] On,
  Stockpile,
  Off,
}

impl HydrogenTankMode {
  #[inline]
  pub fn items() -> impl IntoIterator<Item=Self> {
    use HydrogenTankMode::*;
    const ITEMS: [HydrogenTankMode; 3] = [On, Stockpile, Off];
    ITEMS.into_iter()
  }

  #[inline]
  pub fn is_refilling(&self) -> bool {
    use HydrogenTankMode::*;
    match self { On => true, Stockpile => true, _ => false }
  }

  #[inline]
  pub fn is_providing(&self) -> bool {
    use HydrogenTankMode::*;
    match self { On => true, _ => false }
  }
}

impl Display for HydrogenTankMode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    use HydrogenTankMode::*;
    match self {
      On => f.write_str("On"),
      Stockpile => f.write_str("Stockpile"),
      Off => f.write_str("Off"),
    }
  }
}

// Calculator

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct GridCalculator {
  /// Gravity multiplier 0-* (g)
  pub gravity_multiplier: f64,
  /// Container multiplier 0-*
  pub container_multiplier: f64,
  /// Planetary influence 0-1
  pub planetary_influence: f64,
  /// Additional mass (kg)
  pub additional_mass: f64,

  /// Thruster power 0-100%
  pub thruster_power: f64,
  /// Wheel power 0-100%
  pub wheel_power: f64,

  /// Are railguns charging?
  pub railgun_charging: bool,
  /// Are jump drives charging?
  pub jump_drive_charging: bool,
  /// Battery mode
  pub battery_mode: BatteryMode,
  /// Fill level of batteries 0-100%
  pub battery_fill: f64,

  /// Hydrogen tanks mode?
  pub hydrogen_tank_mode: HydrogenTankMode,
  /// Fill level of hydrogen tanks 0-100%
  pub hydrogen_tank_fill: f64,
  /// Hydrogen engines enabled?
  pub hydrogen_engine_enabled: bool,
  /// Fill level of hydrogen engines 0-100%
  pub hydrogen_engine_fill: f64,

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
      additional_mass: 0.0,

      thruster_power: 100.0,
      wheel_power: 100.0,

      railgun_charging: true,
      jump_drive_charging: true,
      battery_mode: Default::default(),
      battery_fill: 100.0,

      hydrogen_tank_mode: Default::default(),
      hydrogen_tank_fill: 100.0,
      hydrogen_engine_enabled: true,
      hydrogen_engine_fill: 100.0,

      ice_only_fill: 100.0,
      ore_only_fill: 100.0,
      any_fill_with_ice: 0.0,
      any_fill_with_ore: 0.0,
      any_fill_with_steel_plates: 0.0,

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
    let mut hydrogen_consumption_tank = 0.0;

    let mut jump_strength = 0.0; // Divide by mass to get max jump distance.
    let mut max_jump_distance = 0.0; // Cap on max jump distance.

    c.total_mass_empty += self.additional_mass;

    // Non-directional blocks
    let wheel_power_ratio = self.wheel_power / 100.0;
    for (id, count) in self.blocks.iter().filter(|(_, c)| **c != 0) {
      let count = *count as f64;
      if let Some(block) = data.blocks.containers.get(id) { // Containers.
        c.total_mass_empty += block.mass(&data.components) * count;
        if block.store_any {
          let volume = block.details.inventory_volume_any * count * self.container_multiplier;
          c.total_volume_any += volume;
          c.total_volume_ore += volume;
          c.total_volume_ice += volume;
        }
      } else if let Some(block) = data.blocks.connectors.get(id) { // Connectors.
        c.total_mass_empty += block.mass(&data.components) * count;
        let volume = block.details.inventory_volume_any * count * self.container_multiplier;
        c.total_volume_any += volume;
        c.total_volume_ore += volume;
        c.total_volume_ice += volume;
      } else if let Some(block) = data.blocks.cockpits.get(id) { // Cockpits.
        c.total_mass_empty += block.mass(&data.components) * count;
        if block.has_inventory {
          let volume = block.details.inventory_volume_any * count * self.container_multiplier;
          c.total_volume_any += volume;
          c.total_volume_ore += volume;
          c.total_volume_ice += volume;
        }
      } else if let Some(block) = data.blocks.wheel_suspensions.get(id) { // Wheel suspensions
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.wheel_force += details.force * count * wheel_power_ratio;
        power_consumption_idle += details.idle_power_consumption * count;
        power_consumption_wheel_suspension += details.operational_power_consumption * count * wheel_power_ratio;
      } else if let Some(block) = data.blocks.hydrogen_engines.get(id) { // Hydrogen Engines.
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        let maximum_fuel_consumption = details.max_fuel_consumption * count;
        let maximum_power_output = details.max_power_generation * count;
        let maximum_refilling_input = maximum_fuel_consumption * 60.0; // Hydrogen engine input is multiplied by 60 when not full in MyFueledPowerProducer.cs
        if self.hydrogen_engine_enabled {
          c.power_generation += maximum_power_output;
          hydrogen_consumption_engine += if self.hydrogen_engine_fill != 100.0 {
            maximum_refilling_input
          } else {
            maximum_fuel_consumption
          };
        }
        let hydrogen_engine = c.hydrogen_engine.get_or_insert(HydrogenEngineCalculated::default());
        hydrogen_engine.capacity += details.fuel_capacity * count;
        hydrogen_engine.maximum_fuel_consumption = maximum_fuel_consumption;
        hydrogen_engine.maximum_output = maximum_power_output;
        hydrogen_engine.maximum_refilling_input = maximum_refilling_input;
      } else if let Some(block) = data.blocks.reactors.get(id) { // Reactors.
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.power_generation += details.max_power_generation * count;
        // TODO: inventory - uranium ingot only
        // TODO: fuel capacity/use
      } else if let Some(block) = data.blocks.batteries.get(id) { // Batteries.
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        let input = details.input * count;
        let output = details.output * count;
        if self.battery_mode.is_charging() {
          power_consumption_battery += input;
        }
        if self.battery_mode.is_discharging() {
          c.power_generation += output;
        }
        let battery = c.battery.get_or_insert(BatteryCalculated::default());
        battery.capacity += details.capacity * count;
        battery.maximum_input += input;
        battery.maximum_output += output;
      } else if let Some(block) = data.blocks.jump_drives.get(id) { // Jump drives
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        let input = details.operational_power_consumption * count;
        if self.jump_drive_charging {
          power_consumption_jump_drive += input;
        }
        let jump_drive = c.jump_drive.get_or_insert(JumpDriveCalculated::default());
        jump_drive.capacity += block.capacity * count;
        jump_drive.maximum_input = input;
        // Formula based on https://www.spaceengineerswiki.com/Jump_drive
        let max_jump_drive_distance = details.max_jump_distance / 1000.0; // Convert from m to km.
        jump_strength += max_jump_drive_distance * details.max_jump_mass * count;
        max_jump_distance += max_jump_drive_distance * count;
      } else if let Some(block) = data.blocks.railguns.get(id) { // Railguns
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        let input = details.operational_power_consumption * count;
        power_consumption_idle += details.idle_power_consumption * count;
        if self.railgun_charging {
          power_consumption_railgun += input;
        }
        let railgun = c.railgun.get_or_insert(RailgunCalculated::default());
        railgun.capacity += block.capacity * count;
        railgun.maximum_input = input;
      } else if let Some(block) = data.blocks.generators.get(id) { // Hydrogen Generators.
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.total_volume_ice_only += details.inventory_volume_ice * count;
        power_consumption_idle += details.idle_power_consumption * count;
        power_consumption_generator += details.operational_power_consumption * count;
        c.hydrogen_generation += details.hydrogen_generation * count;
        // TODO: ice consumption
      } else if let Some(block) = data.blocks.hydrogen_tanks.get(id) { // Hydrogen Tanks.
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        let maximum_input_output = details.capacity * 0.05; // Hydrogen tank consumption is capacity * 0.05 when not full according to MyGasTank.cs
        if self.hydrogen_tank_mode.is_refilling() {
          power_consumption_idle += details.idle_power_consumption * count;
          power_consumption_utility += details.operational_power_consumption * count;
          hydrogen_consumption_tank = if self.hydrogen_tank_fill != 100.0 {
            maximum_input_output
          } else {
            0.0
          };
        }
        let hydrogen_tank = c.hydrogen_tank.get_or_insert(HydrogenTankCalculated::default());
        hydrogen_tank.capacity += details.capacity * count;
        hydrogen_tank.maximum_input = maximum_input_output;
        hydrogen_tank.maximum_output = maximum_input_output;
      } else if let Some(block) = data.blocks.drills.get(id) { // Drills
        let details = &block.details;
        c.total_mass_empty += block.mass(&data.components) * count;
        c.total_volume_ore_only += details.inventory_volume_ore * count;
        power_consumption_idle += details.idle_power_consumption * count;
        power_consumption_utility += details.operational_power_consumption * count;
      }
    }
    // Directional blocks
    let thruster_power_ratio = self.thruster_power / 100.0;
    for (id, count_per_direction) in self.directional_blocks.iter() {
      for (direction, count) in count_per_direction.iter_with_direction() {
        if let Some(block) = data.blocks.thrusters.get(id) { // Thrusters
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
          c.thruster_acceleration[direction].force += details.force * thruster_power_ratio * effectiveness * count;
          match details.ty {
            ThrusterType::Hydrogen => {
              hydrogen_consumption_idle += details.actual_min_consumption(&data.gas_properties) * count;
              let max_consumption = details.actual_max_consumption(&data.gas_properties) * thruster_power_ratio * effectiveness * count;
              hydrogen_consumption_thruster[direction] += max_consumption;
            },
            _ => {
              power_consumption_idle += details.actual_min_consumption(&data.gas_properties) * count;
              let max_consumption = details.actual_max_consumption(&data.gas_properties) * thruster_power_ratio * effectiveness * count;
              power_consumption_thruster[direction] += max_consumption;
            },
          }
        }
      }
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

    // Calculate power
    let (actual_power_consumption_railgun, actual_power_consumption_jump_drive, actual_power_consumption_battery) = {
      struct PowerCalculatedBuilder {
        generation: f64,
        battery_capacity: Option<f64>,
        battery_fill: f64,
        battery_generation: f64,
        battery_discharging: bool,
        engine_capacity: Option<f64>,
        engine_fill: f64,
        engine_fuel_consumption: f64,
        engine_generation: f64,
        engine_is_generating_power: bool
      }
      impl PowerCalculatedBuilder {
        fn power_resource(&self, consumption: f64, total_consumption: f64) -> PowerCalculated {
          let balance = self.generation - total_consumption;
          let battery_duration = if total_consumption != 0.0 && self.battery_discharging {
            self.battery_capacity.map(|c| Duration::from_hours(c * (self.battery_fill / 100.0) / total_consumption.min(self.battery_generation)))
          } else {
            None
          };
          let engine_duration = if total_consumption != 0.0 && self.engine_is_generating_power {
            self.engine_capacity.map(|c| {
              let capacity = c * (self.engine_fill / 100.0);
              Duration::from_seconds((capacity / self.engine_fuel_consumption) * (self.engine_generation / total_consumption.min(self.engine_generation)))
            })
          } else {
            None
          };
          PowerCalculated { consumption, total_consumption, balance, battery_duration, engine_duration }
        }
      }
      let b = PowerCalculatedBuilder {
        generation: c.power_generation,
        battery_capacity: c.battery.as_ref().map(|b| b.capacity),
        battery_fill: self.battery_fill,
        battery_generation: c.battery.as_ref().map(|b| b.maximum_output).unwrap_or(0.0),
        battery_discharging: self.battery_mode.is_discharging() && self.battery_fill != 0.0,
        engine_capacity: c.hydrogen_engine.as_ref().map(|e| e.capacity),
        engine_fill: self.hydrogen_engine_fill,
        engine_fuel_consumption: c.hydrogen_engine.as_ref().map(|e| e.maximum_fuel_consumption).unwrap_or(0.0),
        engine_generation: c.hydrogen_engine.as_ref().map(|e| e.maximum_output).unwrap_or(0.0),
        engine_is_generating_power: self.hydrogen_engine_enabled && self.hydrogen_engine_fill != 0.0,
      };

      // Idle
      c.power_idle = b.power_resource(power_consumption_idle, power_consumption_idle);

      // Non-idle
      // Defense (railgun)
      let actual_power_consumption_railgun = power_consumption_railgun.min(c.power_generation).max(0.0);
      let mut total_consumption = power_consumption_railgun;
      c.power_railgun_charge = b.power_resource(power_consumption_railgun, total_consumption);
      // Utility
      total_consumption += power_consumption_utility;
      c.power_upto_utility = b.power_resource(power_consumption_utility, total_consumption);
      // Utility (wheel suspensions)
      total_consumption += power_consumption_wheel_suspension;
      c.power_upto_wheel_suspension = b.power_resource(power_consumption_wheel_suspension, total_consumption);
      // Charge jump drive
      let actual_power_consumption_jump_drive = power_consumption_jump_drive.min(c.power_upto_wheel_suspension.balance).max(0.0);
      total_consumption += power_consumption_jump_drive;
      c.power_upto_jump_drive_charge = b.power_resource(power_consumption_jump_drive, total_consumption);
      // Generator
      total_consumption += power_consumption_generator;
      c.power_upto_generator = b.power_resource(power_consumption_generator, total_consumption);
      // Thrust - Up/Down
      let up_down_consumption = Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Up, Direction::Down);
      total_consumption += up_down_consumption;
      c.power_upto_up_down_thruster = b.power_resource(up_down_consumption, total_consumption);
      // Thrust - Front/Back
      let front_back_consumption = Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Front, Direction::Back);
      total_consumption += front_back_consumption;
      c.power_upto_front_back_thruster = b.power_resource(front_back_consumption, total_consumption);
      // Thrust - Left/Right
      let left_right_consumption = Self::thruster_consumption_peak(&power_consumption_thruster, Direction::Left, Direction::Right);
      total_consumption += left_right_consumption;
      c.power_upto_left_right_thruster = b.power_resource(left_right_consumption, total_consumption);
      // Charge battery
      let actual_power_consumption_battery = power_consumption_battery.min(c.power_upto_left_right_thruster.balance).max(0.0);
      total_consumption += power_consumption_battery;
      c.power_upto_battery_charge = b.power_resource(power_consumption_battery, total_consumption);

      (actual_power_consumption_railgun, actual_power_consumption_jump_drive, actual_power_consumption_battery)
    };

    if let Some(railgun) = &mut c.railgun { // TODO: is this also 80% efficient?
      railgun.charge_duration = self.railgun_charging.then(|| Duration::from_hours(railgun.capacity / actual_power_consumption_railgun));
    }

    const CHARGE_EFFICIENCY: f64 = 0.8;

    if let Some(jump_drive) = &mut c.jump_drive {
      // TODO: use efficiency from jump drive data, instead of hardcoded 80% efficiency!
      let should_charge = self.jump_drive_charging;
      jump_drive.charge_duration = should_charge.then(|| Duration::from_hours(jump_drive.capacity / (actual_power_consumption_jump_drive * CHARGE_EFFICIENCY)));
      jump_drive.max_distance_empty = (jump_strength / c.total_mass_empty).min(max_jump_distance);
      jump_drive.max_distance_filled = (jump_strength / c.total_mass_filled).min(max_jump_distance);
    }

    if let Some(battery) = &mut c.battery {
      let anti_fill = 1.0 - self.battery_fill / 100.0;
      let should_charge = self.battery_mode.is_charging() && self.battery_fill != 100.0;
      battery.charge_duration = should_charge.then(|| Duration::from_hours((battery.capacity * anti_fill) / (actual_power_consumption_battery * CHARGE_EFFICIENCY)));
    }

    // Calculate Hydrogen
    let (actual_hydrogen_consumption_tank, actual_hydrogen_consumption_engine) = {
      struct HydrogenCalculatedBuilder {
        generation: f64,
        tank_capacity: Option<f64>,
        tank_fill: f64,
        tank_generation: f64,
        tank_is_providing_hydrogen: bool,
      }
      impl HydrogenCalculatedBuilder {
        fn hydrogen_resource(&self, consumption: f64, total_consumption: f64) -> HydrogenCalculated {
          let balance_without_tank = self.generation - total_consumption;
          let balance_with_tank = if self.tank_is_providing_hydrogen {
            self.generation + self.tank_generation - total_consumption
          } else {
            balance_without_tank
          };
          let has_consumption = total_consumption != 0.0;
          let tank_duration = if has_consumption && self.tank_is_providing_hydrogen {
            self.tank_capacity.map(|c| Duration::from_seconds((c * (self.tank_fill / 100.0)) / total_consumption.min(self.tank_generation)))
          } else {
            None
          };
          HydrogenCalculated { consumption, total_consumption, balance_without_tank, balance_with_tank, tank_duration }
        }
      }
      let mut b = HydrogenCalculatedBuilder {
        generation: c.hydrogen_generation,
        tank_capacity: c.hydrogen_tank.as_ref().map(|t| t.capacity),
        tank_fill: self.hydrogen_tank_fill,
        tank_generation: c.hydrogen_tank.as_ref().map(|t| t.maximum_output).unwrap_or(0.0),
        tank_is_providing_hydrogen: self.hydrogen_tank_mode.is_providing() && self.hydrogen_tank_fill != 0.0,
      };

      // Idle
      c.hydrogen_idle = b.hydrogen_resource(hydrogen_consumption_idle, hydrogen_consumption_idle);
      // Non-idle
      // Hydrogen engine
      let actual_hydrogen_consumption_engine = hydrogen_consumption_engine.min(c.hydrogen_generation).max(0.0);
      let mut total_consumption = hydrogen_consumption_engine;
      c.hydrogen_engine_fill = b.hydrogen_resource(hydrogen_consumption_engine, total_consumption);
      // Thrust - Up/Down
      let up_down_consumption = Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Up, Direction::Down);
      total_consumption += up_down_consumption;
      c.hydrogen_upto_up_down_thruster = b.hydrogen_resource(up_down_consumption, total_consumption);
      // Thrust - Front/Back
      let front_back_consumption = Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Front, Direction::Back);
      total_consumption += front_back_consumption;
      c.hydrogen_upto_front_back_thruster = b.hydrogen_resource(front_back_consumption, total_consumption);
      // Thrust - Left/Right
      let left_right_consumption = Self::thruster_consumption_peak(&hydrogen_consumption_thruster, Direction::Left, Direction::Right);
      total_consumption += left_right_consumption;
      c.hydrogen_upto_left_right_thruster = b.hydrogen_resource(left_right_consumption, total_consumption);
      // Tank
      let actual_hydrogen_consumption_tank = hydrogen_consumption_tank.min(c.hydrogen_generation).max(0.0);
      total_consumption += hydrogen_consumption_tank;
      b.tank_is_providing_hydrogen = false; // Disable tank duration for tanks.
      c.hydrogen_upto_tank_fill = b.hydrogen_resource(hydrogen_consumption_tank, total_consumption);

      (actual_hydrogen_consumption_tank, actual_hydrogen_consumption_engine)
    };

    if let Some(hydrogen_tank) = &mut c.hydrogen_tank {
      let anti_fill = 1.0 - self.hydrogen_tank_fill / 100.0;
      let should_refill = self.hydrogen_tank_mode.is_refilling() && self.hydrogen_tank_fill != 100.0;
      hydrogen_tank.fill_duration = should_refill.then(|| Duration::from_seconds((hydrogen_tank.capacity * anti_fill) / actual_hydrogen_consumption_tank));
    }

    if let Some(hydrogen_engine) = &mut c.hydrogen_engine {
      let anti_fill = 1.0 - self.hydrogen_engine_fill / 100.0;
      let should_refill = self.hydrogen_engine_enabled && self.hydrogen_engine_fill != 100.0;
      hydrogen_engine.fill_duration = should_refill.then(|| Duration::from_seconds((hydrogen_engine.capacity * anti_fill) / actual_hydrogen_consumption_engine));
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

  /// Thruster force (N) and acceleration (m/s^2)
  pub thruster_acceleration: PerDirection<ThrusterAccelerationCalculated>,
  /// Wheel force (N)
  pub wheel_force: f64,

  /// Total power generation (MW)
  pub power_generation: f64,
  /// Idle power calculation
  pub power_idle: PowerCalculated,
  /// Railgun (charging) power calculation
  pub power_railgun_charge: PowerCalculated,
  /// + Utility power calculation
  pub power_upto_utility: PowerCalculated,
  /// + Wheel suspension power calculation
  pub power_upto_wheel_suspension: PowerCalculated,
  /// + Jump drive (charging) power calculation
  pub power_upto_jump_drive_charge: PowerCalculated,
  /// + Generator power calculation
  pub power_upto_generator: PowerCalculated,
  /// + Up/down thruster power calculation
  pub power_upto_up_down_thruster: PowerCalculated,
  /// + Front/back thruster power calculation
  pub power_upto_front_back_thruster: PowerCalculated,
  /// + Left/right thruster power calculation
  pub power_upto_left_right_thruster: PowerCalculated,
  /// + Battery (charging) power calculation
  pub power_upto_battery_charge: PowerCalculated,

  /// Railgun calculation, or None if there are no railguns.
  pub railgun: Option<RailgunCalculated>,
  /// Jump drive calculation, or None if there are no jump drives.
  pub jump_drive: Option<JumpDriveCalculated>,
  /// Battery calculation, or None if there are no batteries.
  pub battery: Option<BatteryCalculated>,

  /// Total hydrogen generation (L/s)
  pub hydrogen_generation: f64,
  /// Idle hydrogen calculation
  pub hydrogen_idle: HydrogenCalculated,
  /// + Engine (filling) hydrogen calculation
  pub hydrogen_engine_fill: HydrogenCalculated,
  /// + Up/down thruster hydrogen calculation
  pub hydrogen_upto_up_down_thruster: HydrogenCalculated,
  /// + Front/back thruster hydrogen calculation
  pub hydrogen_upto_front_back_thruster: HydrogenCalculated,
  /// + Left/right thruster hydrogen calculation
  pub hydrogen_upto_left_right_thruster: HydrogenCalculated,
  /// + Tank (filling) hydrogen calculation
  pub hydrogen_upto_tank_fill: HydrogenCalculated,

  /// Hydrogen tank calculation, or None if there are no hydrogen tanks.
  pub hydrogen_tank: Option<HydrogenTankCalculated>,
  /// Hydrogen engine calculation, or None if there are no hydrogen engines.
  pub hydrogen_engine: Option<HydrogenEngineCalculated>,
}

#[derive(Default, Copy, Clone)]
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

#[derive(Default, Copy, Clone)]
pub struct PowerCalculated {
  /// Power consumption of this group (MW)
  pub consumption: f64,
  /// Total power consumption upto this group (MW)
  pub total_consumption: f64,
  /// Power balance upto this group (+-MW)
  pub balance: f64,
  /// Duration until batteries are empty when discharging (min), or None if there are no batteries
  /// or they are not discharging.
  pub battery_duration: Option<Duration>,
  /// Duration until engines are empty when discharging (min), or None if there are no engines
  /// or they are not enabled.
  pub engine_duration: Option<Duration>,
}

#[derive(Default)]
pub struct RailgunCalculated {
  /// Total power capacity in railguns (MWh)
  pub capacity: f64,
  /// Maximum power input (MW)
  pub maximum_input: f64,
  /// Duration until railguns are full when charging (min), or None if railguns are not charging.
  pub charge_duration: Option<Duration>,
}

#[derive(Default)]
pub struct JumpDriveCalculated {
  /// Total power capacity in jump drives (MWh)
  pub capacity: f64,
  /// Maximum power input (MW)
  pub maximum_input: f64,
  /// Duration until jump drives are full when charging (min), or None if jump drives are not 
  /// charging.
  pub charge_duration: Option<Duration>,
  /// Maximum jump distance when empty (km)
  pub max_distance_empty: f64,
  /// Maximum jump distance when filled (km)
  pub max_distance_filled: f64,
}

#[derive(Default)]
pub struct BatteryCalculated {
  /// Total power capacity in batteries (MWh)
  pub capacity: f64,
  /// Maximum power input (MW)
  pub maximum_input: f64,
  /// Maximum power output (MW)
  pub maximum_output: f64,
  /// Duration until batteries are full when charging (min), or None if batteries are not charging.
  pub charge_duration: Option<Duration>,
}

#[derive(Default, Copy, Clone)]
pub struct HydrogenCalculated {
  /// Hydrogen consumption of this group (L/s)
  pub consumption: f64,
  /// Total hydrogen consumption upto this group (L/s)
  pub total_consumption: f64,
  /// Hydrogen balance upto this group, without hydrogen provided by tanks (+-L/s)
  pub balance_without_tank: f64,
  /// Hydrogen balance upto this group, with hydrogen provided by tanks (+-L/s)
  pub balance_with_tank: f64,
  /// Duration until hydrogen tanks are empty when discharging (min), or None if there are no 
  /// hydrogen tanks or they are stockpiling.
  pub tank_duration: Option<Duration>,
}

#[derive(Default)]
pub struct HydrogenTankCalculated {
  /// Total hydrogen capacity in hydrogen tanks (L)
  pub capacity: f64,
  /// Maximum hydrogen input (L/s)
  pub maximum_input: f64,
  /// Maximum hydrogen output (L/s)
  pub maximum_output: f64,
  /// Duration until hydrogen tanks are full(min), or None if hydrogen tanks are disabled.
  pub fill_duration: Option<Duration>,
}

#[derive(Default)]
pub struct HydrogenEngineCalculated {
  /// Total hydrogen capacity in hydrogen engines (L)
  pub capacity: f64,
  /// Maximum fuel consumption (L/s)
  pub maximum_fuel_consumption: f64,
  /// Maximum power output (MW)
  pub maximum_output: f64,
  /// Maximum hydrogen input when refilling (L/s)
  pub maximum_refilling_input: f64,
  /// Duration until hydrogen engines are full (min), or None if hydrogen engines are disabled.
  pub fill_duration: Option<Duration>,
}