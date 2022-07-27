use std::ops::RangeInclusive;

use eframe::Frame;
use egui::{Button, CollapsingHeader, Context, DragValue, Grid, Ui, WidgetText, Window};
use egui::emath::Numeric;
use tracing::trace;

use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

// App

pub struct App {
  data: Data,
  calculator: GridCalculator,
  calculator_default: GridCalculator,
  calculated: GridCalculated,
}

impl App {
  pub fn new(data: Data, _ctx: &eframe::CreationContext<'_>) -> Self {
    let calculator = GridCalculator::default();
    let calculator_default = GridCalculator::default();
    let calculated = GridCalculated::default();
    Self { data, calculator, calculator_default, calculated }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    let changed = Window::new("Grid Calculator")
      .auto_sized()
      .show(ctx, |ui| self.show_calculator(ui)).map_or(false, |r| r.inner.unwrap_or_default());
    if changed {
      trace!("Calculating");
      self.calculated = self.calculator.calculate(&self.data);
    }
    Window::new("Results")
      .auto_sized()
      .show(ctx, |ui| self.show_results(ui));
  }
}

impl App {
  fn show_calculator(&mut self, ui: &mut Ui) -> bool {
    let mut changed = false;
    CollapsingHeader::new("Options").default_open(true).show(ui, |ui| {
      Grid::new("Options Grid")
        .striped(true)
        .num_columns(4)
        .min_col_width(1.0)
        .show(ui, |ui| {
          let mut edit_builder = EditBuilder::new(ui, 60.0);
          edit_builder.edit("Gravity Multiplier", "x", &mut self.calculator.gravity_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.gravity_multiplier);
          edit_builder.edit("Container Multiplier", "x", &mut self.calculator.container_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.container_multiplier);
          edit_builder.edit("Planetary Influence", "x", &mut self.calculator.planetary_influence, 0.001, 0.0..=1.0, self.calculator_default.planetary_influence);
          edit_builder.edit("Additional Mass", "kg", &mut self.calculator.additional_mass, 100.0, 0.0..=f64::INFINITY, self.calculator_default.additional_mass);
          edit_builder.edit("Ice-only Fill", "%", &mut self.calculator.ice_only_fill, 0.1, 0.0..=100.0, self.calculator_default.ice_only_fill);
          edit_builder.edit("Ore-only Fill", "%", &mut self.calculator.ore_only_fill, 0.1, 0.0..=100.0, self.calculator_default.ore_only_fill);
          edit_builder.edit("Any-fill with Ice", "%", &mut self.calculator.any_fill_with_ice, 0.1, 0.0..=100.0, self.calculator_default.any_fill_with_ice);
          edit_builder.edit("Any-fill with Ore", "%", &mut self.calculator.any_fill_with_ore, 0.1, 0.0..=100.0, self.calculator_default.any_fill_with_ore);
          edit_builder.edit("Any-fill with Steel Plates", "%", &mut self.calculator.any_fill_with_steel_plates, 0.1, 0.0..=100.0, self.calculator_default.any_fill_with_steel_plates);
          changed |= edit_builder.changed;
        });
    });
    changed
  }

  fn show_results(&mut self, _ui: &mut Ui) {}
}


// Edit builder

struct EditBuilder<'ui> {
  ui: &'ui mut Ui,
  edit_size: f32,
  changed: bool,
}

impl<'ui> EditBuilder<'ui> {
  fn new(ui: &'ui mut Ui, edit_size: f32, ) -> Self {
    Self { ui, edit_size, changed: false }
  }

  fn edit<N: Numeric>(&mut self, label: impl Into<WidgetText>, suffix: impl Into<WidgetText>, value: &mut N, speed: impl Into<f64>, clamp_range: RangeInclusive<N>, reset_value: N) {
    self.ui.label(label);
    self.changed |= self.ui.add_sized([self.edit_size, self.ui.available_height()], DragValue::new(value).speed(speed).clamp_range(clamp_range)).changed();
    self.ui.label(suffix);
    self.reset_button_with(value, reset_value);
    self.ui.end_row();
  }

  fn reset_button_with<T: PartialEq>(&mut self, value: &mut T, reset_value: T) {
    if self.ui.add_enabled(*value != reset_value, Button::new("↺")).clicked() {
      *value = reset_value;
      self.changed = true;
    }
  }
}
