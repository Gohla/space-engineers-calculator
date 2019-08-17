#![allow(dead_code, unused_imports)]

use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use gio::prelude::*;
use gtk::{Align, Application, ApplicationWindow, Entry, Grid, Label};
use gtk::prelude::*;

use crate::calc::Calculator;
use crate::data::blocks::{Block, Blocks};
use crate::data::Data;

pub struct MainWindow {
  window: ApplicationWindow,

  gravity_multiplier: Entry,
  container_multiplier: Entry,
  planetary_influence: Entry,
  inventory_fill: Entry,
  additional_mass: Entry,

  volume_mass_input_small: Grid,
  volume_mass_input_large: Grid,

  total_volume_ore: Label,

  // Rc to support usage in 'static closures by clone+move, RefCell to support mutability in those closures.
  calculator: Rc<RefCell<Calculator>>,
}

impl MainWindow {
  pub fn new(data: &Data) -> Self {
    let glade_src = include_str!("main_window.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    let window: ApplicationWindow = builder.get_object("application_window").unwrap();

    let gravity_multiplier = builder.get_object("gravity_multiplier").unwrap();
    let container_multiplier = builder.get_object("container_multiplier").unwrap();
    let planetary_influence = builder.get_object("planetary_influence").unwrap();
    let inventory_fill = builder.get_object("inventory_fill").unwrap();
    let additional_mass = builder.get_object("additional_mass").unwrap();

    let volume_mass_input_small = builder.get_object("volume_mass_input_small").unwrap();
    let volume_mass_input_large = builder.get_object("volume_mass_input_large").unwrap();

    let total_volume_ore = builder.get_object("total_volume_ore").unwrap();

    let calculator = Rc::new(RefCell::new(Calculator::new()));

    let mut main_window = MainWindow {
      window,

      gravity_multiplier,
      container_multiplier,
      planetary_influence,
      inventory_fill,
      additional_mass,

      volume_mass_input_small,
      volume_mass_input_large,

      total_volume_ore,

      calculator,
    };
    main_window.initialize(data);
    main_window
  }

  fn initialize(&mut self, data: &Data) {
    self.create_block_inputs(data, data.blocks.containers.values(), &self.volume_mass_input_small, &self.volume_mass_input_large, |c| &mut c.containers);
  }

  fn create_block_inputs<'a, T: 'a, I: Iterator<Item=&'a Block<T>>, F: (Fn(&mut Calculator) -> &mut HashMap<u64, u64>) + 'static + Copy>(&self, data: &'a Data, iter: I, small_grid: &Grid, large_grid: &Grid, calculator_func: F) {
    let (small, large) = Blocks::small_and_large_sorted(iter);
    self.create_block_input_grid(data, small, small_grid, calculator_func);
    self.create_block_input_grid(data, large, large_grid, calculator_func);
  }

  fn create_block_input_grid<T, F: (Fn(&mut Calculator) -> &mut HashMap<u64, u64>) + 'static + Copy>(&self, data: &Data, blocks: Vec<&Block<T>>, grid: &Grid, calculator_func: F) {
    for (index, block) in blocks.into_iter().enumerate() {
      let index = index as i32;
      grid.insert_row(index);

      let label = Label::new(Some(block.name(&data.localization)));
      label.set_halign(Align::Start);
      grid.attach(&label, 0, index, 1, 1);

      let entry = Entry::new();
      entry.set_placeholder_text(Some("0"));
      grid.attach(&entry, 1, index, 1, 1);

      let calculator = self.calculator.clone(); // Clone here to avoid capture of `self` in closure.
      let id = block.id; // Copy here to avoid capture of `block` in closure.
      // Move environment into closure, since closure must be 'static (cannot borrow).
      entry.clone().connect_changed(move |entry| {
        if let Some(text) = entry.get_text() {
          if let Ok(count) = text.parse() {
            let mut calculator = calculator.deref().borrow_mut();
            calculator_func(calculator.deref_mut()).insert(id, count);
            // TODO: recalculate
          }
        }
      });
    }
  }

  fn recalculate(&self, data: &Data) {
    let calculated = self.calculator.deref().borrow().calculate(data);
    self.total_volume_ore.set_text(&format!("{}", calculated.total_volume_ore));
  }


  pub fn set_application(&self, app: &Application) {
    self.window.set_application(Some(app));
  }

  pub fn show(&self) {
    self.window.show_all();
  }
}
