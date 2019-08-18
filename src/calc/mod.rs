use std::collections::HashMap;

use crate::data::Data;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
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

  // Volume & Mass
  pub containers: HashMap<u64, u64>,
  pub cockpits: HashMap<u64, u64>,
  // TODO: drills

  pub thrusters: HashMap<ThrusterSide, HashMap<u64, u64>>,

  pub hydrogen_engines: HashMap<u64, u64>,
  pub reactors: HashMap<u64, u64>,

  pub batteries: HashMap<u64, u64>,

  pub generators: HashMap<u64, u64>,
  pub hydrogen_tanks: HashMap<u64, u64>,
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

  pub fn calculate(&self, data: &Data) -> Calculated {
    let ice_weight_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ice_items_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ore_weight_per_volume = 1.0 / 0.37; // TODO: derive from data
    let ore_items_per_volume = 1.0 / 0.37; // TODO: derive from data
    let steel_plate_weight_per_volume = 20.0 / 3.0; // TODO: derive from data
    let steel_plate_items_per_volume = 1.0 / 3.0; // TODO: derive from data

    let mut c = Calculated::default();

    // Containers.
    for (id, count) in self.containers.iter() {
      let block = data.blocks.containers.get(id).unwrap();
      let count = *count as f64;
      // Mass
      c.total_mass_empty += block.mass(&data.components) * count;
      // Volume
      if block.store_any {
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

        // Mass
        c.total_mass_empty += block.mass(&data.components) * count;

        // Calculate force.
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
      }
      c.acceleration.insert(*side, a);
    }

    // Calculate filled volumes.
    let ice_only_volume = c.total_volume_ice_only * (self.ice_only_fill / 100.0);
    let ore_only_volume = c.total_volume_ore_only * (self.ore_only_fill / 100.0);
    let ice_in_any_volume = c.total_volume_any * (self.any_fill_with_ice / 100.0);
    let ore_in_any_volume = c.total_volume_any * (self.any_fill_with_ore / 100.0);
    let steel_plates_in_any_volume = c.total_volume_any * (self.any_fill_with_steel_plates / 100.0);

    // Calculate filled mass.
    let ice_only_mass = ice_only_volume * ice_weight_per_volume;
    let ore_only_mass = ore_only_volume * ore_weight_per_volume;
    let any_mass = (ice_in_any_volume * ice_weight_per_volume) + (ore_in_any_volume * ore_weight_per_volume) + (steel_plates_in_any_volume * steel_plate_weight_per_volume);
    c.total_mass_filled = c.total_mass_empty + ice_only_mass + ore_only_mass + any_mass;

    // Calculate filled items.
    c.total_items_ice = (ice_only_volume + ice_in_any_volume) * ice_items_per_volume;
    c.total_items_ore = (ore_only_volume + ore_in_any_volume) * ore_items_per_volume;
    c.total_items_steel_plates = steel_plates_in_any_volume * steel_plate_items_per_volume;

    // Calculate Acceleration
    for a in c.acceleration.values_mut() {
      a.acceleration_empty_no_gravity = a.force / c.total_mass_empty;
      a.acceleration_filled_no_gravity = a.force / c.total_mass_filled;
      a.acceleration_empty_gravity = (a.force - (c.total_mass_empty * 9.81 * self.gravity_multiplier)) / c.total_mass_empty;
      a.acceleration_filled_gravity = (a.force - (c.total_mass_filled * 9.81 * self.gravity_multiplier)) / c.total_mass_filled;
    }

    c
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
  pub total_items_steel_plates: f64,

  pub acceleration: HashMap<ThrusterSide, AccelerationCalculated>,
}

#[derive(Default)]
pub struct AccelerationCalculated {
  pub force: f64,
  pub acceleration_empty_no_gravity: f64,
  pub acceleration_empty_gravity: f64,
  pub acceleration_filled_no_gravity: f64,
  pub acceleration_filled_gravity: f64,
}