use std::fmt::Display;
use std::ops::{Deref, DerefMut, RangeInclusive};

use egui::{Button, ComboBox, DragValue, Response, Ui, WidgetText};
use egui::emath::Numeric;
use thousands::SeparatorPolicy;

use secalc_core::data::blocks::GridSize;
use secalc_core::grid::CountPerDirection;

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub(crate) fn show_calculator(&mut self, ui: &mut Ui) -> bool {
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
      ComboBox::from_id_source("Grid Size")
        .selected_text(format!("{}", self.grid_size))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.grid_size, GridSize::Small, "Small");
          ui.selectable_value(&mut self.grid_size, GridSize::Large, "Large");
        });
      ui.open_header_with_grid("Thrusters", |ui| {
        let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
        ui.header_count_directed_row();
        for block in self.data.blocks.thrusters.values().filter(|b| b.size == self.grid_size) {
          let count_per_direction = self.calculator.directional_blocks.entry(block.id.clone()).or_default();
          ui.edit_count_directed_row(block.name(&self.data.localization), count_per_direction);
        }
        changed |= ui.changed
      });
      ui.open_header_with_grid("Storage", |ui| {
        let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
        for block in self.data.blocks.containers.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        for block in self.data.blocks.connectors.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        for block in self.data.blocks.cockpits.values().filter(|b| b.size == self.grid_size && b.has_inventory) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        changed |= ui.changed
      });
      ui.open_header_with_grid("Power", |ui| {
        let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
        for block in self.data.blocks.hydrogen_engines.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        for block in self.data.blocks.reactors.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        for block in self.data.blocks.batteries.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        changed |= ui.changed
      });
      ui.open_header_with_grid("Hydrogen", |ui| {
        let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
        for block in self.data.blocks.generators.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        for block in self.data.blocks.hydrogen_tanks.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        changed |= ui.changed
      });
      ui.open_header_with_grid("Ship Tools", |ui| {
        let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
        for block in self.data.blocks.drills.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        changed |= ui.changed
      });
    });
    changed
  }
}

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
    self.reset_button_with_hover_tooltip(count_per_direction, CountPerDirection::default(), "Double-click to reset all to 0");
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
    self.reset_button_with_hover_tooltip(value, reset_value, format!("Double-click to reset to {}", reset_value))
  }

  fn reset_button_with_hover_tooltip<T: PartialEq>(&mut self, value: &mut T, reset_value: T, hover_tooltip: impl Into<WidgetText>) {
    let response = self.reset_button(*value != reset_value)
      .on_hover_text_at_pointer(hover_tooltip);
    if response.double_clicked() {
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