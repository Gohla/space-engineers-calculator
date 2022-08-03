use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

use super::components::Components;
use super::gas_properties::GasProperties;
use super::localization::Localization;

#[cfg(feature = "extract")]
pub mod extract;

/// Grid size.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug, )]
pub enum GridSize {
  #[default] Small,
  Large
}

impl GridSize {
  /// Cube size as defined by Configuration.sbc
  pub fn size(&self) -> f64 {
    match self {
      GridSize::Small => 0.5,
      GridSize::Large => 2.5,
    }
  }
}

impl Display for GridSize {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      GridSize::Small => f.write_str("Small"),
      GridSize::Large => f.write_str("Large"),
    }
  }
}


/// Alias for block identifiers.
pub type BlockId = String;

/// Common block data which can be created from a definition in a SBC XML file.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct BlockData {
  pub id: BlockId,
  pub name: String,
  pub size: GridSize,
  pub components: LinkedHashMap<String, f64>,
  pub has_physics: bool,

  pub index: u64,
  pub hidden: bool,
  pub rename: Option<String>,
}

impl BlockData {
  #[inline]
  pub fn id_cloned(&self) -> BlockId { self.id.clone() }

  #[inline]
  pub fn name<'a>(&'a self, localization: &'a Localization) -> &'a str {
    if let Some(rename) = &self.rename {
      &rename
    } else {
      localization.get(&self.name).unwrap_or(&self.name)
    }
  }

  #[inline]
  pub fn mass(&self, components: &Components) -> f64 {
    let mut mass = 0.0;
    if !self.has_physics { return mass }
    for (component_id, count) in self.components.iter() {
      if let Some(component) = components.get(&component_id) {
        mass += component.mass * *count;
      }
    }
    mass
  }
}

impl PartialEq for BlockData {
  #[inline]
  fn eq(&self, other: &Self) -> bool { self.id.eq(&other.id) }
}

impl Eq for BlockData {}

impl PartialOrd for BlockData {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.id.partial_cmp(&other.id) }
}

impl Ord for BlockData {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering { self.id.cmp(&other.id) }
}


/// Block with data and details.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Block<T> {
  pub data: BlockData,
  pub details: T,
}

impl<T> Block<T> {
  #[inline]
  pub fn new(data: BlockData, details: T) -> Self {
    Block { data, details }
  }


  #[inline]
  pub fn id(&self) -> &BlockId { &self.data.id }
  #[inline]
  pub fn id_cloned(&self) -> BlockId { self.data.id_cloned() }

  #[inline]
  pub fn name<'a>(&'a self, localization: &'a Localization) -> &'a str {
    self.data.name(localization)
  }

  #[inline]
  pub fn mass(&self, components: &Components) -> f64 { self.data.mass(components) }
}

impl<T> PartialEq for Block<T> {
  #[inline]
  fn eq(&self, other: &Self) -> bool { self.data.eq(&other.data) }
}

impl<T> Eq for Block<T> {}

impl<T> PartialOrd for Block<T> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.data.partial_cmp(&other.data) }
}

impl<T> Ord for Block<T> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering { self.data.cmp(&other.data) }
}

impl<T> Deref for Block<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.details
  }
}


/// Battery.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Battery {
  /// Power capacity (MWh)
  pub capacity: f64,
  /// Maximum power input (MW)
  pub input: f64,
  /// Maximum power output (MW)
  pub output: f64,
}

/// Battery.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct JumpDrive {
  /// Maximum power input (MW)
  pub input: f64,
}

/// Type of thruster
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Serialize, Deserialize)]
pub enum ThrusterType {
  Ion,
  Atmospheric,
  Hydrogen,
}

/// Thruster.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thruster {
  /// Thruster type
  pub ty: ThrusterType,
  /// Optional fuel gas ID, used to determine actual consumption.
  pub fuel_gas_id: Option<String>,
  /// Force (N)
  pub force: f64,
  /// Maximum consumption (MW for energy-based thrusters, otherwise max_consumption/<energy_density of fuel> L/s for fuel-based thrusters)
  pub max_consumption: f64,
  /// Minimum consumption (MW for energy-based thrusters, otherwise min_consumption/<energy_density of fuel> L/s for fuel-based thrusters)
  pub min_consumption: f64,
  pub min_planetary_influence: f64,
  pub max_planetary_influence: f64,
  pub effectiveness_at_min_influence: f64,
  pub effectiveness_at_max_influence: f64,
  pub needs_atmosphere_for_influence: bool,
}

impl Thruster {
  pub fn actual_max_consumption(&self, gas_properties: &GasProperties) -> f64 {
    if let Some(id) = &self.fuel_gas_id {
      if let Some(gas_property) = gas_properties.get(id) {
        self.max_consumption / gas_property.energy_density
      } else {
        self.max_consumption
      }
    } else {
      self.max_consumption
    }
  }

  pub fn actual_min_consumption(&self, gas_properties: &GasProperties) -> f64 {
    if let Some(id) = &self.fuel_gas_id {
      if let Some(gas_property) = gas_properties.get(id) {
        self.min_consumption / gas_property.energy_density
      } else {
        self.min_consumption
      }
    } else {
      self.min_consumption
    }
  }
}

/// Wheel suspension.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WheelSuspension {
  /// Force (N)
  pub force: f64,
  /// Operational power consumption (MW)
  pub operational_power_consumption: f64,
  /// Idle power consumption (MW)
  pub idle_power_consumption: f64,
}

/// Hydrogen engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HydrogenEngine {
  /// Fuel capacity (L)
  pub fuel_capacity: f64,
  /// Maximum power generation (MW)
  pub max_power_generation: f64,
  /// Maximum fuel consumption (L/s)
  pub max_fuel_consumption: f64,
}

/// Reactor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reactor {
  /// Maximum power generation (MW)
  pub max_power_generation: f64,
  /// Maximum fuel usage (#/s)
  pub max_fuel_consumption: f64,
}

/// Generator (O2/H2)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generator {
  /// Ice consumption (#/s)
  pub ice_consumption: f64,
  /// Inventory volume - ice only (L)
  pub inventory_volume_ice: f64,
  /// Operational power consumption (MW)
  pub operational_power_consumption: f64,
  /// Idle power consumption (MW)
  pub idle_power_consumption: f64,
  /// Oxygen generation (L/s)
  pub oxygen_generation: f64,
  /// Hydrogen generation (L/s)
  pub hydrogen_generation: f64,
}

/// Hydrogen tank
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HydrogenTank {
  /// Hydrogen capacity (L)
  pub capacity: f64,
  /// Operational power consumption (MW)
  pub operational_power_consumption: f64,
  /// Idle power consumption (MW)
  pub idle_power_consumption: f64,
}

/// Container
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container {
  /// Inventory volume (L)
  pub inventory_volume_any: f64,
  /// Stores any item?
  pub store_any: bool,
}

/// Connector
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connector {
  /// Inventory volume (L)
  pub inventory_volume_any: f64,
}

/// Cockpit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cockpit {
  /// Whether 'cockpit' has an inventory.
  pub has_inventory: bool,
  /// Inventory volume (L)
  pub inventory_volume_any: f64,
}

/// Drill
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Drill {
  /// Inventory volume - ore only (L)
  pub inventory_volume_ore: f64,
  /// Operational power consumption (MW)
  pub operational_power_consumption: f64,
  /// Idle power consumption (MW)
  pub idle_power_consumption: f64,
}

/// All blocks
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Blocks {
  pub batteries: LinkedHashMap<BlockId, Block<Battery>>,
  pub jump_drives: LinkedHashMap<BlockId, Block<JumpDrive>>,
  pub thrusters: LinkedHashMap<BlockId, Block<Thruster>>,
  pub wheel_suspensions: LinkedHashMap<BlockId, Block<WheelSuspension>>,
  pub hydrogen_engines: LinkedHashMap<BlockId, Block<HydrogenEngine>>,
  pub reactors: LinkedHashMap<BlockId, Block<Reactor>>,
  pub generators: LinkedHashMap<BlockId, Block<Generator>>,
  pub hydrogen_tanks: LinkedHashMap<BlockId, Block<HydrogenTank>>,
  pub containers: LinkedHashMap<BlockId, Block<Container>>,
  pub connectors: LinkedHashMap<BlockId, Block<Connector>>,
  pub cockpits: LinkedHashMap<BlockId, Block<Cockpit>>,
  pub drills: LinkedHashMap<BlockId, Block<Drill>>,
}

impl Blocks {
  pub fn thruster_blocks(&self, grid_size: GridSize) -> impl Iterator<Item=&BlockData> {
    self.thrusters.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data)
  }

  pub fn storage_blocks(&self, grid_size: GridSize) -> impl Iterator<Item=&BlockData> {
    self.containers.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data)
      .chain(self.connectors.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data))
      .chain(self.cockpits.values().filter(move |b| filter(b, grid_size) && b.has_inventory).map(|b| &b.data))
  }

  pub fn power_blocks(&self, grid_size: GridSize) -> impl Iterator<Item=&BlockData> {
    self.hydrogen_engines.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data)
      .chain(self.reactors.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data))
      .chain(self.batteries.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data))
  }

  pub fn hydrogen_blocks(&self, grid_size: GridSize) -> impl Iterator<Item=&BlockData> {
    self.generators.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data)
      .chain(self.hydrogen_tanks.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data))
  }

  pub fn ship_tool_blocks(&self, grid_size: GridSize) -> impl Iterator<Item=&BlockData> {
    self.drills.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data)
  }

  pub fn wheel_suspension_blocks(&self, grid_size: GridSize) -> impl Iterator<Item=&BlockData> {
    self.wheel_suspensions.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data)
  }

  pub fn jump_drive_blocks(&self, grid_size: GridSize) -> impl Iterator<Item=&BlockData> {
    self.jump_drives.values().filter(move |b| filter(b, grid_size)).map(|b| &b.data)
  }
}

fn filter<T>(b: &Block<T>, grid_size: GridSize) -> bool { !b.data.hidden && b.data.size == grid_size }