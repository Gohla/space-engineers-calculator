use std::backtrace::Backtrace;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};

use linked_hash_map::LinkedHashMap;
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use super::components::Components;
use super::gas_properties::GasProperties;
use super::localization::Localization;
use super::xml::{NodeExt, read_string_from_file};

/// Some calculated volumes are multiplied by this number.
pub const VOLUME_MULTIPLIER: f64 = 1000.0;

/// Default FuelProductionToCapacityMultiplier in SE's code.
pub const DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER: f64 = 3600.0;


#[derive(Error, Debug)]
pub enum Error {
  #[error("Could not read CubeBlocks file '{file}'")]
  ReadCubeBlocksFile { file: PathBuf, source: std::io::Error },
  #[error("Could not XML parse CubeBlocks file '{file}'")]
  ParseCubeBlocksFile { file: PathBuf, source: roxmltree::Error },
  #[error("Could not read EntityComponents file '{file}'")]
  ReadEntityComponentsFile { file: PathBuf, source: std::io::Error },
  #[error("Could not XML parse EntityComponents file '{file}'")]
  ParseEntityComponentsFile { file: PathBuf, source: roxmltree::Error },
  #[error("Unexpected XML structure")]
  XmlStructure(Backtrace),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;


/// Grid size.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug, )]
pub enum GridSize {
  #[default] Small,
  Large
}

impl GridSize {
  pub fn from_def(def: &Node) -> Self {
    match def.child_elem("CubeSize").unwrap().text().unwrap() {
      "Small" => GridSize::Small,
      "Large" => GridSize::Large,
      t => panic!("Unrecognized grid size {}", t),
    }
  }

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


/// Data which can be created from a definition in a SBC XML file.
pub trait FromDef: Clone + Debug {
  fn from_def(def: &Node, size: GridSize, entity_components: &Node) -> Self;
}


/// Alias for block identifiers.
pub type BlockId = String;

/// Common block data which can be created from a definition in a SBC XML file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block<T> {
  pub id: BlockId,
  pub index: u64,
  pub name: String,
  pub size: GridSize,
  pub components: LinkedHashMap<String, f64>,
  pub has_physics: bool,
  pub details: T,
}

impl<T> Block<T> {
  #[inline]
  pub fn name<'a>(&'a self, localization: &'a Localization) -> &'a str {
    localization.get(&self.name).unwrap_or(&self.name)
  }

  #[inline]
  pub fn is_small(&self) -> bool { self.size == GridSize::Small }
  #[inline]
  pub fn is_large(&self) -> bool { self.size == GridSize::Large }

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

impl<T: FromDef> Block<T> {
  pub fn from_def(def: &Node, entity_components: &Node, index: u64) -> Self {
    let id_node = def.child_elem("Id").unwrap();
    let type_id: String = id_node.parse_child_elem("TypeId").unwrap().unwrap();
    let subtype_id = id_node.parse_child_elem("SubtypeId").unwrap().unwrap_or(String::new());
    let id = type_id + "." + &subtype_id;
    let name = def.parse_child_elem("DisplayName").unwrap().unwrap();
    let mut components = LinkedHashMap::new();
    let size = GridSize::from_def(def);
    for component in def.child_elem("Components").unwrap().children_elems("Component") {
      if let (Some(component_id), Some(count)) = (component.parse_attribute("Subtype").unwrap(), component.parse_attribute::<f64, _>("Count").unwrap()) {
        *components.entry(component_id).or_insert(0.0) += count;
      }
    }
    let has_physics = def.parse_child_elem("HasPhysics").unwrap().unwrap_or(true);
    let details = T::from_def(def, size, entity_components);
    Block { id, index, name, size, components, has_physics, details }
  }
}

impl<T> PartialEq for Block<T> {
  fn eq(&self, other: &Self) -> bool {
    self.id.eq(&other.id)
  }
}

impl<T> Eq for Block<T> {}

impl<T> PartialOrd for Block<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.id.partial_cmp(&other.id)
  }
}

impl<T> Ord for Block<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.id.cmp(&other.id)
  }
}

impl<T: FromDef> Deref for Block<T> {
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

impl FromDef for Battery {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let capacity: f64 = def.parse_child_elem("MaxStoredPower").unwrap().unwrap();
    let input: f64 = def.parse_child_elem("RequiredPowerInput").unwrap().unwrap();
    let output: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    Battery { capacity, input, output }
  }
}


/// Type of thruster
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Serialize, Deserialize)]
pub enum ThrusterType {
  Ion,
  Atmospheric,
  Hydrogen,
}

impl ThrusterType {
  pub fn from_def(def: &Node) -> Self {
    match def.child_elem("ThrusterType").unwrap().text().unwrap() {
      "Ion" => ThrusterType::Ion,
      "Atmospheric" => ThrusterType::Atmospheric,
      "Hydrogen" => ThrusterType::Hydrogen,
      t => panic!("Unrecognized thruster type {}", t),
    }
  }
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

impl FromDef for Thruster {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let ty = ThrusterType::from_def(def);
    let force = def.parse_child_elem("ForceMagnitude").unwrap().unwrap();
    let fuel_gas_id = def.child_elem("FuelConverter").map(|n| n.first_element_child().unwrap().parse_child_elem("SubtypeId").unwrap().unwrap());
    let max_consumption = def.parse_child_elem("MaxPowerConsumption").unwrap().unwrap();
    let min_consumption = def.parse_child_elem("MinPowerConsumption").unwrap().unwrap();
    let min_planetary_influence = def.parse_child_elem("MinPlanetaryInfluence").unwrap().unwrap_or(0.0);
    let max_planetary_influence = def.parse_child_elem("MaxPlanetaryInfluence").unwrap().unwrap_or(1.0);
    let effectiveness_at_min_influence = def.parse_child_elem("EffectivenessAtMinInfluence").unwrap().unwrap_or(1.0);
    let effectiveness_at_max_influence = def.parse_child_elem("EffectivenessAtMaxInfluence").unwrap().unwrap_or(1.0);
    let needs_atmosphere_for_influence = def.parse_child_elem("NeedsAtmosphereForInfluence").unwrap().unwrap_or(false);
    Thruster { ty, fuel_gas_id, force, max_consumption, min_consumption, min_planetary_influence, max_planetary_influence, effectiveness_at_min_influence, effectiveness_at_max_influence, needs_atmosphere_for_influence }
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

impl FromDef for WheelSuspension {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let force: f64 = def.parse_child_elem("PropulsionForce").unwrap().unwrap();
    let operational_power_consumption: f64 = def.parse_child_elem("RequiredPowerInput").unwrap().unwrap();
    let idle_power_consumption: f64 = def.parse_child_elem("RequiredIdlePowerInput").unwrap().unwrap();
    Self { force, operational_power_consumption, idle_power_consumption }
  }
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

impl FromDef for HydrogenEngine {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let fuel_capacity: f64 = def.parse_child_elem("FuelCapacity").unwrap().unwrap();
    let max_power_generation: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    let fuel_production_to_capacity_multiplier: f64 = def.parse_child_elem("FuelProductionToCapacityMultiplier").unwrap().unwrap_or(DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER);
    let max_fuel_consumption = max_power_generation / fuel_production_to_capacity_multiplier;
    HydrogenEngine { fuel_capacity, max_power_generation, max_fuel_consumption }
  }
}


/// Reactor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reactor {
  /// Maximum power generation (MW)
  pub max_power_generation: f64,
  /// Maximum fuel usage (#/s)
  pub max_fuel_consumption: f64,
}

impl FromDef for Reactor {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let max_power_generation: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    let fuel_production_to_capacity_multiplier: f64 = def.parse_child_elem("FuelProductionToCapacityMultiplier").unwrap().unwrap_or(DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER);
    let max_fuel_consumption = max_power_generation / fuel_production_to_capacity_multiplier;
    Reactor { max_power_generation, max_fuel_consumption }
  }
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

impl FromDef for Generator {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let ice_consumption: f64 = def.parse_child_elem("IceConsumptionPerSecond").unwrap().unwrap();
    let inventory_volume_ice: f64 = def.parse_child_elem::<f64>("InventoryMaxVolume").unwrap().unwrap() * VOLUME_MULTIPLIER;
    let operational_power_consumption: f64 = def.parse_child_elem("OperationalPowerConsumption").unwrap().unwrap();
    let idle_power_consumption: f64 = def.parse_child_elem("StandbyPowerConsumption").unwrap().unwrap();
    let mut oxygen_generation = 0.0;
    let mut hydrogen_generation = 0.0;
    for gas_info in def.child_elem("ProducedGases").unwrap().children_elems("GasInfo") {
      let gas_id: String = gas_info.child_elem("Id").unwrap().parse_child_elem("SubtypeId").unwrap().unwrap();
      let ice_to_gas_ratio: f64 = gas_info.parse_child_elem("IceToGasRatio").unwrap().unwrap();
      let gas_generation = ice_consumption * ice_to_gas_ratio;
      *(match gas_id.as_ref() {
        "Oxygen" => &mut oxygen_generation,
        "Hydrogen" => &mut hydrogen_generation,
        _ => panic!("Unrecognized gas ID {} in generator {:?}", gas_id, def),
      }) = gas_generation;
    }
    Generator {
      ice_consumption,
      inventory_volume_ice,
      operational_power_consumption,
      idle_power_consumption,
      oxygen_generation,
      hydrogen_generation
    }
  }
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

impl FromDef for HydrogenTank {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let capacity: f64 = def.parse_child_elem("Capacity").unwrap().unwrap();
    let operational_power_consumption: f64 = def.parse_child_elem("OperationalPowerConsumption").unwrap().unwrap();
    let idle_power_consumption: f64 = def.parse_child_elem("StandbyPowerConsumption").unwrap().unwrap();
    HydrogenTank { capacity, operational_power_consumption, idle_power_consumption }
  }
}


/// Container
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container {
  /// Inventory volume (L)
  pub inventory_volume_any: f64,
  /// Stores any item?
  pub store_any: bool,
}

impl FromDef for Container {
  fn from_def(def: &Node, _size: GridSize, entity_components: &Node) -> Self {
    let subtype_id: String = def.child_elem("Id").unwrap().parse_child_elem("SubtypeId").unwrap().unwrap();
    let mut inventory_volume_any = 0.0;
    let mut store_any = false;
    for entity_component in entity_components.children_elems("EntityComponent") {
      if let Some("MyObjectBuilder_InventoryComponentDefinition") = entity_component.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
        let entity_component_subtype_id: String = entity_component.child_elem("Id").unwrap().parse_child_elem("SubtypeId").unwrap().unwrap();
        if subtype_id != entity_component_subtype_id { continue }
        let size = entity_component.child_elem("Size").unwrap();
        let x: f64 = size.parse_attribute("x").unwrap().unwrap();
        let y: f64 = size.parse_attribute("y").unwrap().unwrap();
        let z: f64 = size.parse_attribute("z").unwrap().unwrap();
        inventory_volume_any = x * y * z * VOLUME_MULTIPLIER;
        store_any = entity_component.child_elem("InputConstraint").map_or(true, |_| false);
        break;
      }
    }
    if inventory_volume_any == 0.0 {
      panic!("Unrecognized container {:?}", def);
    }
    Container { inventory_volume_any, store_any }
  }
}


/// Connector
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connector {
  /// Inventory volume (L)
  pub inventory_volume_any: f64,
}

impl FromDef for Connector {
  fn from_def(def: &Node, grid_size: GridSize, _entity_components: &Node) -> Self {
    let size = def.child_elem("Size").unwrap();
    let x: f64 = size.parse_attribute("x").unwrap().unwrap();
    let y: f64 = size.parse_attribute("y").unwrap().unwrap();
    let z: f64 = size.parse_attribute("z").unwrap().unwrap();
    let multiplier = grid_size.size() * 0.8;
    let inventory_volume_any = (x * multiplier) * (y * multiplier) * (z * multiplier) * VOLUME_MULTIPLIER; // Inventory capacity according to MyShipConnector.cs.
    Self {
      inventory_volume_any,
    }
  }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
/// Cockpit
pub struct Cockpit {
  /// Whether 'cockpit' has an inventory.
  pub has_inventory: bool,
  /// Inventory volume (L)
  pub inventory_volume_any: f64,
}

impl FromDef for Cockpit {
  fn from_def(def: &Node, _size: GridSize, _entity_components: &Node) -> Self {
    let has_inventory = def.parse_child_elem("HasInventory").unwrap().unwrap_or(true);
    let inventory_volume_any = if has_inventory { VOLUME_MULTIPLIER } else { 0.0 }; // Inventory capacity according to MyCockpit.cs.
    Cockpit { has_inventory, inventory_volume_any }
  }
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

impl FromDef for Drill {
  fn from_def(def: &Node, grid_size: GridSize, _entity_components: &Node) -> Self {
    let size = def.child_elem("Size").unwrap();
    let x: f64 = size.parse_attribute("x").unwrap().unwrap();
    let y: f64 = size.parse_attribute("y").unwrap().unwrap();
    let z: f64 = size.parse_attribute("z").unwrap().unwrap();
    let cube_size = grid_size.size();
    let inventory_volume_ore = x * y * z * cube_size * cube_size * cube_size * 0.5 * VOLUME_MULTIPLIER; // Inventory capacity according to MyShipDrill.cs.
    let operational_power_consumption: f64 = 1.0 / 500.0 * 1.0; // Maximum required power according to ComputeMaxRequiredPower in MyShipDrill.cs.
    let idle_power_consumption: f64 = 1e-06; // Idle power according to ComputeMaxRequiredPower in MyShipDrill.cs.
    Self {
      inventory_volume_ore,
      operational_power_consumption,
      idle_power_consumption,
    }
  }
}


#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
/// All blocks
pub struct Blocks {
  pub batteries: LinkedHashMap<BlockId, Block<Battery>>,
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
  pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P, localization: &Localization) -> Result<Self> {
    Self::from_sbc_files(se_dir_path.as_ref().join("Content/Data/"), se_dir_path.as_ref().join("Content/Data/EntityComponents.sbc"), localization)
  }

  pub fn from_sbc_files<P: AsRef<Path>>(cube_blocks_search_dir: P, entity_components_file_path: P, localization: &Localization) -> Result<Self> {
    let block_name_excludes = HashSet::from([
      // Small grid storage
      "Weapon Rack",
      "Control Seat",
      "Passenger Seat Offset",
      "Passenger Bench",
      // Large grid storage
      "Armory",
      "Armory Lockers",
      "Lockers",
      "Large Industrial Cargo Container",
      "Weapon Rack",
      "Control Station",
      // Small and large grid storage
      "Passenger Seat",
      // Small and large grid thrusters
      "Industrial Large Hydrogen Thruster",
      "Industrial Hydrogen Thruster",
      "Sci-Fi Ion Thruster",
      "Sci-Fi Large Ion Thruster",
      "Sci-Fi Atmospheric Thruster",
      "Sci-Fi Large Atmospheric Thruster",
      "Warfare Ion Thruster",
      "Large Warfare Ion Thruster",
      "Small Warfare Reactor",
      "Large Warfare Reactor",
      "Warfare Battery",
      "Industrial Hydrogen Tank",
    ]);

    let entity_components_file_path = entity_components_file_path.as_ref();
    let entity_components_string = read_string_from_file(entity_components_file_path)
      .map_err(|source| Error::ReadEntityComponentsFile { file: entity_components_file_path.to_path_buf(), source })?;
    let entity_components_doc = Document::parse(&entity_components_string)
      .map_err(|source| Error::ParseEntityComponentsFile { file: entity_components_file_path.to_path_buf(), source })?;
    let entity_components_root_node = entity_components_doc.root().first_element_child()
      .ok_or(Error::XmlStructure(Backtrace::capture()))?;
    let entity_components_node = entity_components_root_node.child_elem("EntityComponents")
      .ok_or(Error::XmlStructure(Backtrace::capture()))?;

    let mut batteries: Vec<Block<Battery>> = Vec::new();
    let mut thrusters: Vec<Block<Thruster>> = Vec::new();
    let mut wheel_suspensions: Vec<Block<WheelSuspension>> = Vec::new();
    let mut hydrogen_engines: Vec<Block<HydrogenEngine>> = Vec::new();
    let mut reactors: Vec<Block<Reactor>> = Vec::new();
    let mut generators: Vec<Block<Generator>> = Vec::new();
    let mut hydrogen_tanks: Vec<Block<HydrogenTank>> = Vec::new();
    let mut containers: Vec<Block<Container>> = Vec::new();
    let mut connectors: Vec<Block<Connector>> = Vec::new();
    let mut cockpits: Vec<Block<Cockpit>> = Vec::new();
    let mut drills: Vec<Block<Drill>> = Vec::new();

    let mut id = 0;
    let cube_blocks_file_paths = WalkDir::new(cube_blocks_search_dir)
      .into_iter()
      .filter_map(|de| {
        if let Ok(de) = de {
          let path = de.into_path();
          if !path.extension().map_or(false, |e| e == "sbc") { return None; }
          if !path.file_name().map_or(false, |n| n.to_string_lossy().contains("CubeBlocks")) { return None; }
          Some(path)
        } else {
          None
        }
      });
    for cube_blocks_file_path in cube_blocks_file_paths {
      let cube_blocks_file_path = &cube_blocks_file_path;
      let cube_blocks_string = read_string_from_file(cube_blocks_file_path)
        .map_err(|source| Error::ReadCubeBlocksFile { file: cube_blocks_file_path.to_path_buf(), source })?;
      let cube_blocks_doc = Document::parse(&cube_blocks_string)
        .map_err(|source| Error::ParseCubeBlocksFile { file: cube_blocks_file_path.to_path_buf(), source })?;
      let definitions_node = cube_blocks_doc.root()
        .first_element_child().ok_or(Error::XmlStructure(Backtrace::capture()))?
        .first_element_child().ok_or(Error::XmlStructure(Backtrace::capture()))?;
      for def in definitions_node.children_elems("Definition") {
        if let Some(ty) = def.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
          match ty {
            "MyObjectBuilder_BatteryBlockDefinition" => {
              let block = Block::<Battery>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                batteries.push(block);
              }
            }
            "MyObjectBuilder_ThrustDefinition" => {
              let block = Block::<Thruster>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                thrusters.push(block);
              }
            }
            "MyObjectBuilder_MotorSuspensionDefinition" => {
              let block = Block::<WheelSuspension>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                wheel_suspensions.push(block);
              }
            }
            "MyObjectBuilder_HydrogenEngineDefinition" => {
              let block = Block::<HydrogenEngine>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                hydrogen_engines.push(block);
              }
            }
            "MyObjectBuilder_ReactorDefinition" => {
              let block = Block::<Reactor>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                reactors.push(block);
              }
            }
            "MyObjectBuilder_OxygenGeneratorDefinition" => {
              let block = Block::<Generator>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                generators.push(block);
              }
            }
            "MyObjectBuilder_GasTankDefinition" => {
              if def.child_elem("StoredGasId").unwrap().parse_child_elem::<String>("SubtypeId").unwrap().unwrap() != "Hydrogen".to_owned() { continue }
              let block = Block::<HydrogenTank>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                hydrogen_tanks.push(block);
              }
            }
            "MyObjectBuilder_CargoContainerDefinition" => {
              let block = Block::<Container>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                containers.push(block);
              }
            }
            "MyObjectBuilder_ShipConnectorDefinition" => {
              let block = Block::<Connector>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                connectors.push(block);
              }
            }
            "MyObjectBuilder_CockpitDefinition" => {
              let block = Block::<Cockpit>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                cockpits.push(block);
              }
            }
            "MyObjectBuilder_ShipDrillDefinition" => {
              let block = Block::<Drill>::from_def(&def, &entity_components_node, id);
              if !block_name_excludes.contains(block.name(localization)) {
                drills.push(block);
              }
            }
            _ => {}
          }
        }
        id += 1;
      }
    }

    fn sort_block_vec<T>(vec: &mut Vec<Block<T>>, localization: &Localization) {
      vec.sort_by_key(|b| b.name(localization).to_string());
    }
    sort_block_vec(&mut batteries, localization);
    sort_block_vec(&mut thrusters, localization);
    sort_block_vec(&mut wheel_suspensions, localization);
    sort_block_vec(&mut hydrogen_engines, localization);
    sort_block_vec(&mut reactors, localization);
    sort_block_vec(&mut generators, localization);
    sort_block_vec(&mut hydrogen_tanks, localization);
    sort_block_vec(&mut containers, localization);
    sort_block_vec(&mut connectors, localization);
    sort_block_vec(&mut cockpits, localization);
    sort_block_vec(&mut drills, localization);
    fn create_map<T>(vec: Vec<Block<T>>) -> LinkedHashMap<BlockId, Block<T>> {
      LinkedHashMap::from_iter(vec.into_iter().map(|b| (b.id.clone(), b)))
    }
    let blocks = Blocks {
      batteries: create_map(batteries),
      thrusters: create_map(thrusters),
      wheel_suspensions: create_map(wheel_suspensions),
      hydrogen_engines: create_map(hydrogen_engines),
      reactors: create_map(reactors),
      generators: create_map(generators),
      hydrogen_tanks: create_map(hydrogen_tanks),
      containers: create_map(containers),
      connectors: create_map(connectors),
      cockpits: create_map(cockpits),
      drills: create_map(drills),
    };
    Ok(blocks)
  }

  pub fn small_and_large_sorted<'a, T, I: Iterator<Item=&'a Block<T>>>(iter: I) -> (Vec<&'a Block<T>>, Vec<&'a Block<T>>) {
    let mut small_vec = Vec::new();
    let mut large_vec = Vec::new();
    for block in iter {
      match block.size {
        GridSize::Small => small_vec.push(block),
        GridSize::Large => large_vec.push(block),
      }
    }
    small_vec.sort();
    large_vec.sort();
    (small_vec, large_vec)
  }
}


