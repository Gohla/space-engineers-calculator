use std::borrow::Borrow;
use std::fmt::Display;
use std::ops::{Deref, DerefMut, RangeInclusive};

use eframe::epaint::Rgba;
use eframe::Frame;
use egui::{Align, Button, CollapsingHeader, CollapsingResponse, Context, DragValue, Grid, InnerResponse, Layout, Response, Ui, Visuals, WidgetText, Window};
use egui::emath::Numeric;
use thousands::{Separable, SeparatorPolicy};
use tracing::trace;

use secalc_core::data::Data;
use secalc_core::grid::{CountPerDirection, GridCalculated, GridCalculator};

// App

pub struct App {
  data: Data,
  calculator: GridCalculator,
  calculator_default: GridCalculator,
  calculated: GridCalculated,
  number_separator_policy: SeparatorPolicy<'static>,
}

impl App {
  pub fn new(data: Data, _ctx: &eframe::CreationContext<'_>) -> Self {
    let calculator = GridCalculator::default();
    let calculator_default = GridCalculator::default();
    let calculated = GridCalculated::default();
    let number_separator_policy = SeparatorPolicy {
      separator: "·",
      groups: &[3],
      digits: thousands::digits::ASCII_DECIMAL,
    };
    Self { data, calculator, calculator_default, calculated, number_separator_policy }
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

  fn clear_color(&self, visuals: &Visuals) -> Rgba {
    visuals.window_fill().into()
  }
}

impl App {
  fn show_calculator(&mut self, ui: &mut Ui) -> bool {
    let mut changed = false;
    ui.open_header_with_grid("Options", |ui| {
      let mut ui = CalculatorUi::new(ui, self.number_separator_policy, 60.0);
      ui.edit_suffix_row("Gravity Multiplier", "x", &mut self.calculator.gravity_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.gravity_multiplier);
      ui.edit_suffix_row("Container Multiplier", "x", &mut self.calculator.container_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.container_multiplier);
      ui.edit_suffix_row("Planetary Influence", "x", &mut self.calculator.planetary_influence, 0.001, 0.0..=1.0, self.calculator_default.planetary_influence);
      ui.edit_suffix_row("Additional Mass", "kg", &mut self.calculator.additional_mass, 100.0, 0.0..=f64::INFINITY, self.calculator_default.additional_mass);
      ui.edit_percentage_row("Ice-only Fill", &mut self.calculator.ice_only_fill, self.calculator_default.ice_only_fill);
      ui.edit_percentage_row("Ore-only Fill", &mut self.calculator.ore_only_fill, self.calculator_default.ore_only_fill);
      ui.edit_percentage_row("Any-fill with Ice", &mut self.calculator.any_fill_with_ice, self.calculator_default.any_fill_with_ice);
      ui.edit_percentage_row("Any-fill with Ore", &mut self.calculator.any_fill_with_ore, self.calculator_default.any_fill_with_ore);
      ui.edit_percentage_row("Any-fill with Steel Plates", &mut self.calculator.any_fill_with_steel_plates, self.calculator_default.any_fill_with_steel_plates);
      changed |= ui.changed
    });
    let block_edit_size = 5.0;
    ui.open_header("Grid", |ui| {
      ui.open_header("Storage", |ui| {
        ui.open_header_with_grid("Small", |ui| {
          let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
          for block in self.data.blocks.containers.values().filter(|b| b.is_small()) {
            ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
          }
          changed |= ui.changed
        });
        ui.open_header_with_grid("Large", |ui| {
          let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
          for block in self.data.blocks.containers.values().filter(|b| b.is_large()) {
            ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
          }
          changed |= ui.changed
        });
      });
      ui.open_header("Thrusters", |ui| {
        ui.open_header_with_grid("Small", |ui| {
          let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
          ui.header_count_directed_row();
          for block in self.data.blocks.thrusters.values().filter(|b| b.is_small()) {
            let count_per_direction = self.calculator.directional_blocks.entry(block.id.clone()).or_default();
            ui.edit_count_directed_row(block.name(&self.data.localization), count_per_direction);
          }
          changed |= ui.changed
        });
        ui.open_header_with_grid("Large", |ui| {
          let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
          ui.header_count_directed_row();
          for block in self.data.blocks.thrusters.values().filter(|b| b.is_large()) {
            let count_per_direction = self.calculator.directional_blocks.entry(block.id.clone()).or_default();
            ui.edit_count_directed_row(block.name(&self.data.localization), count_per_direction);
          }
          changed |= ui.changed
        });
      });
    });
    changed
  }

  fn show_results(&mut self, ui: &mut Ui) {
    ui.open_header_with_grid("Mass", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      ui.show("Empty", format!("{} kg", self.calculated.total_mass_empty.round()));
      ui.show("Filled", format!("{} kg", self.calculated.total_mass_filled.round()));
    });
    ui.open_header_with_grid("Volume", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      ui.show("Any", format!("{} L", self.calculated.total_volume_any));
      ui.show("Ore", format!("{} L", self.calculated.total_volume_ore));
      ui.show("Ice", format!("{} L", self.calculated.total_volume_ice));
      ui.show("Ore-only", format!("{} L", self.calculated.total_volume_ore_only));
      ui.show("Ice-only", format!("{} L", self.calculated.total_volume_ice_only));
    });
    ui.open_header_with_grid("Items", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      ui.show("Ore", format!("{} #", self.calculated.total_items_ore.round()));
      ui.show("Ice", format!("{} #", self.calculated.total_items_ice.round()));
      ui.show("Steel Plate", format!("{} #", self.calculated.total_items_steel_plate.round()));
    });
  }
}


// Calculator UI

struct CalculatorUi<'ui> {
  ui: &'ui mut Ui,
  _number_separator_policy: SeparatorPolicy<'static>,
  edit_size: f32,
  changed: bool,
}

impl<'ui> CalculatorUi<'ui> {
  fn new(ui: &'ui mut Ui, number_separator_policy: SeparatorPolicy<'static>, edit_size: f32, ) -> Self {
    Self { ui, _number_separator_policy: number_separator_policy, edit_size, changed: false }
  }


  fn edit_row<N: Numeric + Display>(&mut self, label: impl Into<WidgetText>, suffix: Option<impl Into<WidgetText>>, value: &mut N, speed: impl Into<f64>, clamp_range: RangeInclusive<N>, reset_value: N) {
    self.ui.label(label);
    self.drag(value, speed, clamp_range);
    if let Some(suffix) = suffix {
      self.ui.label(suffix);
    }
    self.reset_button_with(value, reset_value);
    self.ui.end_row();
  }

  fn edit_suffix_row<N: Numeric + Display>(&mut self, label: impl Into<WidgetText>, suffix: impl Into<WidgetText>, value: &mut N, speed: impl Into<f64>, clamp_range: RangeInclusive<N>, reset_value: N) {
    self.edit_row(label, Some(suffix), value, speed, clamp_range, reset_value)
  }

  fn edit_percentage_row(&mut self, label: impl Into<WidgetText>, value: &mut f64, reset_value: f64) {
    self.edit_suffix_row(label, "%", value, 0.1, 0.0..=100.0, reset_value)
  }

  fn edit_count_row(&mut self, label: impl Into<WidgetText>, value: &mut u64) {
    self.edit_row(label, None::<&str>, value, 0.01, 0..=u64::MAX, 0)
  }


  fn header_count_directed_row(&mut self) {
    self.ui.label("");
    self.ui.label("Up");
    self.ui.label("Down");
    self.ui.label("Front");
    self.ui.label("Back");
    self.ui.label("Left");
    self.ui.label("Right");
    self.ui.label("");
    self.ui.end_row();
  }

  fn edit_count_directed_row(&mut self, label: impl Into<WidgetText>, count_per_direction: &mut CountPerDirection) {
    self.ui.label(label);
    self.unlabelled_edit_count(count_per_direction.up_mut());
    self.unlabelled_edit_count(count_per_direction.down_mut());
    self.unlabelled_edit_count(count_per_direction.front_mut());
    self.unlabelled_edit_count(count_per_direction.back_mut());
    self.unlabelled_edit_count(count_per_direction.left_mut());
    self.unlabelled_edit_count(count_per_direction.right_mut());
    self.reset_button_with_hover_tooltip(count_per_direction, CountPerDirection::default(), "Reset all to 0");
    self.ui.end_row();
  }

  fn unlabelled_edit_count(&mut self, value: &mut u64) {
    self.drag(value, 0.01, 0..=u64::MAX)
  }


  fn drag<N: Numeric>(&mut self, value: &mut N, speed: impl Into<f64>, clamp_range: RangeInclusive<N>) {
    let drag_value = DragValue::new(value)
      .speed(speed)
      .clamp_range(clamp_range)
      //.custom_formatter(|value, range| emath::format_with_decimals_in_range(value, range).separate_by_policy(self.number_separator_policy))
      ;
    self.changed |= self.ui.add_sized([self.edit_size, self.ui.available_height()], drag_value).changed();
  }


  fn reset_button(&mut self, enabled: bool) -> Response {
    self.ui.add_enabled(enabled, Button::new("↺"))
  }

  fn reset_button_with<T: PartialEq + Display + Copy>(&mut self, value: &mut T, reset_value: T) {
    self.reset_button_with_hover_tooltip(value, reset_value, format!("Reset to {}", reset_value))
  }

  fn reset_button_with_hover_tooltip<T: PartialEq>(&mut self, value: &mut T, reset_value: T, hover_tooltip: impl Into<WidgetText>) {
    let response = self.reset_button(*value != reset_value)
      .on_hover_text_at_pointer(hover_tooltip);
    if response.clicked() {
      *value = reset_value;
      self.changed = true;
    }
  }
}

impl<'ui> Deref for CalculatorUi<'ui> {
  type Target = Ui;
  fn deref(&self) -> &Self::Target { &self.ui }
}

impl<'ui> DerefMut for CalculatorUi<'ui> {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.ui }
}


// Calculated UI

struct CalculatedUi<'ui> {
  ui: &'ui mut Ui,
  number_separator_policy: SeparatorPolicy<'static>,
}

impl<'ui> CalculatedUi<'ui> {
  fn new(ui: &'ui mut Ui, number_separator_policy: SeparatorPolicy<'static>) -> Self {
    Self { ui, number_separator_policy }
  }

  fn show(&mut self, label: impl Into<WidgetText>, value: impl Borrow<str>) {
    self.ui.label(label);
    self.ui.with_layout(Layout::right_to_left(Align::Center), |ui| ui.label(value.borrow().separate_by_policy(self.number_separator_policy)));
    self.ui.end_row();
  }
}

impl<'ui> Deref for CalculatedUi<'ui> {
  type Target = Ui;
  fn deref(&self) -> &Self::Target { &self.ui }
}

impl<'ui> DerefMut for CalculatedUi<'ui> {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.ui }
}


// UI extensions

trait UiExtensions {
  fn open_header_with_grid<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>>;
  fn open_header<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R>;
}

impl UiExtensions for Ui {
  fn open_header_with_grid<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>> {
    CollapsingHeader::new(header).default_open(true).show(self, |ui| {
      Grid::new(format!("{} Grid", header)).striped(true).min_col_width(1.0).show(ui, add_contents)
    })
  }

  fn open_header<R>(&mut self, header: &str, add_body: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R> {
    CollapsingHeader::new(header).default_open(true).show(self, add_body)
  }
}