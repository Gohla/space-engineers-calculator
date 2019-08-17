use std::collections::HashMap;

use crate::data::Data;

pub struct Calculator {
  pub gravity_multiplier: f64,
  pub container_multiplier: f64,
  pub planetary_influence: f64,
  pub inventory_fill: f64,
  pub additional_mass: f64,

  // Volume & Mass
  pub containers: HashMap<u64, u64>,
  pub cockpits: HashMap<u64, u64>,

  // Thrusters
  pub thrusters_up: HashMap<u64, u64>,
  pub thrusters_down: HashMap<u64, u64>,
  pub thrusters_front: HashMap<u64, u64>,
  pub thrusters_back: HashMap<u64, u64>,
  pub thrusters_left: HashMap<u64, u64>,
  pub thrusters_right: HashMap<u64, u64>,

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
      inventory_fill: 100.0,
      additional_mass: 0.0,

      containers: Default::default(),
      cockpits: Default::default(),

      thrusters_up: Default::default(),
      thrusters_down: Default::default(),
      thrusters_front: Default::default(),
      thrusters_back: Default::default(),
      thrusters_left: Default::default(),
      thrusters_right: Default::default(),

      hydrogen_engines: Default::default(),
      reactors: Default::default(),
      batteries: Default::default(),

      generators: Default::default(),
      hydrogen_tanks: Default::default(),
    }
  }

  pub fn calculate(&self, data: &Data) -> Calculated {
    let mut total_volume_ore = 0.0;
    for (id, count) in self.containers.iter() {
      let container = data.blocks.containers.get(&id).unwrap();
      total_volume_ore += container.details.capacity * (*count as f64) * self.container_multiplier;
    }
    Calculated { total_volume_ore }
  }
}


pub struct Calculated {
  pub total_volume_ore: f64,
}