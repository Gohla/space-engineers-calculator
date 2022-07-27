use std::ops::RangeInclusive;

use eframe::Frame;
use egui::{CollapsingHeader, Context, DragValue, Grid, Label, Ui, WidgetText, Window};
use egui::emath::Numeric;
use egui_extras::{Size, TableBuilder, TableRow};
use tracing::debug;

use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

use crate::widget::*;

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
      .min_width(10.0)
      .default_width(10.0)
      .show(ctx, |ui| self.show_calculator(ui)).map_or(false, |r| r.inner.unwrap_or_default());
    if changed {
      debug!("Calculating");
      self.calculated = self.calculator.calculate(&self.data);
    }
    ctx.window("Results", |ui| self.show_results(ui));
  }
}

impl App {
  fn show_calculator(&mut self, ui: &mut Ui) -> bool {
    let mut changed = false;
    let mut edit_builder = EditBuilder::new();
    CollapsingHeader::new("Options").default_open(true).show(ui, |ui| {
      TableBuilder::new(ui)
        .striped(true)
        .columns(Size::remainder(), 4)
        .body(|mut body| {
          body.row(10.0, |row|{edit_builder.edit(row, "Gravity Multiplier", "x", &mut self.calculator.gravity_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.gravity_multiplier);})
        });
      // Grid::new("Options Grid").striped(true).num_columns(4).show(ui, |ui| {
      //   let mut edit_builder = EditBuilder::new(ui);
      //   edit_builder.edit("Gravity Multiplier", "x", &mut self.calculator.gravity_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.gravity_multiplier);
      //   edit_builder.edit("Container Multiplier", "x", &mut self.calculator.container_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.container_multiplier);
      //   edit_builder.edit("Planetary Influence", "x", &mut self.calculator.planetary_influence, 0.001, 0.0..=1.0, self.calculator_default.planetary_influence);
      //   edit_builder.edit("Additional Mass", "kg", &mut self.calculator.additional_mass, 1.0, 0.0..=f64::INFINITY, self.calculator_default.additional_mass);
      //   edit_builder.edit("Ice-only Fill", "%", &mut self.calculator.ice_only_fill, 0.1, 0.0..=100.0, self.calculator_default.ice_only_fill);
      //   changed |= edit_builder.changed;
      // });
    });
    changed
  }

  fn show_results(&mut self, _ui: &mut Ui) {}
}

struct EditBuilder {
  changed: bool,
}

impl EditBuilder {
  fn new() -> Self { Self { changed: false } }
  fn edit<N: Numeric>(&mut self, mut table_row: TableRow, label: impl Into<WidgetText>, suffix: impl Into<WidgetText>, value: &mut N, speed: impl Into<f64>, clamp_range: RangeInclusive<N>, reset_value: N) {
    table_row.col(|ui| { ui.label(label); });
    table_row.col(|ui| { ui.add(DragValue::new(value).speed(speed).clamp_range(clamp_range)); });
    table_row.col(|ui| { ui.add(Label::new(suffix)); });
    table_row.col(|ui| { ui.reset_button_with(value, reset_value); });
    // // self.;
    // let row_height = self.ui.available_height();
    // self.changed |= self.ui.add_sized([50.0, row_height], DragValue::new(value).speed(speed).clamp_range(clamp_range)).changed();
    // self.ui.add_sized([1.0, row_height], Label::new(suffix));
    // self.changed |= self.ui.reset_button_with(value, reset_value).inner;
    // self.ui.end_row();
  }
}
