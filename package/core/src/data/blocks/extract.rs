use std::backtrace::Backtrace;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use hashlink::LinkedHashMap;
use regex::{Regex, RegexSet};
use roxmltree::{Document, Node};
use thiserror::Error;
use walkdir::WalkDir;

use crate::data::blocks::*;
use crate::xml::{NodeExt, read_string_from_file, XmlError};

// Block definition

impl BlockData {
  pub fn from_def(
    def: &Node,
    localization: &Localization,
    mod_id: Option<u64>,
    hide_block_by_exact_name: &HashSet<String>,
    hide_block_by_regex_name: &RegexSet,
    hide_block_by_exact_subtype_id: &HashSet<String>,
    hide_block_by_regex_subtype_id: &RegexSet,
    hide_block_by_exact_id: &HashSet<String>,
    hide_block_by_regex_id: &RegexSet,
    rename_block_by_regex: &[(Regex, String)]
  ) -> Result<Self, XmlError> {
    let id_node = def.child_elem("Id")?;
    let type_id: String = id_node.parse_child_elem("TypeId")?;
    let subtype_id: String = id_node.parse_child_elem_opt("SubtypeId")?.unwrap_or_default();
    let id = if let Some(mod_id) = mod_id {
      format!("{}.{}@{}", type_id, subtype_id, mod_id)
    } else {
      format!("{}.{}", type_id, subtype_id)
    };
    let name: String = def.parse_child_elem("DisplayName")?;
    let mut components = LinkedHashMap::new();
    let size = GridSize::from_def(def)?;
    for component in def.child_elem("Components")?.children_elems("Component") {
      let component_id = component.parse_attribute("Subtype")?;
      let count: f64 = component.parse_attribute("Count")?;
      *components.entry(component_id).or_insert(0.0) += count;
    }
    let has_physics = def.parse_child_elem_opt("HasPhysics")?.unwrap_or(true);

    let localized_name = localization.get(&name);
    let public = def.child_elem_opt("Public").and_then(|n| n.text().map(|t| t.parse::<bool>().unwrap_or(true))).unwrap_or(true);
    let hidden = if !public {
      true
    } else {
      Self::is_hidden(localized_name, hide_block_by_exact_name, hide_block_by_regex_name)
        || Self::is_hidden(&subtype_id, hide_block_by_exact_subtype_id, hide_block_by_regex_subtype_id)
        || Self::is_hidden(&id, hide_block_by_exact_id, hide_block_by_regex_id)
    };
    let rename = Self::rename(localized_name, rename_block_by_regex);

    Ok(BlockData { id, name, size, components, has_physics, mod_id, hidden, rename })
  }

  fn is_hidden(name: &str, hide_block_by_exact_name: &HashSet<String>, hide_block_by_regex_name: &RegexSet) -> bool {
    if hide_block_by_exact_name.contains(name) { return true; }
    return hide_block_by_regex_name.is_match(name);
  }

  fn rename(name: &str, rename_block_by_regex: &[(Regex, String)]) -> Option<String> {
    for (regex, replacement) in rename_block_by_regex {
      if regex.is_match(name) {
        return Some(regex.replace_all(name, replacement).to_string())
      }
    }
    None
  }
}

impl GridSize {
  pub fn from_def(def: &Node) -> Result<Self, XmlError> {
    let size = match def.child_elem("CubeSize")?.text_or_err()? {
      "Small" => GridSize::Small,
      "Large" => GridSize::Large,
      t => panic!("Unrecognized grid size {}", t),
    };
    Ok(size)
  }
}


// Block detail definitions

/// Some calculated volumes are multiplied by this number.
pub const VOLUME_MULTIPLIER: f64 = 1000.0;

/// Default FuelProductionToCapacityMultiplier in SE's code.
pub const DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER: f64 = 3600.0;

impl Battery {
  pub fn from_def(def: &Node) -> Result<Self, XmlError> {
    let capacity = def.parse_child_elem("MaxStoredPower")?;
    let input = def.parse_child_elem("RequiredPowerInput")?;
    let output = def.parse_child_elem("MaxPowerOutput")?;
    Ok(Self { capacity, input, output })
  }
}

impl JumpDrive {
  pub fn from_def(def: &Node) -> Result<Self, XmlError> { // Defaults according to MyObjectBuilder_JumpDriveDefinition.cs
    let capacity = def.parse_child_elem_opt("PowerNeededForJump")?.unwrap_or(1.0);
    let operational_power_consumption = def.parse_child_elem_opt("RequiredPowerInput")?.unwrap_or(4.0);
    let power_efficiency = def.parse_child_elem_opt("PowerEfficiency")?.unwrap_or(0.8);
    let max_jump_distance = def.parse_child_elem_opt("MaxJumpDistance")?.unwrap_or(5000.0);
    let max_jump_mass = def.parse_child_elem_opt("MaxJumpMass")?.unwrap_or(1250000.0);
    Ok(Self { capacity, operational_power_consumption, power_efficiency, max_jump_distance, max_jump_mass })
  }
}

impl Railgun {
  pub fn from_def(def: &Node, entity_components: &Node) -> Result<Self, XmlError> {
    let mut capacity = None;
    let mut operational_power_consumption = None;
    let subtype_id: String = def.child_elem("Id")?.parse_child_elem("SubtypeId")?;
    for entity_component in entity_components.children_elems("EntityComponent") {
      if let Some("MyObjectBuilder_EntityCapacitorComponentDefinition") = entity_component.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
        let entity_component_subtype_id: String = entity_component.child_elem("Id")?.parse_child_elem("SubtypeId")?;
        if subtype_id != entity_component_subtype_id { continue }
        capacity = Some(entity_component.parse_child_elem("Capacity")?);
        operational_power_consumption = Some(entity_component.parse_child_elem("RechargeDraw")?);
        break;
      }
    }
    if let (Some(capacity), Some(operational_power_consumption)) = (capacity, operational_power_consumption) {
      let idle_power_consumption = 0.0002; // According to MySmallMissileLauncher.cs
      Ok(Self { capacity, operational_power_consumption, idle_power_consumption })
    } else {
      Err(XmlError::StructureFail(Backtrace::capture()))
    }
  }
}

impl ThrusterType {
  pub fn from_def(def: &Node) -> Result<Self, XmlError> {
    let ty = match def.child_elem("ThrusterType")?.text_or_err()? {
      "Ion" => ThrusterType::Ion,
      "Atmospheric" => ThrusterType::Atmospheric,
      "Hydrogen" => ThrusterType::Hydrogen,
      t => panic!("Unrecognized thruster type {}", t),
    };
    Ok(ty)
  }
}

impl Thruster {
  fn from_def(def: &Node) -> Result<Self, XmlError> {
    let ty = ThrusterType::from_def(def)?;
    let force = def.parse_child_elem("ForceMagnitude")?;
    let fuel_gas_id = if let Some(node) = def.child_elem_opt("FuelConverter") {
      Some(node.first_child_elem()?.parse_child_elem("SubtypeId")?)
    } else {
      None
    };
    let max_consumption = def.parse_child_elem("MaxPowerConsumption")?;
    let min_consumption = def.parse_child_elem("MinPowerConsumption")?;
    let min_planetary_influence = def.parse_child_elem_opt("MinPlanetaryInfluence")?.unwrap_or(0.0);
    let max_planetary_influence = def.parse_child_elem_opt("MaxPlanetaryInfluence")?.unwrap_or(1.0);
    let effectiveness_at_min_influence = def.parse_child_elem_opt("EffectivenessAtMinInfluence")?.unwrap_or(1.0);
    let effectiveness_at_max_influence = def.parse_child_elem_opt("EffectivenessAtMaxInfluence")?.unwrap_or(1.0);
    let needs_atmosphere_for_influence = def.parse_child_elem_opt("NeedsAtmosphereForInfluence")?.unwrap_or(false);
    Ok(Thruster {
      ty,
      fuel_gas_id,
      force,
      max_consumption,
      min_consumption,
      min_planetary_influence,
      max_planetary_influence,
      effectiveness_at_min_influence,
      effectiveness_at_max_influence,
      needs_atmosphere_for_influence
    })
  }
}

impl WheelSuspension {
  fn from_def(def: &Node) -> Result<Self, XmlError> {
    let force = def.parse_child_elem("PropulsionForce")?;
    let operational_power_consumption = def.parse_child_elem("RequiredPowerInput")?;
    let idle_power_consumption = def.parse_child_elem("RequiredIdlePowerInput")?;
    Ok(Self { force, operational_power_consumption, idle_power_consumption })
  }
}

impl HydrogenEngine {
  fn from_def(def: &Node) -> Result<Self, XmlError> {
    let fuel_capacity = def.parse_child_elem("FuelCapacity")?;
    let max_power_generation = def.parse_child_elem("MaxPowerOutput")?;
    let fuel_production_to_capacity_multiplier = def.parse_child_elem_opt("FuelProductionToCapacityMultiplier")?.unwrap_or(DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER);
    let max_fuel_consumption = max_power_generation / fuel_production_to_capacity_multiplier;
    Ok(Self { fuel_capacity, max_power_generation, max_fuel_consumption })
  }
}

impl Reactor {
  fn from_def(def: &Node) -> Result<Self, XmlError> {
    let max_power_generation = def.parse_child_elem("MaxPowerOutput")?;
    let fuel_production_to_capacity_multiplier = def.parse_child_elem_opt("FuelProductionToCapacityMultiplier")?.unwrap_or(DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER);
    let max_fuel_consumption = max_power_generation / fuel_production_to_capacity_multiplier;
    Ok(Self { max_power_generation, max_fuel_consumption })
  }
}

impl Generator {
  fn from_def(def: &Node) -> Result<Self, XmlError> {
    let ice_consumption = def.parse_child_elem("IceConsumptionPerSecond")?;
    let inventory_volume_ice = def.parse_child_elem::<f64>("InventoryMaxVolume")? * VOLUME_MULTIPLIER;
    let operational_power_consumption = def.parse_child_elem("OperationalPowerConsumption")?;
    let idle_power_consumption = def.parse_child_elem("StandbyPowerConsumption")?;
    let mut oxygen_generation = 0.0;
    let mut hydrogen_generation = 0.0;
    for gas_info in def.child_elem("ProducedGases")?.children_elems("GasInfo") {
      let gas_id: String = gas_info.child_elem("Id")?.parse_child_elem("SubtypeId")?;
      let ice_to_gas_ratio: f64 = gas_info.parse_child_elem("IceToGasRatio")?;
      let gas_generation = ice_consumption * ice_to_gas_ratio;
      *(match gas_id.as_ref() {
        "Oxygen" => &mut oxygen_generation,
        "Hydrogen" => &mut hydrogen_generation,
        _ => panic!("Unrecognized gas ID {} in generator {:?}", gas_id, def),
      }) = gas_generation;
    }
    Ok(Self {
      ice_consumption,
      inventory_volume_ice,
      operational_power_consumption,
      idle_power_consumption,
      oxygen_generation,
      hydrogen_generation
    })
  }
}

impl HydrogenTank {
  fn from_def(def: &Node) -> Result<Self, XmlError> {
    let capacity = def.parse_child_elem("Capacity")?;
    let operational_power_consumption = def.parse_child_elem("OperationalPowerConsumption")?;
    let idle_power_consumption = def.parse_child_elem("StandbyPowerConsumption")?;
    Ok(Self { capacity, operational_power_consumption, idle_power_consumption })
  }
}

impl Container {
  fn from_def(def: &Node, entity_components: &Node) -> Result<Self, XmlError> {
    let subtype_id: String = def.child_elem("Id")?.parse_child_elem("SubtypeId")?;
    let mut inventory_volume_any = None;
    let mut store_any = None;
    for entity_component in entity_components.children_elems("EntityComponent") {
      if let Some("MyObjectBuilder_InventoryComponentDefinition") = entity_component.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
        let entity_component_subtype_id: String = entity_component.child_elem("Id")?.parse_child_elem("SubtypeId")?;
        if subtype_id != entity_component_subtype_id { continue }
        let size = entity_component.child_elem("Size")?;
        let x: f64 = size.parse_attribute("x")?;
        let y: f64 = size.parse_attribute("y")?;
        let z: f64 = size.parse_attribute("z")?;
        inventory_volume_any = Some(x * y * z * VOLUME_MULTIPLIER);
        store_any = Some(entity_component.child_elem_opt("InputConstraint").map_or(true, |_| false));
        break;
      }
    }
    if let (Some(inventory_volume_any), Some(store_any)) = (inventory_volume_any, store_any) {
      Ok(Self { inventory_volume_any, store_any })
    } else {
      Err(XmlError::StructureFail(Backtrace::capture()))
    }
  }
}

impl Connector {
  fn from_def(def: &Node, data: &BlockData) -> Result<Self, XmlError> {
    let size = def.child_elem("Size")?;
    let x: f64 = size.parse_attribute("x")?;
    let y: f64 = size.parse_attribute("y")?;
    let z: f64 = size.parse_attribute("z")?;
    let multiplier = data.size.size() * 0.8;
    let inventory_volume_any = (x * multiplier) * (y * multiplier) * (z * multiplier) * VOLUME_MULTIPLIER; // Inventory capacity according to MyShipConnector.cs.
    Ok(Self { inventory_volume_any, })
  }
}

impl Cockpit {
  fn from_def(def: &Node) -> Result<Self, XmlError> {
    let has_inventory = def.parse_child_elem_opt("HasInventory")?.unwrap_or(true);
    let inventory_volume_any = if has_inventory { VOLUME_MULTIPLIER } else { 0.0 }; // Inventory capacity according to MyCockpit.cs.
    Ok(Self { has_inventory, inventory_volume_any })
  }
}


impl Drill {
  fn from_def(def: &Node, data: &BlockData) -> Result<Self, XmlError> {
    let size = def.child_elem("Size")?;
    let x: f64 = size.parse_attribute("x")?;
    let y: f64 = size.parse_attribute("y")?;
    let z: f64 = size.parse_attribute("z")?;
    let cube_size = data.size.size();
    let inventory_volume_ore = x * y * z * cube_size * cube_size * cube_size * 0.5 * VOLUME_MULTIPLIER; // Inventory capacity according to MyShipDrill.cs.
    let operational_power_consumption = 1.0 / 500.0 * 1.0; // Maximum required power according to ComputeMaxRequiredPower in MyShipDrill.cs.
    let idle_power_consumption = 1e-06; // Idle power according to ComputeMaxRequiredPower in MyShipDrill.cs.
    Ok(Self { inventory_volume_ore, operational_power_consumption, idle_power_consumption })
  }
}


// All block definitions

pub struct BlocksBuilder {
  hide_block_by_exact_name: HashSet<String>,
  hide_block_by_regex_name: RegexSet,
  hide_block_by_exact_subtype_id: HashSet<String>,
  hide_block_by_regex_subtype_id: RegexSet,
  hide_block_by_exact_id: HashSet<String>,
  hide_block_by_regex_id: RegexSet,
  rename_block_by_regex: Vec<(Regex, String)>,

  batteries: Vec<Block<Battery>>,
  jump_drives: Vec<Block<JumpDrive>>,
  railguns: Vec<Block<Railgun>>,
  thrusters: Vec<Block<Thruster>>,
  wheel_suspensions: Vec<Block<WheelSuspension>>,
  hydrogen_engines: Vec<Block<HydrogenEngine>>,
  reactors: Vec<Block<Reactor>>,
  generators: Vec<Block<Generator>>,
  hydrogen_tanks: Vec<Block<HydrogenTank>>,
  containers: Vec<Block<Container>>,
  connectors: Vec<Block<Connector>>,
  cockpits: Vec<Block<Cockpit>>,
  drills: Vec<Block<Drill>>,
}

#[derive(Error, Debug)]
pub enum CreateError {
  #[error(transparent)]
  XmlError(#[from] regex::Error),
}

impl BlocksBuilder {
  pub fn new(
    hide_block_by_exact_name: impl Iterator<Item=String>,
    hide_block_by_regex_name: impl Iterator<Item=String>,
    hide_block_by_exact_subtype_id: impl Iterator<Item=String>,
    hide_block_by_regex_subtype_id: impl Iterator<Item=String>,
    hide_block_by_exact_id: impl Iterator<Item=String>,
    hide_block_by_regex_id: impl Iterator<Item=String>,
    rename_block_by_regex: impl Iterator<Item=(String, String)>,
  ) -> Result<Self, CreateError> {
    let hide_block_by_regex_name = RegexSet::new(hide_block_by_regex_name)?;
    let hide_block_by_regex_subtype_id = RegexSet::new(hide_block_by_regex_subtype_id)?;
    let hide_block_by_regex_id = RegexSet::new(hide_block_by_regex_id)?;
    let rename_block_by_regex = {
      let mut renames = Vec::with_capacity(rename_block_by_regex.size_hint().0);
      for (regex, rename) in rename_block_by_regex {
        let regex = Regex::new(&regex)?;
        renames.push((regex, rename));
      }
      renames
    };
    Ok(Self {
      hide_block_by_exact_name: HashSet::from_iter(hide_block_by_exact_name),
      hide_block_by_regex_name,
      hide_block_by_exact_subtype_id: HashSet::from_iter(hide_block_by_exact_subtype_id),
      hide_block_by_regex_subtype_id,
      hide_block_by_exact_id: HashSet::from_iter(hide_block_by_exact_id),
      hide_block_by_regex_id,
      rename_block_by_regex,

      batteries: vec![],
      jump_drives: vec![],
      railguns: vec![],
      thrusters: vec![],
      wheel_suspensions: vec![],
      hydrogen_engines: vec![],
      reactors: vec![],
      generators: vec![],
      hydrogen_tanks: vec![],
      containers: vec![],
      connectors: vec![],
      cockpits: vec![],
      drills: vec![]
    })
  }
}

#[derive(Error, Debug)]
pub enum ExtractError {
  #[error("Could not read CubeBlocks file '{file}'")]
  ReadCubeBlocksFileFail { file: PathBuf, source: std::io::Error },
  #[error("Could not XML parse CubeBlocks file '{file}'")]
  ParseCubeBlocksFileFail { file: PathBuf, source: roxmltree::Error },
  #[error("Could not read EntityComponents file '{file}'")]
  ReadEntityComponentsFileFail { file: PathBuf, source: std::io::Error },
  #[error("Could not XML parse EntityComponents file '{file}'")]
  ParseEntityComponentsFileFail { file: PathBuf, source: roxmltree::Error },
  #[error(transparent)]
  XmlFail {
    #[from]
    source: XmlError
  },
}

impl BlocksBuilder {
  pub fn update_from_se_dir(
    &mut self,
    se_directory: impl AsRef<Path>,
    localization: &Localization
  ) -> Result<(), ExtractError> {
    self.update_from_sbc_files(
      se_directory.as_ref().join("Content/Data/"),
      |path| path.file_name().map_or(false, |n| n.to_string_lossy().contains("CubeBlocks")),
      se_directory.as_ref().join("Content/Data/EntityComponents.sbc"),
      localization,
      None,
    )
  }

  pub fn update_from_mod(
    &mut self,
    se_directory: impl AsRef<Path>,
    se_workshop_directory: impl AsRef<Path>,
    mod_id: u64,
    localization: &Localization
  ) -> Result<(), ExtractError> {
    let search_path = se_workshop_directory.as_ref().join(format!("{}", mod_id));
    self.update_from_sbc_files(
      search_path,
      |_| true,
      se_directory.as_ref().join("Content/Data/EntityComponents.sbc"),
      localization,
      Some(mod_id),
    )
  }

  pub fn update_from_sbc_files(
    &mut self,
    search_path: impl AsRef<Path>,
    search_path_filter: impl Fn(&PathBuf) -> bool,
    entity_components_file: impl AsRef<Path>,
    localization: &Localization,
    mod_id: Option<u64>,
  ) -> Result<(), ExtractError> {
    let entity_components_file = entity_components_file.as_ref();
    let entity_components_string = read_string_from_file(entity_components_file)
      .map_err(|source| ExtractError::ReadEntityComponentsFileFail { file: entity_components_file.to_path_buf(), source })?;
    let entity_components_doc = Document::parse(&entity_components_string)
      .map_err(|source| ExtractError::ParseEntityComponentsFileFail { file: entity_components_file.to_path_buf(), source })?;
    let entity_components_root = entity_components_doc.root();
    let entity_components_root_node = entity_components_root.first_child_elem()?;
    let entity_components_node = entity_components_root_node.child_elem("EntityComponents")?;

    let cube_blocks_file_paths = WalkDir::new(search_path)
      .into_iter()
      .filter_map(|de| {
        if let Ok(de) = de {
          let path = de.into_path();
          if !path.extension().map_or(false, |e| e == "sbc") { return None; }
          if !search_path_filter(&path) { return None; }
          Some(path)
        } else {
          None
        }
      });
    for cube_blocks_file_path in cube_blocks_file_paths {
      let cube_blocks_file_path = &cube_blocks_file_path;
      let cube_blocks_string = read_string_from_file(cube_blocks_file_path)
        .map_err(|source| ExtractError::ReadCubeBlocksFileFail { file: cube_blocks_file_path.to_path_buf(), source })?;
      let cube_blocks_doc = Document::parse(&cube_blocks_string)
        .map_err(|source| ExtractError::ParseCubeBlocksFileFail { file: cube_blocks_file_path.to_path_buf(), source })?;
      let definitions_node = cube_blocks_doc.root();
      let definitions_node = definitions_node.first_child_elem()?;
      let definitions_node = definitions_node.first_child_elem()?;
      for def in definitions_node.children_elems("Definition") {
        let data = BlockData::from_def(
          &def,
          localization,
          mod_id,
          &self.hide_block_by_exact_name,
          &self.hide_block_by_regex_name,
          &self.hide_block_by_exact_subtype_id,
          &self.hide_block_by_regex_subtype_id,
          &self.hide_block_by_exact_id,
          &self.hide_block_by_regex_id,
          &self.rename_block_by_regex,
        )?;
        fn add_block<T>(details: T, data: BlockData, vec: &mut Vec<Block<T>>) {
          let block = Block::new(data, details);
          vec.push(block);
        }
        if let Some(ty) = def.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
          match ty {
            "MyObjectBuilder_BatteryBlockDefinition" => {
              add_block(Battery::from_def(&def)?, data, &mut self.batteries);
            }
            "MyObjectBuilder_JumpDriveDefinition" => {
              add_block(JumpDrive::from_def(&def)?, data, &mut self.jump_drives);
            }
            "MyObjectBuilder_WeaponBlockDefinition" => {
              if data.id.contains("Railgun") {
                add_block(Railgun::from_def(&def, &entity_components_node)?, data, &mut self.railguns);
              }
            }
            "MyObjectBuilder_ThrustDefinition" => {
              add_block(Thruster::from_def(&def)?, data, &mut self.thrusters);
            }
            "MyObjectBuilder_MotorSuspensionDefinition" => {
              add_block(WheelSuspension::from_def(&def)?, data, &mut self.wheel_suspensions);
            }
            "MyObjectBuilder_HydrogenEngineDefinition" => {
              add_block(HydrogenEngine::from_def(&def)?, data, &mut self.hydrogen_engines);
            }
            "MyObjectBuilder_ReactorDefinition" => {
              add_block(Reactor::from_def(&def)?, data, &mut self.reactors);
            }
            "MyObjectBuilder_OxygenGeneratorDefinition" => {
              add_block(Generator::from_def(&def)?, data, &mut self.generators);
            }
            "MyObjectBuilder_GasTankDefinition" => {
              if def.child_elem("StoredGasId")?.parse_child_elem::<String>("SubtypeId")? != "Hydrogen".to_owned() { continue }
              add_block(HydrogenTank::from_def(&def)?, data, &mut self.hydrogen_tanks);
            }
            "MyObjectBuilder_CargoContainerDefinition" => {
              add_block(Container::from_def(&def, &entity_components_node)?, data, &mut self.containers);
            }
            "MyObjectBuilder_ShipConnectorDefinition" => {
              add_block(Connector::from_def(&def, &data)?, data, &mut self.connectors);
            }
            "MyObjectBuilder_CockpitDefinition" => {
              add_block(Cockpit::from_def(&def)?, data, &mut self.cockpits);
            }
            "MyObjectBuilder_ShipDrillDefinition" => {
              add_block(Drill::from_def(&def, &data)?, data, &mut self.drills);
            }
            _ => {}
          }
        }
      }
    }
    Ok(())
  }

  pub fn into_blocks(mut self, localization: &Localization) -> Blocks {
    fn sort_block_vec<T>(vec: &mut Vec<Block<T>>, localization: &Localization) {
      vec.sort_by(|a, b| alphanumeric_sort::compare_str(a.name(localization), b.name(localization)));
    }
    sort_block_vec(&mut self.batteries, localization);
    sort_block_vec(&mut self.jump_drives, localization);
    sort_block_vec(&mut self.railguns, localization);
    sort_block_vec(&mut self.thrusters, localization);
    sort_block_vec(&mut self.wheel_suspensions, localization);
    sort_block_vec(&mut self.hydrogen_engines, localization);
    sort_block_vec(&mut self.reactors, localization);
    sort_block_vec(&mut self.generators, localization);
    sort_block_vec(&mut self.hydrogen_tanks, localization);
    sort_block_vec(&mut self.containers, localization);
    sort_block_vec(&mut self.connectors, localization);
    sort_block_vec(&mut self.cockpits, localization);
    sort_block_vec(&mut self.drills, localization);
    fn create_map<T>(vec: Vec<Block<T>>) -> LinkedHashMap<BlockId, Block<T>> {
      LinkedHashMap::from_iter(vec.into_iter().map(|b| (b.data.id.clone(), b)))
    }
    Blocks {
      batteries: create_map(self.batteries),
      jump_drives: create_map(self.jump_drives),
      railguns: create_map(self.railguns),
      thrusters: create_map(self.thrusters),
      wheel_suspensions: create_map(self.wheel_suspensions),
      hydrogen_engines: create_map(self.hydrogen_engines),
      reactors: create_map(self.reactors),
      generators: create_map(self.generators),
      hydrogen_tanks: create_map(self.hydrogen_tanks),
      containers: create_map(self.containers),
      connectors: create_map(self.connectors),
      cockpits: create_map(self.cockpits),
      drills: create_map(self.drills),
    }
  }
}
