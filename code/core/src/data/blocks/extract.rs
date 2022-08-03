use linked_hash_map::LinkedHashMap;
use regex::Regex;
use roxmltree::Node;

use crate::data::blocks::*;
use crate::data::xml::NodeExt;

// All block definitions

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

impl Blocks {
  pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P, localization: &Localization) -> Result<Self, Error> {
    Self::from_sbc_files(se_dir_path.as_ref().join("Content/Data/"), se_dir_path.as_ref().join("Content/Data/EntityComponents.sbc"), localization)
  }

  pub fn from_sbc_files<P: AsRef<Path>>(cube_blocks_search_dir: P, entity_components_file_path: P, localization: &Localization) -> Result<Self, Error> {
    let hide_block_names = HashSet::from_iter([
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
      "Warfare Battery",
      "Industrial Hydrogen Tank",
    ]);
    let hide_block_regexes = Vec::from_iter([
      // Small and large grid thrusters
      "Industrial .+ Thruster",
      "Sci-Fi .+ Thruster",
      ".*Warfare .+ Thruster",
      ".+ Warfare Reactor",
      // Small and large grid wheel suspensions
      "Offroad Wheel Suspension .+",
      "Wheel Suspension .+ Right",
    ].into_iter().map(|s| Regex::new(s).unwrap()));
    let rename_blocks = Vec::from_iter([
      ("Wheel Suspension (.+) Left", "Wheel Suspension $1"),
    ].into_iter().map(|(r, rp)| (Regex::new(r).unwrap(), rp)));

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

    let mut index = 0;
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
        let data = BlockData::from_def(&def, index, localization, &hide_block_names, &hide_block_regexes, &rename_blocks);
        fn add_block<T>(details: T, data: BlockData, vec: &mut Vec<Block<T>>) {
          let block = Block::new(data, details);
          vec.push(block);
        }
        if let Some(ty) = def.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type")) {
          match ty {
            "MyObjectBuilder_BatteryBlockDefinition" => {
              add_block(Battery::from_def(&def), data, &mut batteries);
            }
            "MyObjectBuilder_ThrustDefinition" => {
              add_block(Thruster::from_def(&def), data, &mut thrusters);
            }
            "MyObjectBuilder_MotorSuspensionDefinition" => {
              add_block(WheelSuspension::from_def(&def), data, &mut wheel_suspensions);
            }
            "MyObjectBuilder_HydrogenEngineDefinition" => {
              add_block(HydrogenEngine::from_def(&def), data, &mut hydrogen_engines);
            }
            "MyObjectBuilder_ReactorDefinition" => {
              add_block(Reactor::from_def(&def), data, &mut reactors);
            }
            "MyObjectBuilder_OxygenGeneratorDefinition" => {
              add_block(Generator::from_def(&def), data, &mut generators);
            }
            "MyObjectBuilder_GasTankDefinition" => {
              if def.child_elem("StoredGasId").unwrap().parse_child_elem::<String>("SubtypeId").unwrap().unwrap() != "Hydrogen".to_owned() { continue }
              add_block(HydrogenTank::from_def(&def), data, &mut hydrogen_tanks);
            }
            "MyObjectBuilder_CargoContainerDefinition" => {
              add_block(Container::from_def(&def, &entity_components_node), data, &mut containers);
            }
            "MyObjectBuilder_ShipConnectorDefinition" => {
              add_block(Connector::from_def(&def, &data), data, &mut connectors);
            }
            "MyObjectBuilder_CockpitDefinition" => {
              add_block(Cockpit::from_def(&def), data, &mut cockpits);
            }
            "MyObjectBuilder_ShipDrillDefinition" => {
              add_block(Drill::from_def(&def, &data), data, &mut drills);
            }
            _ => {}
          }
        }
        index += 1;
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
      LinkedHashMap::from_iter(vec.into_iter().map(|b| (b.data.id.clone(), b)))
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
}


// Block definition

impl BlockData {
  pub fn from_def(
    def: &Node,
    index: u64,
    localization: &Localization,
    hide_block_names: &HashSet<&str>,
    hide_block_regexes: &[Regex],
    rename_blocks: &[(Regex, &str)]
  ) -> Self {
    let id_node = def.child_elem("Id").unwrap();
    let type_id: String = id_node.parse_child_elem("TypeId").unwrap().unwrap();
    let subtype_id = id_node.parse_child_elem("SubtypeId").unwrap().unwrap_or(String::new());
    let id = type_id + "." + &subtype_id;
    let name: String = def.parse_child_elem("DisplayName").unwrap().unwrap();
    let mut components = LinkedHashMap::new();
    let size = GridSize::from_def(def);
    for component in def.child_elem("Components").unwrap().children_elems("Component") {
      if let (Some(component_id), Some(count)) = (component.parse_attribute("Subtype").unwrap(), component.parse_attribute::<f64, _>("Count").unwrap()) {
        *components.entry(component_id).or_insert(0.0) += count;
      }
    }
    let has_physics = def.parse_child_elem("HasPhysics").unwrap().unwrap_or(true);

    let localized_name = localization.get(&name).unwrap_or(&name).as_str();
    let hidden = Self::is_hidden(localized_name, hide_block_names, hide_block_regexes);
    let rename = Self::rename(localized_name, rename_blocks);

    BlockData { id, name, size, components, has_physics, index, hidden, rename }
  }

  fn is_hidden(name: &str, hide_block_names: &HashSet<&str>, hide_block_regexes: &[Regex]) -> bool {
    if hide_block_names.contains(name) { return true; }
    for regex in hide_block_regexes {
      if regex.is_match(name) {
        return true;
      }
    }
    false
  }

  fn rename(name: &str, rename_blocks: &[(Regex, &str)]) -> Option<String> {
    for (regex, replacement) in rename_blocks {
      if regex.is_match(name) {
        return Some(regex.replace_all(name, *replacement).to_string())
      }
    }
    None
  }
}

impl GridSize {
  pub fn from_def(def: &Node) -> Self {
    match def.child_elem("CubeSize").unwrap().text().unwrap() {
      "Small" => GridSize::Small,
      "Large" => GridSize::Large,
      t => panic!("Unrecognized grid size {}", t),
    }
  }
}


// Block detail definitions

/// Some calculated volumes are multiplied by this number.
pub const VOLUME_MULTIPLIER: f64 = 1000.0;

/// Default FuelProductionToCapacityMultiplier in SE's code.
pub const DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER: f64 = 3600.0;

impl Battery {
  pub fn from_def(def: &Node) -> Self {
    let capacity: f64 = def.parse_child_elem("MaxStoredPower").unwrap().unwrap();
    let input: f64 = def.parse_child_elem("RequiredPowerInput").unwrap().unwrap();
    let output: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    Battery { capacity, input, output }
  }
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

impl Thruster {
  fn from_def(def: &Node) -> Self {
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

impl WheelSuspension {
  fn from_def(def: &Node) -> Self {
    let force: f64 = def.parse_child_elem("PropulsionForce").unwrap().unwrap();
    let operational_power_consumption: f64 = def.parse_child_elem("RequiredPowerInput").unwrap().unwrap();
    let idle_power_consumption: f64 = def.parse_child_elem("RequiredIdlePowerInput").unwrap().unwrap();
    Self { force, operational_power_consumption, idle_power_consumption }
  }
}

impl HydrogenEngine {
  fn from_def(def: &Node) -> Self {
    let fuel_capacity: f64 = def.parse_child_elem("FuelCapacity").unwrap().unwrap();
    let max_power_generation: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    let fuel_production_to_capacity_multiplier: f64 = def.parse_child_elem("FuelProductionToCapacityMultiplier").unwrap().unwrap_or(DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER);
    let max_fuel_consumption = max_power_generation / fuel_production_to_capacity_multiplier;
    HydrogenEngine { fuel_capacity, max_power_generation, max_fuel_consumption }
  }
}

impl Reactor {
  fn from_def(def: &Node) -> Self {
    let max_power_generation: f64 = def.parse_child_elem("MaxPowerOutput").unwrap().unwrap();
    let fuel_production_to_capacity_multiplier: f64 = def.parse_child_elem("FuelProductionToCapacityMultiplier").unwrap().unwrap_or(DEFAULT_FUEL_PRODUCTION_TO_CAPACITY_MULTIPLIER);
    let max_fuel_consumption = max_power_generation / fuel_production_to_capacity_multiplier;
    Reactor { max_power_generation, max_fuel_consumption }
  }
}

impl Generator {
  fn from_def(def: &Node) -> Self {
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

impl HydrogenTank {
  fn from_def(def: &Node) -> Self {
    let capacity: f64 = def.parse_child_elem("Capacity").unwrap().unwrap();
    let operational_power_consumption: f64 = def.parse_child_elem("OperationalPowerConsumption").unwrap().unwrap();
    let idle_power_consumption: f64 = def.parse_child_elem("StandbyPowerConsumption").unwrap().unwrap();
    HydrogenTank { capacity, operational_power_consumption, idle_power_consumption }
  }
}

impl Container {
  fn from_def(def: &Node, entity_components: &Node) -> Self {
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

impl Connector {
  fn from_def(def: &Node, data: &BlockData) -> Self {
    let size = def.child_elem("Size").unwrap();
    let x: f64 = size.parse_attribute("x").unwrap().unwrap();
    let y: f64 = size.parse_attribute("y").unwrap().unwrap();
    let z: f64 = size.parse_attribute("z").unwrap().unwrap();
    let multiplier = data.size.size() * 0.8;
    let inventory_volume_any = (x * multiplier) * (y * multiplier) * (z * multiplier) * VOLUME_MULTIPLIER; // Inventory capacity according to MyShipConnector.cs.
    Self {
      inventory_volume_any,
    }
  }
}

impl Cockpit {
  fn from_def(def: &Node) -> Self {
    let has_inventory = def.parse_child_elem("HasInventory").unwrap().unwrap_or(true);
    let inventory_volume_any = if has_inventory { VOLUME_MULTIPLIER } else { 0.0 }; // Inventory capacity according to MyCockpit.cs.
    Cockpit { has_inventory, inventory_volume_any }
  }
}


impl Drill {
  fn from_def(def: &Node, data: &BlockData) -> Self {
    let size = def.child_elem("Size").unwrap();
    let x: f64 = size.parse_attribute("x").unwrap().unwrap();
    let y: f64 = size.parse_attribute("y").unwrap().unwrap();
    let z: f64 = size.parse_attribute("z").unwrap().unwrap();
    let cube_size = data.size.size();
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
