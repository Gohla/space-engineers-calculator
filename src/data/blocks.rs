use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::path::Path;

use roxmltree::{Document, Node};

use super::components::Components;
use super::gas_properties::GasProperties;
use super::localization::Localization;
use super::xml::{NodeExt, read_string_from_file};

/// Grid type.
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum GridType {
  Small,
  Large
}

impl GridType {
  pub fn from_def(def: &Node) -> Self {
    match def.child_elem("CubeSize").unwrap().text().unwrap() {
      "Small" => GridType::Small,
      "Large" => GridType::Large,
      t => panic!("Unrecognized grid size {}", t),
    }
  }
}


/// Data which can be created from a definition in a SBC XML file.
pub trait FromDef: Clone + Debug {
  fn from_def(def: &Node, entity_components: &Node) -> Self;
}


/// Common block data which can be created from a definition in a SBC XML file.
#[derive(Clone, Debug)]
pub struct Block<T> {
  pub id: u64,
  pub name: String,
  pub grid_type: GridType,
  pub components: HashMap<String, f64>,
  pub has_physics: bool,
  pub details: T,
}

impl<T> Block<T> {
  pub fn name<'a>(&'a self, localization: &'a Localization) -> &'a str {
    localization.get(&self.name).unwrap_or(&self.name)
  }

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
  pub fn from_def(def: &Node, entity_components: &Node, id: u64) -> Self {
    let name = def.parse_child_elem("DisplayName").unwrap().unwrap();
    let mut components = HashMap::new();
    let grid_type = GridType::from_def(def);
    for component in def.child_elem("Components").unwrap().children_elems("Component") {
      if let (Some(component_id), Some(count)) = (component.parse_attribute("Subtype").unwrap(), component.parse_attribute("Count").unwrap()) {
        components.entry(component_id).and_modify(|c| *c += count).or_insert(count);
      }
    }
    let has_physics = def.parse_child_elem("HasPhysics").unwrap().unwrap_or(true);
    let details = T::from_def(def, entity_components);
    Block { id, name, grid_type, components, has_physics, details }
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
#[derive(Copy, Clone, Debug)]
pub struct Battery {
  /// Power capacity (MWh)
  pub capacity: f64,
  /// Maximum power input (MW)
  pub input: f64,
  /// Maximum power output (MW)
  pub output: f64,
}

impl FromDef for Battery {
  fn from_def(def: &Node, _entity_components: &Node) -> Self {
    let capacity: f64 = def.parse_child_elem("MaxStoredPower").unwrap().unwrap();
    let input: f64 = def.parse_child_elem("RequiredPowerInput").unwrap().unwrap();
    let output: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    Battery { capacity, input, output }
  }
}


/// Type of thruster
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
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
#[derive(Clone, Debug)]
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
  fn from_def(def: &Node, _entity_components: &Node) -> Self {
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


/// Hydrogen engine.
#[derive(Clone, Debug)]
pub struct HydrogenEngine {
  /// Fuel capacity (L)
  pub fuel_capacity: f64,
  /// Maximum power generation (MW)
  pub max_power_generation: f64,
  /// Maximum fuel consumption (L/s)
  pub max_fuel_consumption: f64,
}

impl FromDef for HydrogenEngine {
  fn from_def(def: &Node, _entity_components: &Node) -> Self {
    let fuel_capacity: f64 = def.parse_child_elem("FuelCapacity").unwrap().unwrap();
    let max_power_generation: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    // HACK: hardcoded fuel consumption.
    let max_fuel_consumption: f64 = match fuel_capacity as u64 {
      16000 => 100.0, // Small grid
      500000 => 1000.0, // Large grid
      _ => panic!("Unrecognized hydrogen engine {:?}", def),
    };
    HydrogenEngine { fuel_capacity, max_power_generation, max_fuel_consumption }
  }
}


/// Reactor
#[derive(Clone, Debug)]
pub struct Reactor {
  /// Maximum power generation (MW)
  pub max_power_generation: f64,
  /// Maximum fuel usage (#/s)
  pub max_fuel_consumption: f64,
}

impl FromDef for Reactor {
  fn from_def(def: &Node, _entity_components: &Node) -> Self {
    let max_power_generation: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    // HACK: hardcoded fuel consumption.
    let max_fuel_consumption: f64 = match max_power_generation as u64 {
      0 => 0.00014, // Small grid, small reactor
      14 => 0.0041, // Small grid, large reactor
      15 => 0.00417, // Large grid, small reactor
      300 => 0.08333, // Large grid, large reactor,
      _ => panic!("Unrecognized reactor {:?}", def),
    };
    Reactor { max_power_generation, max_fuel_consumption }
  }
}


/// Generator (O2/H2)
#[derive(Clone, Debug)]
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
  fn from_def(def: &Node, _entity_components: &Node) -> Self {
    let ice_consumption: f64 = def.parse_child_elem("IceConsumptionPerSecond").unwrap().unwrap();
    let inventory_volume_ice: f64 = def.parse_child_elem::<f64>("InventoryMaxVolume").unwrap().unwrap() * 1000.0;
    let operational_power_consumption: f64 = def.parse_child_elem("OperationalPowerConsumption").unwrap().unwrap();
    let idle_power_consumption: f64 = def.parse_child_elem("StandbyPowerConsumption").unwrap().unwrap();
    let mut oxygen_generation = 100.0;
    let mut hydrogen_generation = 100.0;
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
#[derive(Clone, Debug)]
pub struct HydrogenTank {
  /// Hydrogen capacity (L)
  pub capacity: f64,
  /// Operational power consumption (MW)
  pub operational_power_consumption: f64,
  /// Idle power consumption (MW)
  pub idle_power_consumption: f64,
}

impl FromDef for HydrogenTank {
  fn from_def(def: &Node, _entity_components: &Node) -> Self {
    let capacity: f64 = def.parse_child_elem("Capacity").unwrap().unwrap();
    let operational_power_consumption: f64 = def.parse_child_elem("OperationalPowerConsumption").unwrap().unwrap();
    let idle_power_consumption: f64 = def.parse_child_elem("StandbyPowerConsumption").unwrap().unwrap();
    HydrogenTank { capacity, operational_power_consumption, idle_power_consumption }
  }
}


/// Container
#[derive(Clone, Debug)]
pub struct Container {
  /// Inventory capacity (L)
  pub capacity: f64,
  /// Stores any item?
  pub store_any: bool,
}

impl FromDef for Container {
  fn from_def(def: &Node, entity_components: &Node) -> Self {
    let subtype_id: String = def.child_elem("Id").unwrap().parse_child_elem("SubtypeId").unwrap().unwrap();
    let mut capacity = 0.0;
    let mut store_any = false;
    for entity_component in entity_components.children_elems("EntityComponent") {
      if let Some("MyObjectBuilder_InventoryComponentDefinition") = entity_component.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
        let entity_component_subtype_id: String = entity_component.child_elem("Id").unwrap().parse_child_elem("SubtypeId").unwrap().unwrap();
        if subtype_id != entity_component_subtype_id { continue }
        let size = entity_component.child_elem("Size").unwrap();
        let x: f64 = size.parse_attribute("x").unwrap().unwrap();
        let y: f64 = size.parse_attribute("x").unwrap().unwrap();
        let z: f64 = size.parse_attribute("x").unwrap().unwrap();
        capacity = x * y * z * 1000.0;
        store_any = entity_component.child_elem("InputConstraint").map_or(true, |_| false);
        break;
      }
    }
    if capacity == 0.0 {
      panic!("Unrecognized container {:?}", def);
    }
    Container { capacity, store_any }
  }
}


#[derive(Clone, Debug)]
pub struct Cockpit {
  /// Whether 'cockpit' has an inventory.
  pub has_inventory: bool,
  /// Inventory capacity (L)
  pub capacity: f64,
}

impl FromDef for Cockpit {
  fn from_def(def: &Node, _entity_components: &Node) -> Self {
    let has_inventory = def.parse_child_elem("HasInventory").unwrap().unwrap_or(true);
    // HACK: hardcoded inventory capacity.
    let capacity = if has_inventory { 1000.0 } else { 0.0 };
    Cockpit { has_inventory, capacity }
  }
}


#[derive(Clone, Debug)]
pub struct Blocks {
  pub batteries: HashMap<u64, Block<Battery>>,
  pub thrusters: HashMap<u64, Block<Thruster>>,
  pub hydrogen_engines: HashMap<u64, Block<HydrogenEngine>>,
  pub reactors: HashMap<u64, Block<Reactor>>,
  pub generators: HashMap<u64, Block<Generator>>,
  pub hydrogen_tanks: HashMap<u64, Block<HydrogenTank>>,
  pub containers: HashMap<u64, Block<Container>>,
  pub cockpits: HashMap<u64, Block<Cockpit>>,
}

impl Blocks {
  pub fn from_se_dir<P: AsRef<Path>>(se_dir: P) -> Self {
    Self::from_sbc_files(se_dir.as_ref().join("Content/Data/CubeBlocks.sbc"), se_dir.as_ref().join("Content/Data/EntityComponents.sbc"))
  }

  pub fn from_sbc_files<P: AsRef<Path>>(cubeblocks_sbc_path: P, entitycomponents_sbc_path: P) -> Self {
    let cubeblocks_string = read_string_from_file(cubeblocks_sbc_path).unwrap();
    let cubeblocks_doc = Document::parse(&cubeblocks_string).unwrap();

    let entitycomponents_string = read_string_from_file(entitycomponents_sbc_path).unwrap();
    let entitycomponents_doc = Document::parse(&entitycomponents_string).unwrap();
    let entitycomponents_root_node = entitycomponents_doc.root().first_element_child().unwrap();
    let entitycomponents_node = entitycomponents_root_node.child_elem("EntityComponents").unwrap();

    let mut batteries = HashMap::new();
    let mut thrusters = HashMap::new();
    let mut hydrogen_engines = HashMap::new();
    let mut reactors = HashMap::new();
    let mut generators = HashMap::new();
    let mut hydrogen_tanks = HashMap::new();
    let mut containers = HashMap::new();
    let mut cockpits = HashMap::new();

    let mut id = 0;
    for def in cubeblocks_doc.root().first_element_child().unwrap().first_element_child().unwrap().children_elems("Definition") {
      if let Some(ty) = def.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
        match ty {
          "MyObjectBuilder_BatteryBlockDefinition" => {
            let block = Block::<Battery>::from_def(&def, &entitycomponents_node, id);
            batteries.insert(block.id, block);
          }
          "MyObjectBuilder_ThrustDefinition" => {
            let block = Block::<Thruster>::from_def(&def, &entitycomponents_node, id);
            thrusters.insert(block.id, block);
          }
          "MyObjectBuilder_HydrogenEngineDefinition" => {
            let block = Block::<HydrogenEngine>::from_def(&def, &entitycomponents_node, id);
            hydrogen_engines.insert(block.id, block);
          }
          "MyObjectBuilder_ReactorDefinition" => {
            let block = Block::<Reactor>::from_def(&def, &entitycomponents_node, id);
            reactors.insert(block.id, block);
          }
          "MyObjectBuilder_OxygenGeneratorDefinition" => {
            let block = Block::<Generator>::from_def(&def, &entitycomponents_node, id);
            generators.insert(block.id, block);
          }
          "MyObjectBuilder_GasTankDefinition" => {
            if def.child_elem("StoredGasId").unwrap().parse_child_elem::<String>("SubtypeId").unwrap().unwrap() != "Hydrogen".to_owned() { continue }
            let block = Block::<HydrogenTank>::from_def(&def, &entitycomponents_node, id);
            hydrogen_tanks.insert(block.id, block);
          }
          "MyObjectBuilder_CargoContainerDefinition" => {
            let block = Block::<Container>::from_def(&def, &entitycomponents_node, id);
            containers.insert(block.id, block);
          }
          "MyObjectBuilder_CockpitDefinition" => {
            let block = Block::<Cockpit>::from_def(&def, &entitycomponents_node, id);
            cockpits.insert(block.id, block);
          }
          _ => {}
        }
      }
      id += 1;
    }

    Self { batteries, thrusters, hydrogen_engines, reactors, generators, hydrogen_tanks, containers, cockpits }
  }

  pub fn small_and_large_sorted<'a, T, I: Iterator<Item=&'a Block<T>>>(iter: I) -> (Vec<&'a Block<T>>, Vec<&'a Block<T>>) {
    let mut small_vec = Vec::new();
    let mut large_vec = Vec::new();
    for block in iter {
      match block.grid_type {
        GridType::Small => small_vec.push(block),
        GridType::Large => large_vec.push(block),
      }
    }
    small_vec.sort();
    large_vec.sort();
    (small_vec, large_vec)
  }
}


