use eframe::Frame;
use egui::{Context, Ui};

use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};
use crate::widget::*;

pub struct App {
  data: Data,
  calculator: GridCalculator,
  calculated: GridCalculated,
}

impl App {
  pub fn new(data: Data, _ctx: &eframe::CreationContext<'_>) -> Self {
    let calculator = GridCalculator::default();
    let calculated =
    Self { data, calculator }
  }
}

impl eframe::App for App {
  fn update(&mut self, context: &Context, _frame: &mut Frame) {
    context.window("Grid Calculator", |ui|self.show_calculator(ui));
    context.window("Results", |ui|self.show_results(ui));
  }
}

impl App {
  fn show_calculator(&mut self, ui: &mut Ui) {
    ui.collapsing_open_with_grid("Options", "Options Grid", |ui|{
      ui.label("Gravity Multiplier");
      ui.drag_unlabelled_range_with_reset(&mut self.calculator.gravity_multiplier, 0.01, 0.0..=f64::INFINITY, 1.0);
      ui.end_row();
    });
  }

  fn show_results(&mut self, ui: &mut Ui) {
    
  }
}