#![allow(dead_code, unused_imports)]

use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::str::FromStr;

use gio::prelude::*;
use gtk::{Align, Application, ApplicationWindow, Entry, Grid, InputPurpose, Label};
use gtk::prelude::*;

use crate::calc::{Calculator, ThrusterSide};
use crate::data::blocks::{Block, Blocks};
use crate::data::Data;

pub struct MainWindow {
  window: ApplicationWindow,

  gravity_multiplier: Entry,
  container_multiplier: Entry,
  planetary_influence: Entry,
  additional_mass: Entry,
  ice_only_fill: Entry,
  ore_only_fill: Entry,
  any_fill_with_ice: Entry,
  any_fill_with_ore: Entry,
  any_fill_with_steel_plates: Entry,

  volume_mass_input_small: Grid,
  volume_mass_input_large: Grid,
  total_volume_any: Label,
  total_volume_ore: Label,
  total_volume_ice: Label,
  total_volume_ore_only: Label,
  total_volume_ice_only: Label,
  total_mass_empty: Label,
  total_mass_filled: Label,
  total_items_ice: Label,
  total_items_ore: Label,
  total_items_steel_plates: Label,

  acceleration_input_small: Grid,
  acceleration_input_large: Grid,
  thrusters: HashMap<ThrusterSide, ThrusterWidgets>,

  power_input_small: Grid,
  power_input_large: Grid,
  power_generation: Label,
  power_balance_idle: Label,
  power_balance_misc: Label,
  power_balance_upto_jump_drive: Label,
  power_balance_upto_generator: Label,
  power_balance_upto_up_down_thruster: Label,
  power_balance_upto_front_back_thruster: Label,
  power_balance_upto_left_right_thruster: Label,
  power_balance_upto_battery: Label,
  power_capacity_battery: Label,

  hydrogen_input_small: Grid,
  hydrogen_input_large: Grid,
  hydrogen_generation: Label,
  hydrogen_balance_idle: Label,
  hydrogen_balance_engine: Label,
  hydrogen_balance_upto_up_down_thruster: Label,
  hydrogen_balance_upto_front_back_thruster: Label,
  hydrogen_balance_upto_left_right_thruster: Label,
  hydrogen_capacity_tank: Label,
  hydrogen_capacity_engine: Label,

  // Rc to support usage in 'static closures by clone+move.
  data: Rc<Data>,
  // Rc to support usage in 'static closures by clone+move, RefCell to support mutability in those closures.
  calculator: Rc<RefCell<Calculator>>,
}

pub struct ThrusterWidgets {
  force: Label,
  acceleration_empty_no_gravity: Label,
  acceleration_filled_no_gravity: Label,
  acceleration_empty_gravity: Label,
  acceleration_filled_gravity: Label,
}

impl MainWindow {
  pub fn new(data: Rc<Data>) -> Rc<Self> {
    let glade_src = include_str!("main_window.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    let window: ApplicationWindow = builder.get_object("application_window").unwrap();

    let gravity_multiplier = builder.get_object("gravity_multiplier").unwrap();
    let container_multiplier = builder.get_object("container_multiplier").unwrap();
    let planetary_influence = builder.get_object("planetary_influence").unwrap();
    let additional_mass = builder.get_object("additional_mass").unwrap();
    let ice_only_fill = builder.get_object("ice_only_fill").unwrap();
    let ore_only_fill = builder.get_object("ore_only_fill").unwrap();
    let any_fill_with_ice = builder.get_object("any_fill_with_ice").unwrap();
    let any_fill_with_ore = builder.get_object("any_fill_with_ore").unwrap();
    let any_fill_with_steel_plates = builder.get_object("any_fill_with_steel_plates").unwrap();

    let volume_mass_input_small = builder.get_object("volume_mass_input_small").unwrap();
    let volume_mass_input_large = builder.get_object("volume_mass_input_large").unwrap();
    let total_volume_any = builder.get_object("total_volume_any").unwrap();
    let total_volume_ore = builder.get_object("total_volume_ore").unwrap();
    let total_volume_ice = builder.get_object("total_volume_ice").unwrap();
    let total_volume_ore_only = builder.get_object("total_volume_ore_only").unwrap();
    let total_volume_ice_only = builder.get_object("total_volume_ice_only").unwrap();
    let total_mass_empty = builder.get_object("total_mass_empty").unwrap();
    let total_mass_filled = builder.get_object("total_mass_filled").unwrap();
    let total_items_ice = builder.get_object("total_items_ice").unwrap();
    let total_items_ore = builder.get_object("total_items_ore").unwrap();
    let total_items_steel_plates = builder.get_object("total_items_steel_plates").unwrap();

    let acceleration_input_small = builder.get_object("acceleration_input_small").unwrap();
    let acceleration_input_large = builder.get_object("acceleration_input_large").unwrap();
    let mut thrusters = HashMap::default();
    for side in ThrusterSide::iter() {
      let side = *side;
      let id_prefix = match side {
        ThrusterSide::Up => "up",
        ThrusterSide::Down => "down",
        ThrusterSide::Front => "front",
        ThrusterSide::Back => "back",
        ThrusterSide::Left => "left",
        ThrusterSide::Right => "right",
      };
      let force = builder.get_object(&(id_prefix.to_string() + "_force")).unwrap();
      let acceleration_empty_no_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_empty_no_gravity")).unwrap();
      let acceleration_filled_no_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_filled_no_gravity")).unwrap();
      let acceleration_empty_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_empty_gravity")).unwrap();
      let acceleration_filled_gravity = builder.get_object(&(id_prefix.to_string() + "_acceleration_filled_gravity")).unwrap();
      let thruster_widgets = ThrusterWidgets {
        force,
        acceleration_empty_no_gravity,
        acceleration_filled_no_gravity,
        acceleration_empty_gravity,
        acceleration_filled_gravity,
      };
      thrusters.insert(side, thruster_widgets);
    }

    let power_input_small = builder.get_object("power_input_small").unwrap();
    let power_input_large = builder.get_object("power_input_large").unwrap();
    let power_generation = builder.get_object("power_generation").unwrap();
    let power_balance_idle = builder.get_object("power_balance_idle").unwrap();
    let power_balance_misc = builder.get_object("power_balance_misc").unwrap();
    let power_balance_upto_jump_drive = builder.get_object("power_balance_upto_jump_drive").unwrap();
    let power_balance_upto_generator = builder.get_object("power_balance_upto_generator").unwrap();
    let power_balance_upto_up_down_thruster = builder.get_object("power_balance_upto_up_down_thruster").unwrap();
    let power_balance_upto_front_back_thruster = builder.get_object("power_balance_upto_front_back_thruster").unwrap();
    let power_balance_upto_left_right_thruster = builder.get_object("power_balance_upto_left_right_thruster").unwrap();
    let power_balance_upto_battery = builder.get_object("power_balance_upto_battery").unwrap();
    let power_capacity_battery = builder.get_object("power_capacity_battery").unwrap();

    let hydrogen_input_small = builder.get_object("hydrogen_input_small").unwrap();
    let hydrogen_input_large = builder.get_object("hydrogen_input_large").unwrap();
    let hydrogen_generation = builder.get_object("hydrogen_generation").unwrap();
    let hydrogen_balance_idle = builder.get_object("hydrogen_balance_idle").unwrap();
    let hydrogen_balance_engine = builder.get_object("hydrogen_balance_engine").unwrap();
    let hydrogen_balance_upto_up_down_thruster = builder.get_object("hydrogen_balance_upto_up_down_thruster").unwrap();
    let hydrogen_balance_upto_front_back_thruster = builder.get_object("hydrogen_balance_upto_front_back_thruster").unwrap();
    let hydrogen_balance_upto_left_right_thruster = builder.get_object("hydrogen_balance_upto_left_right_thruster").unwrap();
    let hydrogen_capacity_tank = builder.get_object("hydrogen_capacity_tank").unwrap();
    let hydrogen_capacity_engine = builder.get_object("hydrogen_capacity_engine").unwrap();

    let calculator = Rc::new(RefCell::new(Calculator::new()));

    let main_window = Rc::new(MainWindow {
      window,

      gravity_multiplier,
      container_multiplier,
      planetary_influence,
      additional_mass,
      ice_only_fill,
      ore_only_fill,
      any_fill_with_ice,
      any_fill_with_ore,
      any_fill_with_steel_plates,

      volume_mass_input_small,
      volume_mass_input_large,
      total_volume_any,
      total_volume_ore,
      total_volume_ice,
      total_volume_ore_only,
      total_volume_ice_only,
      total_mass_empty,
      total_mass_filled,
      total_items_ice,
      total_items_ore,
      total_items_steel_plates,

      acceleration_input_small,
      acceleration_input_large,
      thrusters,

      power_input_small,
      power_input_large,
      power_generation,
      power_balance_idle,
      power_balance_misc,
      power_balance_upto_jump_drive,
      power_balance_upto_generator,
      power_balance_upto_up_down_thruster,
      power_balance_upto_front_back_thruster,
      power_balance_upto_left_right_thruster,
      power_balance_upto_battery,
      power_capacity_battery,

      hydrogen_input_small,
      hydrogen_input_large,
      hydrogen_generation,
      hydrogen_balance_idle,
      hydrogen_balance_engine,
      hydrogen_balance_upto_up_down_thruster,
      hydrogen_balance_upto_front_back_thruster,
      hydrogen_balance_upto_left_right_thruster,
      hydrogen_capacity_tank,
      hydrogen_capacity_engine,

      data,
      calculator,
    });
    main_window.clone().initialize();
    main_window.clone().recalculate();
    main_window
  }

  fn initialize(self: Rc<Self>) {
    self.gravity_multiplier.set_and_recalc_on_change(&self, 1.0, |c| &mut c.gravity_multiplier);
    self.container_multiplier.set_and_recalc_on_change(&self, 1.0, |c| &mut c.container_multiplier);
    self.planetary_influence.set_and_recalc_on_change(&self, 1.0, |c| &mut c.planetary_influence);
    self.additional_mass.set_and_recalc_on_change(&self, 0.0, |c| &mut c.additional_mass);
    self.ice_only_fill.set_and_recalc_on_change(&self, 100.0, |c| &mut c.ice_only_fill);
    self.ore_only_fill.set_and_recalc_on_change(&self, 100.0, |c| &mut c.ore_only_fill);
    self.any_fill_with_ice.set_and_recalc_on_change(&self, 0.0, |c| &mut c.any_fill_with_ice);
    self.any_fill_with_ore.set_and_recalc_on_change(&self, 0.0, |c| &mut c.any_fill_with_ore);
    self.any_fill_with_steel_plates.set_and_recalc_on_change(&self, 0.0, |c| &mut c.any_fill_with_steel_plates);

    // Volume & Mass
    self.clone().create_block_inputs(self.data.blocks.containers.values().filter(|c| c.details.store_any), &self.volume_mass_input_small, &self.volume_mass_input_large, |c| &mut c.containers);
    self.clone().create_block_inputs(self.data.blocks.cockpits.values().filter(|c| c.details.has_inventory), &self.volume_mass_input_small, &self.volume_mass_input_large, |c| &mut c.cockpits);
    // Acceleration
    self.clone().create_acceleration_block_inputs(self.data.blocks.thrusters.values(), &self.acceleration_input_small, &self.acceleration_input_large);
    // Power
    self.clone().create_block_inputs(self.data.blocks.hydrogen_engines.values(), &self.power_input_small, &self.power_input_large, |c| &mut c.hydrogen_engines);
    self.clone().create_block_inputs(self.data.blocks.reactors.values(), &self.power_input_small, &self.power_input_large, |c| &mut c.reactors);
    self.clone().create_block_inputs(self.data.blocks.batteries.values(), &self.power_input_small, &self.power_input_large, |c| &mut c.batteries);
    // Hydrogen
    self.clone().create_block_inputs(self.data.blocks.generators.values(), &self.hydrogen_input_small, &self.hydrogen_input_large, |c| &mut c.generators);
    self.clone().create_block_inputs(self.data.blocks.hydrogen_tanks.values(), &self.hydrogen_input_small, &self.hydrogen_input_large, |c| &mut c.hydrogen_tanks);
  }


  fn create_block_inputs<'a, T: 'a, I, F>(
    self: Rc<Self>,
    iter: I,
    small_grid: &Grid,
    large_grid: &Grid,
    calculator_func: F
  ) where
    F: (Fn(&mut Calculator) -> &mut HashMap<u64, u64>) + 'static + Copy,
    I: Iterator<Item=&'a Block<T>>
  {
    let (small, large) = Blocks::small_and_large_sorted(iter);
    self.clone().create_block_input_grid(small, small_grid, calculator_func);
    self.create_block_input_grid(large, large_grid, calculator_func);
  }

  fn create_block_input_grid<T, F>(
    self: Rc<Self>,
    blocks: Vec<&Block<T>>,
    grid: &Grid,
    calculator_func: F
  ) where
    F: (Fn(&mut Calculator) -> &mut HashMap<u64, u64>) + 'static + Copy
  {
    let index_offset = grid.get_children().len() as i32;
    for (index, block) in blocks.into_iter().enumerate() {
      let index = index as i32 + index_offset;
      grid.insert_row(index as i32);
      let label = Self::create_static_label(block.name(&self.data.localization));
      grid.attach(&label, 0, index, 1, 1);
      let entry = Self::create_entry();
      entry.insert_and_recalc_on_change(&self, block.id, calculator_func);
      grid.attach(&entry, 1, index, 1, 1);
    }
  }


  fn create_acceleration_block_inputs<'a, T: 'a, I>(
    self: Rc<Self>,
    iter: I,
    small_grid: &Grid,
    large_grid: &Grid,
  ) where
    I: Iterator<Item=&'a Block<T>>
  {
    let (small, large) = Blocks::small_and_large_sorted(iter);
    self.clone().create_acceleration_block_input_grid(small, small_grid);
    self.create_acceleration_block_input_grid(large, large_grid);
  }

  fn create_acceleration_block_input_grid<T>(
    self: Rc<Self>,
    blocks: Vec<&Block<T>>,
    grid: &Grid,
  ) {
    for (index, block) in blocks.into_iter().enumerate() {
      let index = index as i32 + 1;
      grid.insert_row(index as i32);
      let label = Self::create_static_label(block.name(&self.data.localization));
      grid.attach(&label, 0, index, 1, 1);
      let entry_up = Self::create_entry();
      entry_up.insert_and_recalc_on_change(&self, block.id, |c| c.thrusters.get_mut(&ThrusterSide::Up).unwrap());
      grid.attach(&entry_up, 1, index, 1, 1);
      let entry_down = Self::create_entry();
      entry_down.insert_and_recalc_on_change(&self, block.id, |c| c.thrusters.get_mut(&ThrusterSide::Down).unwrap());
      grid.attach(&entry_down, 2, index, 1, 1);
      let entry_front = Self::create_entry();
      entry_front.insert_and_recalc_on_change(&self, block.id, |c| c.thrusters.get_mut(&ThrusterSide::Front).unwrap());
      grid.attach(&entry_front, 3, index, 1, 1);
      let entry_back = Self::create_entry();
      entry_back.insert_and_recalc_on_change(&self, block.id, |c| c.thrusters.get_mut(&ThrusterSide::Back).unwrap());
      grid.attach(&entry_back, 4, index, 1, 1);
      let entry_left = Self::create_entry();
      entry_left.insert_and_recalc_on_change(&self, block.id, |c| c.thrusters.get_mut(&ThrusterSide::Left).unwrap());
      grid.attach(&entry_left, 5, index, 1, 1);
      let entry_right = Self::create_entry();
      entry_right.insert_and_recalc_on_change(&self, block.id, |c| c.thrusters.get_mut(&ThrusterSide::Right).unwrap());
      grid.attach(&entry_right, 6, index, 1, 1);
    }
  }


  fn create_static_label(label: &str) -> Label {
    let label = Label::new(Some(label));
    label.set_halign(Align::Start);
    label
  }

  fn create_entry() -> Entry {
    let entry = Entry::new();
    entry.set_input_purpose(InputPurpose::Number);
    entry.set_placeholder_text(Some("0"));
    entry.set_width_chars(3);
    entry
  }


  fn recalculate(&self) {
    let calculated = self.calculator.deref().borrow().calculate(&self.data);

    // Volume & Mass
    self.total_volume_any.set(calculated.total_volume_any);
    self.total_volume_ore.set(calculated.total_volume_ore);
    self.total_volume_ice.set(calculated.total_volume_ice);
    self.total_volume_ore_only.set(calculated.total_volume_ore_only);
    self.total_volume_ice_only.set(calculated.total_volume_ice_only);
    self.total_mass_empty.set(calculated.total_mass_empty);
    self.total_mass_filled.set(calculated.total_mass_filled);
    self.total_items_ice.set(calculated.total_items_ice);
    self.total_items_ore.set(calculated.total_items_ore);
    self.total_items_steel_plates.set(calculated.total_items_steel_plate);
    // Force & Acceleration
    for (side, a) in calculated.acceleration.iter() {
      let widgets = self.thrusters.get(side).unwrap();
      widgets.force.set(a.force);
      widgets.acceleration_empty_no_gravity.set(a.acceleration_empty_no_gravity);
      widgets.acceleration_filled_no_gravity.set(a.acceleration_filled_no_gravity);
      widgets.acceleration_empty_gravity.set(a.acceleration_empty_gravity);
      widgets.acceleration_filled_gravity.set(a.acceleration_filled_gravity);
    }
    // Power
    self.power_generation.set(calculated.power_generation);
    self.power_balance_idle.set(calculated.power_balance_idle);
    self.power_balance_misc.set(calculated.power_balance_misc);
    self.power_balance_upto_jump_drive.set(calculated.power_balance_upto_jump_drive);
    self.power_balance_upto_generator.set(calculated.power_balance_upto_generator);
    self.power_balance_upto_up_down_thruster.set(calculated.power_balance_upto_up_down_thruster);
    self.power_balance_upto_front_back_thruster.set(calculated.power_balance_upto_front_back_thruster);
    self.power_balance_upto_left_right_thruster.set(calculated.power_balance_upto_left_right_thruster);
    self.power_balance_upto_battery.set(calculated.power_balance_upto_battery);
    self.power_capacity_battery.set(calculated.power_capacity_battery);
    // Hydrogen
    self.hydrogen_generation.set(calculated.hydrogen_generation);
    self.hydrogen_balance_idle.set(calculated.hydrogen_balance_idle);
    self.hydrogen_balance_engine.set(calculated.hydrogen_balance_engine);
    self.hydrogen_balance_upto_up_down_thruster.set(calculated.hydrogen_balance_upto_up_down_thruster);
    self.hydrogen_balance_upto_front_back_thruster.set(calculated.hydrogen_balance_upto_front_back_thruster);
    self.hydrogen_balance_upto_left_right_thruster.set(calculated.hydrogen_balance_upto_left_right_thruster);
    self.hydrogen_capacity_tank.set(calculated.hydrogen_capacity_tank);
    self.hydrogen_capacity_engine.set(calculated.hydrogen_capacity_engine);
  }


  pub fn set_application(&self, app: &Application) {
    self.window.set_application(Some(app));
  }

  pub fn show(&self) {
    self.window.show_all();
  }
}


trait MyEntryExt {
  fn parse<T: FromStr + Copy>(&self, default: T) -> T;
  fn insert_and_recalc_on_change<F: (Fn(&mut Calculator) -> &mut HashMap<u64, u64>) + 'static>(&self, main_window: &Rc<MainWindow>, id: u64, func: F);
  fn set_and_recalc_on_change<T: FromStr + Copy + 'static, F: (Fn(&mut Calculator) -> &mut T) + 'static>(&self, main_window: &Rc<MainWindow>, default: T, func: F);
}

impl MyEntryExt for Entry {
  fn parse<T: FromStr + Copy>(&self, default: T) -> T {
    self.get_text().map(|t| t.parse().unwrap_or(default)).unwrap_or(default)
  }

  fn insert_and_recalc_on_change<F: (Fn(&mut Calculator) -> &mut HashMap<u64, u64>) + 'static>(&self, main_window: &Rc<MainWindow>, id: u64, func: F) {
    let rc_clone = main_window.clone();
    self.connect_changed(move |entry| {
      func(rc_clone.calculator.deref().borrow_mut().deref_mut()).insert(id, entry.parse(0));
      rc_clone.recalculate();
    });
  }

  fn set_and_recalc_on_change<T: FromStr + Copy + 'static, F: (Fn(&mut Calculator) -> &mut T) + 'static>(&self, main_window: &Rc<MainWindow>, default: T, func: F) {
    let rc_clone = main_window.clone();
    self.connect_changed(move |entry| {
      *func(rc_clone.calculator.deref().borrow_mut().deref_mut()) = entry.parse(default);
      rc_clone.recalculate();
    });
  }
}


trait MyLabelExt {
  fn set<T: Display>(&self, value: T);
}

impl MyLabelExt for Label {
  fn set<T: Display>(&self, value: T) {
    self.set_text(&format!("{:.2}", value));
  }
}