use std::fmt::Display;
use std::ops::{Deref, DerefMut, RangeInclusive};

use egui::{Button, ComboBox, DragValue, Response, RichText, Ui, Vec2, WidgetText};
use egui::emath::Numeric;
use thousands::SeparatorPolicy;

use secalc_core::data::blocks::GridSize;
use secalc_core::grid::{BatteryMode, HydrogenTankMode};
use secalc_core::grid::direction::CountPerDirection;

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_calculator(&mut self, ui: &mut Ui) -> bool {
    let mut changed = false;
    ui.open_collapsing_header("Options", |ui| {
      ui.horizontal_top(|ui| {
        ui.grid("Options Grid 1", |ui| {
          let mut ui = CalculatorUi::new(ui, self.number_separator_policy, 100.0 + (self.font_size_modifier * 2) as f32);
          ui.edit_suffix_row("Gravity Multiplier", "x", &mut self.calculator.gravity_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.gravity_multiplier);
          ui.edit_suffix_row("Container Multiplier", "x", &mut self.calculator.container_multiplier, 0.001, 0.0..=f64::INFINITY, self.calculator_default.container_multiplier);
          ui.edit_suffix_row(RichText::new("Planetary Influence").underline(), "x", &mut self.calculator.planetary_influence, 0.001, 0.0..=1.0, self.calculator_default.planetary_influence)
            .on_hover_text_at_pointer("How close to the ground level of a planet's atmosphere the grid is, with 1.0 being on or below ground level, and 0.0 being in vacuum. Lower values negatively affect atmospheric thrusters, and positively affect ion thrusters.");
          ui.edit_suffix_row("Additional Mass", "kg", &mut self.calculator.additional_mass, 100.0, 0.0..=f64::INFINITY, self.calculator_default.additional_mass);
          ui.edit_percentage_row("Thruster Power", &mut self.calculator.thruster_power, self.calculator_default.thruster_power);
          ui.edit_percentage_row("Wheel Power", &mut self.calculator.wheel_power, self.calculator_default.wheel_power);
          ui.checkbox_suffix_row("Charge Railguns", "", &mut self.calculator.railgun_charging, self.calculator_default.railgun_charging);
          ui.checkbox_suffix_row("Charge Jump Drives", "", &mut self.calculator.jump_drive_charging, self.calculator_default.jump_drive_charging);
          ui.combobox_suffix_row("Battery Mode", "Battery Mode", "", &mut self.calculator.battery_mode, BatteryMode::items(), self.calculator_default.battery_mode);
          ui.edit_percentage_row("Battery Fill", &mut self.calculator.battery_fill, self.calculator_default.battery_fill);
          changed |= ui.changed
        });
        ui.grid("Options Grid 2", |ui| {
          let mut ui = CalculatorUi::new(ui, self.number_separator_policy, 90.0 + (self.font_size_modifier * 2) as f32);
          ui.combobox_suffix_row("Hydrogen Tanks Mode", "Hydrogen Tanks Mode", "", &mut self.calculator.hydrogen_tank_mode, HydrogenTankMode::items(), self.calculator_default.hydrogen_tank_mode);
          ui.edit_percentage_row("Hydrogen Tanks Fill", &mut self.calculator.hydrogen_tank_fill, self.calculator_default.hydrogen_tank_fill);
          ui.checkbox_suffix_row("Engines Enabled", "", &mut self.calculator.hydrogen_engine_enabled, self.calculator_default.hydrogen_engine_enabled);
          ui.edit_percentage_row("Engines Fill", &mut self.calculator.hydrogen_engine_fill, self.calculator_default.hydrogen_engine_fill);
          ui.edit_percentage_row("Ice-only Fill", &mut self.calculator.ice_only_fill, self.calculator_default.ice_only_fill);
          ui.edit_percentage_row("Ore-only Fill", &mut self.calculator.ore_only_fill, self.calculator_default.ore_only_fill);
          ui.edit_percentage_row("Any-fill with Ice", &mut self.calculator.any_fill_with_ice, self.calculator_default.any_fill_with_ice);
          ui.edit_percentage_row("Any-fill with Ore", &mut self.calculator.any_fill_with_ore, self.calculator_default.any_fill_with_ore);
          ui.edit_percentage_row("Any-fill with Steel Plates", &mut self.calculator.any_fill_with_steel_plates, self.calculator_default.any_fill_with_steel_plates);
          changed |= ui.changed
        });
      });
    });
    let block_edit_size = 40.0 + self.font_size_modifier as f32;
    ui.open_collapsing_header("Grid", |ui| {
      ComboBox::from_id_source("Grid Size")
        .selected_text(format!("{}", self.grid_size))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.grid_size, GridSize::Small, "Small");
          ui.selectable_value(&mut self.grid_size, GridSize::Large, "Large");
        });
      ui.open_collapsing_header_with_grid("Thrusters", |ui| {
        let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
        ui.header_count_directed_row();
        for data in self.data.blocks.thruster_blocks(self.grid_size, &self.enabled_mod_ids) {
          let count_per_direction = self.calculator.directional_blocks.entry(data.id_cloned()).or_default();
          ui.edit_count_directed_row(data.name(&self.data.localization), count_per_direction);
        }
        changed |= ui.changed
      });
      ui.horizontal(|ui| {
        ui.vertical(|ui| {
          ui.open_collapsing_header_with_grid("Storage", |ui| {
            let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
            for data in self.data.blocks.storage_blocks(self.grid_size, &self.enabled_mod_ids) {
              ui.edit_count_row(data.name(&self.data.localization), self.calculator.blocks.entry(data.id_cloned()).or_default());
            }
            changed |= ui.changed
          });
          ui.open_collapsing_header_with_grid("Wheel Suspensions", |ui| {
            let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
            for data in self.data.blocks.wheel_suspension_blocks(self.grid_size, &self.enabled_mod_ids) {
              ui.edit_count_row(data.name(&self.data.localization), self.calculator.blocks.entry(data.id_cloned()).or_default());
            }
            changed |= ui.changed
          });
        });
        ui.vertical(|ui| {
          ui.open_collapsing_header_with_grid("Power", |ui| {
            let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
            for data in self.data.blocks.power_blocks(self.grid_size, &self.enabled_mod_ids) {
              ui.edit_count_row(data.name(&self.data.localization), self.calculator.blocks.entry(data.id_cloned()).or_default());
            }
            changed |= ui.changed
          });
          ui.open_collapsing_header_with_grid("Hydrogen", |ui| {
            let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
            for data in self.data.blocks.hydrogen_blocks(self.grid_size, &self.enabled_mod_ids) {
              ui.edit_count_row(data.name(&self.data.localization), self.calculator.blocks.entry(data.id_cloned()).or_default());
            }
            changed |= ui.changed
          });
          ui.open_collapsing_header_with_grid("Other", |ui| {
            let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
            for data in self.data.blocks.other_blocks(self.grid_size, &self.enabled_mod_ids) {
              ui.edit_count_row(data.name(&self.data.localization), self.calculator.blocks.entry(data.id_cloned()).or_default());
            }
            changed |= ui.changed
          });
        });
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


  fn edit_row<N: Numeric + Display>(
    &mut self,
    label: impl Into<WidgetText>,
    suffix: Option<impl Into<WidgetText>>,
    value: &mut N,
    speed: impl Into<f64>,
    clamp_range: RangeInclusive<N>,
    reset_value: N
  ) -> Response {
    let label_response = self.ui.label(label);
    self.drag(value, speed, clamp_range);
    if let Some(suffix) = suffix {
      self.ui.label(suffix);
    }
    self.reset_button_with(value, reset_value);
    self.ui.end_row();
    label_response
  }

  fn edit_suffix_row<N: Numeric + Display>(
    &mut self,
    label: impl Into<WidgetText>,
    suffix: impl Into<WidgetText>,
    value: &mut N,
    speed: impl Into<f64>,
    clamp_range: RangeInclusive<N>,
    reset_value: N
  ) -> Response {
    self.edit_row(label, Some(suffix), value, speed, clamp_range, reset_value)
  }

  fn edit_percentage_row(&mut self, label: impl Into<WidgetText>, value: &mut f64, reset_value: f64) -> Response {
    self.edit_suffix_row(label, "%", value, 0.1, 0.0..=100.0, reset_value)
  }

  fn edit_count_row(&mut self, label: impl Into<WidgetText>, value: &mut u64) -> Response {
    self.edit_row(label, None::<&str>, value, 0.01, 0..=u64::MAX, 0)
  }


  fn checkbox_row(&mut self, label: impl Into<WidgetText>, suffix: Option<impl Into<WidgetText>>, value: &mut bool, reset_value: bool) {
    self.ui.label(label);
    self.changed |= self.checkbox(value, "").changed();
    if let Some(suffix) = suffix {
      self.ui.label(suffix);
    }
    self.reset_button_with(value, reset_value);
    self.ui.end_row();
  }

  fn checkbox_suffix_row(&mut self, label: impl Into<WidgetText>, suffix: impl Into<WidgetText>, value: &mut bool, reset_value: bool) {
    self.checkbox_row(label, Some(suffix), value, reset_value)
  }

  fn combobox_row<T: PartialEq + Display + Copy>(
    &mut self,
    label: impl Into<WidgetText>,
    id_source: impl std::hash::Hash,
    suffix: Option<impl Into<WidgetText>>,
    value: &mut T,
    values: impl IntoIterator<Item=T>,
    reset_value: T
  ) {
    self.ui.label(label);
    let style = self.ui.style_mut();
    style.spacing.interact_size = Vec2::new(0.0, 24.0); // HACK: fix combo box not starting at the top
    self.changed |= ComboBox::from_id_source(id_source)
      .width(self.edit_size - 8.0)
      .selected_text(format!("{}", value))
      .show_ui(self.ui, |ui| {
        for v in values {
          self.changed |= ui.selectable_value(value, v, format!("{}", v)).changed();
        }
      }).response.changed();
    self.ui.reset_style();
    if let Some(suffix) = suffix {
      self.ui.label(suffix);
    }
    self.reset_button_with(value, reset_value);
    self.ui.end_row();
  }

  fn combobox_suffix_row<T: PartialEq + Display + Copy>(
    &mut self,
    label: impl Into<WidgetText>,
    id_source: impl std::hash::Hash,
    suffix: impl Into<WidgetText>,
    value: &mut T,
    values: impl IntoIterator<Item=T>,
    reset_value: T
  ) {
    self.combobox_row(label, id_source, Some(suffix), value, values, reset_value)
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
