use std::fmt::Debug;
use std::ops::Deref;
use std::path::Path;

use roxmltree::{Document, Node};

use crate::gas_properties::GasProperties;
use crate::localization::Localization;
use crate::xml::{NodeExt, read_string_from_file};

/// Grid type.
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum GridType {
  Small,
  Large
}

impl GridType {
  pub fn from_def(def: &Node) -> Self {
    match def.get_child_elem("CubeSize").unwrap().text().unwrap() {
      "Small" => GridType::Small,
      "Large" => GridType::Large,
      t => panic!("Unrecognized grid size {}", t),
    }
  }
}


/// Data which can be created from a definition in a SBC XML file.
pub trait FromDef: Clone + Debug {
  fn from_def(def: &Node) -> Self;
}


/// Common block data which can be created from a definition in a SBC XML file.
#[derive(Clone, Debug)]
pub struct Block<T> {
  pub name: String,
  pub grid_type: GridType,
  pub mass: u64,
  pub details: T,
}

impl<T> Block<T> {
  pub fn name<'a>(&'a self, localization: &'a Localization) -> &'a str {
    localization.get(&self.name).unwrap_or(&self.name)
  }
}

impl<T: FromDef> Block<T> {
  pub fn from_def(def: &Node) -> Self {
    let name = def.get_child_elem("DisplayName").unwrap().text().unwrap();
    let name = name.to_string();
    let grid_type = GridType::from_def(def);
    let mass = 0; // TODO: parse components and add up to get mass.
    let details = T::from_def(def);
    Block { name, grid_type, mass, details }
  }
}

impl<T: FromDef> Deref for Block<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.details
  }
}


/// Battery block.
#[derive(Copy, Clone, Debug)]
pub struct Battery {
  /// Power storage (MWh)
  pub storage: f64,
  /// Maximum power input (MW)
  pub input: f64,
  /// Maximum power output (MW)
  pub output: f64,
}

impl FromDef for Battery {
  fn from_def(def: &Node) -> Self {
    let storage: f64 = def.parse_child_elem("MaxStoredPower").unwrap().unwrap();
    let input: f64 = def.parse_child_elem("RequiredPowerInput").unwrap().unwrap();
    let output: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    Battery { storage, input, output }
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
    match def.get_child_elem("ThrusterType").unwrap().text().unwrap() {
      "Ion" => ThrusterType::Ion,
      "Atmospheric" => ThrusterType::Atmospheric,
      "Hydrogen" => ThrusterType::Hydrogen,
      t => panic!("Unrecognized thruster type {}", t),
    }
  }
}

/// Thruster block.
#[derive(Clone, Debug)]
pub struct Thruster {
  /// Thruster type
  ty: ThrusterType,
  /// Optional fuel gas ID, used to determine actual consumption.
  fuel_gas_id: Option<String>,
  /// Force (N)
  force: f64,
  /// Maximum consumption (MW for energy-based thrusters, otherwise max_consumption/<energy_density of fuel> L/s for fuel-based thrusters)
  max_consumption: f64,
  /// Minimum consumption (MW for energy-based thrusters, otherwise min_consumption/<energy_density of fuel> L/s for fuel-based thrusters)
  min_consumption: f64,
  min_planetary_influence: f64,
  max_planetary_influence: f64,
  effectiveness_at_min_influence: f64,
  effectiveness_at_max_influence: f64,
  needs_atmosphere_for_influence: bool,
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
  fn from_def(def: &Node) -> Self {
    let ty = ThrusterType::from_def(def);
    let force = def.parse_child_elem("ForceMagnitude").unwrap().unwrap();
    let fuel_gas_id = def.get_child_elem("FuelConverter").map(|n| n.first_element_child().unwrap().parse_child_elem("SubtypeId").unwrap().unwrap());
    let max_consumption = def.parse_child_elem("MaxPowerConsumption").unwrap().unwrap();
    let min_consumption = def.parse_child_elem("MinPowerConsumption").unwrap().unwrap();
    let min_planetary_influence = def.parse_child_elem("MinPlanetaryInfluence").unwrap().unwrap_or(1.0);
    let max_planetary_influence = def.parse_child_elem("MaxPlanetaryInfluence").unwrap().unwrap_or(1.0);
    let effectiveness_at_min_influence = def.parse_child_elem("EffectivenessAtMinInfluence").unwrap().unwrap_or(1.0);
    let effectiveness_at_max_influence = def.parse_child_elem("EffectivenessAtMaxInfluence").unwrap().unwrap_or(1.0);
    let needs_atmosphere_for_influence = def.parse_child_elem("NeedsAtmosphereForInfluence").unwrap().unwrap_or(false);
    Thruster { ty, fuel_gas_id, force, max_consumption, min_consumption, min_planetary_influence, max_planetary_influence, effectiveness_at_min_influence, effectiveness_at_max_influence, needs_atmosphere_for_influence }
  }
}


#[derive(Clone, Debug)]
pub struct Blocks {
  pub batteries: Vec<Block<Battery>>,
  pub thrusters: Vec<Block<Thruster>>,
}

impl Blocks {
  pub fn from_data<P: AsRef<Path>>(cubeblocks_sbc_path: P) -> Blocks {
    let string = read_string_from_file(cubeblocks_sbc_path).unwrap();
    let doc = Document::parse(&string).unwrap();

    let mut batteries = Vec::new();
    let mut thrusters = Vec::new();

    for def in doc.root().first_element_child().unwrap().first_element_child().unwrap().children() {
      if !def.is_element() { continue }
      if let Some(ty) = def.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
        match ty {
          "MyObjectBuilder_BatteryBlockDefinition" => {
            batteries.push(Block::<Battery>::from_def(&def));
          }
          "MyObjectBuilder_ThrustDefinition" => {
            thrusters.push(Block::<Thruster>::from_def(&def));
          }
          _ => {}
        }
      }
    }

    Blocks { batteries, thrusters }
  }
}


