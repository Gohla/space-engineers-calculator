use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

use egui::{Align, Context, Grid, Layout, TextFormat, TextStyle, Ui, WidgetText};
use egui::text::LayoutJob;
use thousands::{Separable, SeparatorPolicy};

use secalc_core::grid::{HydrogenCalculated, PowerCalculated, ThrusterAccelerationCalculated};
use secalc_core::grid::direction::{Direction, PerDirection};
use secalc_core::grid::duration::Duration;

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_results(&mut self, ui: &mut Ui, ctx: &Context) {
    ui.horizontal(|ui| {
      ui.open_collapsing_header_with_grid("Volume", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_row("Any", format!("{}", self.calculated.total_volume_any.round()), "L");
        ui.show_row("Ore", format!("{}", self.calculated.total_volume_ore.round()), "L");
        ui.show_row("Ice", format!("{}", self.calculated.total_volume_ice.round()), "L");
        ui.show_row("Ore-only", format!("{}", self.calculated.total_volume_ore_only.round()), "L");
        ui.show_row("Ice-only", format!("{}", self.calculated.total_volume_ice_only.round()), "L");
      });
      ui.vertical(|ui| {
        ui.open_collapsing_header_with_grid("Mass", |ui| {
          let mut ui = ResultUi::new(ui, self.number_separator_policy);
          ui.show_row("Empty", format!("{}", self.calculated.total_mass_empty.round()), "kg");
          ui.show_row("Filled", format!("{}", self.calculated.total_mass_filled.round()), "kg");
        });
        ui.open_collapsing_header_with_grid("Items", |ui| {
          let mut ui = ResultUi::new(ui, self.number_separator_policy);
          ui.show_row("Ore", format!("{}", self.calculated.total_items_ore.round()), "#");
          ui.show_row("Ice", format!("{}", self.calculated.total_items_ice.round()), "#");
          ui.show_row("Steel Plate", format!("{}", self.calculated.total_items_steel_plate.round()), "#");
        });
      });
    });
    ui.horizontal(|ui| {
      ui.open_collapsing_header_with_grid("Thruster Acceleration & Force", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.label("");
        ui.label("Filled");
        ui.label("");
        ui.label("Empty");
        ui.label("");
        ui.label("");
        ui.label("Force");
        ui.end_row();
        ui.label("");
        ui.label("Gravity");
        ui.label("No grav.");
        ui.label("Gravity");
        ui.label("No grav.");
        ui.label("");
        ui.end_row();
        for direction in Direction::items() {
          ui.acceleration_row(direction, &self.calculated.thruster_acceleration, ctx);
        }
      });
      ui.open_collapsing_header_with_grid("Wheel Force", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_row("Force", format!("{:.2}", self.calculated.wheel_force / 1000.0), "kN");
      });
    });
    ui.open_collapsing_header("Power", |ui| {
      Grid::new("Power Grid 1").striped(true).show(ui, |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_row("Generation:", format!("{:.2}", self.calculated.power_generation), "MW");
      });
      Grid::new("Power Grid 2").striped(true).show(ui, |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.label("");
        ui.label("Consumption");
        ui.label("");
        ui.label("");
        ui.label("Duration");
        ui.label("");
        ui.end_row();
        ui.label("");
        ui.label("Group");
        ui.label("Total");
        ui.label("Balance");
        ui.label("Batteries");
        ui.label("Engines");
        ui.end_row();
        let power_formatter = |v| format!("{:.2}", v);
        ui.power_row("Idle:", power_formatter, &self.calculated.power_idle);
        ui.power_row("Charge Railguns:", power_formatter, &self.calculated.power_railgun);
        ui.power_row("+ Utility:", power_formatter, &self.calculated.power_upto_utility);
        ui.power_row("+ Wheel Suspensions:", power_formatter, &self.calculated.power_upto_wheel_suspension);
        ui.power_row("+ Charge Jump Drives:", power_formatter, &self.calculated.power_upto_jump_drive);
        ui.power_row("+ O2/H2 Generators:", power_formatter, &self.calculated.power_upto_generator);
        ui.power_row("+ Up/Down Thrusters:", power_formatter, &self.calculated.power_upto_up_down_thruster);
        ui.power_row("+ Front/Back Thrusters:", power_formatter, &self.calculated.power_upto_front_back_thruster);
        ui.power_row("+ Left/Right Thrusters:", power_formatter, &self.calculated.power_upto_left_right_thruster);
        ui.power_row("+ Charge Batteries:", power_formatter, &self.calculated.power_upto_battery);
      });
    });
    ui.horizontal(|ui| {
      ui.open_collapsing_header_with_grid("Railgun", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_optional_row("Capacity:", self.calculated.railgun_capacity.map(|c| format!("{:.2}", c)), "MWh");
        ui.show_optional_duration_row("Charge duration:", self.calculated.railgun_charge_duration);
      });
      ui.open_collapsing_header_with_grid("Jump Drive", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_optional_row("Capacity:", self.calculated.jump_drive_capacity.map(|c| format!("{:.2}", c)), "MWh");
        ui.show_optional_duration_row("Charge duration:", self.calculated.jump_drive_charge_duration);
        ui.show_optional_row("Max range (empty):", self.calculated.jump_drive_max_distance_empty.map(|c| format!("{:.2}", c)), "km");
        ui.show_optional_row("Max range (filled):", self.calculated.jump_drive_max_distance_filled.map(|c| format!("{:.2}", c)), "km");
      });
      ui.open_collapsing_header_with_grid("Battery", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_optional_row("Capacity:", self.calculated.battery_capacity.map(|c| format!("{:.2}", c)), "MWh");
        ui.show_optional_duration_row("Charge duration:", self.calculated.battery_charge_duration);
      });
    });
    ui.open_collapsing_header("Hydrogen", |ui| {
      Grid::new("Hydrogen Grid 1").striped(true).show(ui, |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_row("Generation:", format!("{}", self.calculated.hydrogen_generation.round()), "L/s");
      });
      Grid::new("Hydrogen Grid 2").striped(true).show(ui, |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.label("");
        ui.label("Consumption");
        ui.label("");
        ui.label("Balance");
        ui.label("Duration");
        ui.end_row();
        ui.label("");
        ui.label("Group");
        ui.label("Total");
        ui.label("");
        ui.label("Tanks");
        ui.end_row();
        let hydrogen_formatter = |v| format!("{:.2}", v);
        ui.hydrogen_row("Idle:", hydrogen_formatter, &self.calculated.hydrogen_idle);
        ui.hydrogen_row("Fill Engines:", hydrogen_formatter, &self.calculated.hydrogen_engine);
        ui.hydrogen_row("+ Up/Down Thrusters:", hydrogen_formatter, &self.calculated.hydrogen_upto_up_down_thruster);
        ui.hydrogen_row("+ Front/Back Thrusters:", hydrogen_formatter, &self.calculated.hydrogen_upto_front_back_thruster);
        ui.hydrogen_row("+ Left/Right Thrusters:", hydrogen_formatter, &self.calculated.hydrogen_upto_left_right_thruster);
        ui.hydrogen_row("+ Fill Tanks:", hydrogen_formatter, &self.calculated.hydrogen_upto_tank);
      });
    });
    ui.horizontal(|ui| {
      ui.open_collapsing_header_with_grid("Hydrogen Tank", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_optional_row("Capacity:", self.calculated.hydrogen_tank_capacity.map(|c| format!("{}", c.round())), "L");
        ui.show_optional_duration_row("Fill duration:", self.calculated.hydrogen_tank_fill_duration);
      });
      ui.open_collapsing_header_with_grid("Hydrogen Engine", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_optional_row("Capacity:", self.calculated.hydrogen_engine_capacity.map(|c| format!("{}", c.round())), "L");
        ui.show_optional_duration_row("Fill duration:", self.calculated.hydrogen_engine_fill_duration);
      });
    });
  }
}


struct ResultUi<'ui> {
  ui: &'ui mut Ui,
  number_separator_policy: SeparatorPolicy<'static>,
}

impl<'ui> ResultUi<'ui> {
  fn new(ui: &'ui mut Ui, number_separator_policy: SeparatorPolicy<'static>) -> Self {
    Self { ui, number_separator_policy }
  }


  fn show_row(&mut self, label: impl Into<WidgetText>, value: impl Borrow<str>, unit: impl Into<WidgetText>) {
    self.ui.label(label);
    self.right_align_value_with_unit(value, unit);
    self.ui.end_row();
  }

  fn show_optional_row(&mut self, label: impl Into<WidgetText>, value: Option<impl Borrow<str>>, unit: impl Into<WidgetText>) {
    self.ui.label(label);
    self.right_align_optional_value_with_unit(value, unit);
    self.ui.end_row();
  }


  fn right_align_label(&mut self, label: impl Into<WidgetText>) {
    self.ui.with_layout(Layout::right_to_left(), |ui| ui.label(label));
  }


  fn right_align_value(&mut self, value: impl Borrow<str>) {
    self.ui.with_layout(Layout::right_to_left(), |ui| {
      ui.monospace(value.borrow().separate_by_policy(self.number_separator_policy));
    });
  }

  fn right_align_optional_value(&mut self, value: Option<impl Borrow<str>>) {
    if let Some(value) = value {
      self.right_align_value(value);
    } else {
      self.right_align_value("-");
    }
  }

  fn right_align_value_with_unit(&mut self, value: impl Borrow<str>, unit: impl Into<WidgetText>) {
    self.ui.with_layout(Layout::right_to_left(), |ui| {
      ui.label(unit);
      ui.monospace(value.borrow().separate_by_policy(self.number_separator_policy));
    });
  }

  fn right_align_optional_value_with_unit(&mut self, value: Option<impl Borrow<str>>, unit: impl Into<WidgetText>) {
    if let Some(value) = value {
      self.right_align_value_with_unit(value, unit);
    } else {
      self.right_align_value_with_unit("-", unit);
    }
  }
  

  fn show_optional_duration_row(&mut self, label: impl Into<WidgetText>, duration: Option<Duration>) {
    self.ui.label(label);
    self.right_align_optional_duration(duration);
    self.ui.end_row();
  }

  fn right_align_duration(&mut self, duration: Duration) {
    let (value, unit) = duration.to_f64_and_unit();
    self.right_align_value_with_unit(format!("{:.2}", value), unit);
  }

  fn right_align_optional_duration(&mut self, duration: Option<Duration>) {
    if let Some(duration) = duration {
      self.right_align_duration(duration);
    } else {
      self.right_align_value_with_unit("-", Duration::DEFAULT_UNIT);
    }
  }


  fn acceleration_row(&mut self, direction: Direction, acceleration: &PerDirection<ThrusterAccelerationCalculated>, ctx: &Context) {
    self.right_align_label(format!("{}", direction));
    self.right_align_optional_value(acceleration.get(direction).acceleration_filled_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_filled_no_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_empty_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_empty_no_gravity.map(|a| format!("{:.2}", a)));
    self.acceleration_label(ctx);
    self.right_align_value_with_unit(format!("{:.2}", acceleration.get(direction).force / 1000.0), "kN");
    self.ui.end_row();
  }

  fn acceleration_label(&mut self, ctx: &Context) {
    let mut acceleration = LayoutJob::default();
    let color = ctx.style().visuals.text_color();
    acceleration.append("m/s", 0.0, TextFormat { font_id: TextStyle::Body.resolve(&ctx.style()), color, ..TextFormat::default() });
    acceleration.append("2", 0.0, TextFormat { font_id: TextStyle::Small.resolve(&ctx.style()), color, valign: Align::Min, ..TextFormat::default() });
    self.ui.label(acceleration);
  }

  fn power_row(&mut self, label: impl Into<WidgetText>, power_formatter: impl Fn(f64) -> String, power: &PowerCalculated) {
    self.ui.label(label);
    self.right_align_value_with_unit(power_formatter(power.consumption), "MW");
    self.right_align_value_with_unit(power_formatter(power.total_consumption), "MW");
    self.right_align_value_with_unit(power_formatter(power.balance), "MW");
    self.right_align_optional_duration(power.battery_duration);
    self.right_align_optional_duration(power.engine_duration);
    self.ui.end_row();
  }

  fn hydrogen_row(&mut self, label: impl Into<WidgetText>, hydrogen_formatter: impl Fn(f64) -> String, hydrogen: &HydrogenCalculated) {
    self.ui.label(label);
    self.right_align_value_with_unit(hydrogen_formatter(hydrogen.consumption), "L/s");
    self.right_align_value_with_unit(hydrogen_formatter(hydrogen.total_consumption), "L/s");
    self.right_align_value_with_unit(hydrogen_formatter(hydrogen.balance), "L/s");
    self.right_align_optional_duration(hydrogen.tank_duration);
    self.ui.end_row();
  }
}

impl<'ui> Deref for ResultUi<'ui> {
  type Target = Ui;
  fn deref(&self) -> &Self::Target { &self.ui }
}

impl<'ui> DerefMut for ResultUi<'ui> {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.ui }
}
