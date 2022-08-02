use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

use egui::{Align, Context, Layout, TextFormat, TextStyle, Ui, WidgetText};
use egui::text::LayoutJob;
use thousands::{Separable, SeparatorPolicy};

use secalc_core::grid::{AccelerationCalculated, Direction, HydrogenCalculated, PerDirection, PowerCalculated};

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_results(&mut self, ui: &mut Ui, ctx: &Context) {
    ui.horizontal(|ui|{
      ui.open_header_with_grid("Volume", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_row("Any", format!("{} L", self.calculated.total_volume_any.round()));
        ui.show_row("Ore", format!("{} L", self.calculated.total_volume_ore.round()));
        ui.show_row("Ice", format!("{} L", self.calculated.total_volume_ice.round()));
        ui.show_row("Ore-only", format!("{} L", self.calculated.total_volume_ore_only.round()));
        ui.show_row("Ice-only", format!("{} L", self.calculated.total_volume_ice_only.round()));
      });
      ui.vertical(|ui|{
        ui.open_header_with_grid("Mass", |ui| {
          let mut ui = ResultUi::new(ui, self.number_separator_policy);
          ui.show_row("Empty", format!("{} kg", self.calculated.total_mass_empty.round()));
          ui.show_row("Filled", format!("{} kg", self.calculated.total_mass_filled.round()));
        });
        ui.open_header_with_grid("Items", |ui| {
          let mut ui = ResultUi::new(ui, self.number_separator_policy);
          ui.show_row("Ore", format!("{} #", self.calculated.total_items_ore.round()));
          ui.show_row("Ice", format!("{} #", self.calculated.total_items_ice.round()));
          ui.show_row("Steel Plate", format!("{} #", self.calculated.total_items_steel_plate.round()));
        });
      });
    });
    ui.open_header_with_grid("Acceleration & Force", |ui| {
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
      for direction in Direction::iter() {
        ui.acceleration_row(*direction, &self.calculated.acceleration, ctx);
      }
    });
    ui.open_header_with_grid("Power", |ui| {
      let mut ui = ResultUi::new(ui, self.number_separator_policy);
      ui.show_row("Generation", format!("{:.2} MW", self.calculated.power_generation));
      ui.show_row("Capacity: Batteries", format!("{:.2} MWh", self.calculated.power_capacity_battery));
      ui.label("");
      ui.label("Consumption");
      ui.label("Balance");
      ui.label("Duration: Batteries");
      ui.end_row();
      let power_formatter = |v| format!("{:.2} MW", v);
      let duration_formatter = |v| format!("{:.2} min", v);
      ui.power_row("Idle", power_formatter, duration_formatter, &self.calculated.power_idle);
      ui.power_row("Misc", power_formatter, duration_formatter, &self.calculated.power_misc);
      ui.power_row("+ Charge Jump Drives", power_formatter, duration_formatter, &self.calculated.power_upto_jump_drive);
      ui.power_row("+ O2/H2 Generators", power_formatter, duration_formatter, &self.calculated.power_upto_generator);
      ui.power_row("+ Up/Down Thrusters", power_formatter, duration_formatter, &self.calculated.power_upto_up_down_thruster);
      ui.power_row("+ Front/Back Thrusters", power_formatter, duration_formatter, &self.calculated.power_upto_front_back_thruster);
      ui.power_row("+ Left/Right Thrusters", power_formatter, duration_formatter, &self.calculated.power_upto_left_right_thruster);
      ui.power_row("+ Charge Batteries", power_formatter, duration_formatter, &self.calculated.power_upto_battery);
    });
    ui.open_header_with_grid("Hydrogen", |ui| {
      let mut ui = ResultUi::new(ui, self.number_separator_policy);
      ui.show_row("Generation", format!("{} L/s", self.calculated.hydrogen_generation.round()));
      ui.show_row("Capacity: Tanks", format!("{} L", self.calculated.hydrogen_capacity_tank.round()));
      ui.show_row("Capacity: Engines", format!("{} L", self.calculated.hydrogen_capacity_engine.round()));
      ui.label("");
      ui.label("Consumption");
      ui.label("Balance");
      ui.label("Duration: Tanks");
      ui.label("Duration: Engines");
      ui.end_row();
      let hydrogen_formatter = |v| format!("{:.2} L/s", v);
      let duration_formatter = |v| format!("{:.2} min", v);
      ui.hydrogen_row("Idle", hydrogen_formatter, duration_formatter, &self.calculated.hydrogen_idle);
      ui.hydrogen_row("Engines", hydrogen_formatter, duration_formatter, &self.calculated.hydrogen_engine);
      ui.hydrogen_row("+ Up/Down Thrusters", hydrogen_formatter, duration_formatter, &self.calculated.hydrogen_upto_up_down_thruster);
      ui.hydrogen_row("+ Front/Back Thrusters", hydrogen_formatter, duration_formatter, &self.calculated.hydrogen_upto_front_back_thruster);
      ui.hydrogen_row("+ Left/Right Thrusters", hydrogen_formatter, duration_formatter, &self.calculated.hydrogen_upto_left_right_thruster);
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


  fn show_row(&mut self, label: impl Into<WidgetText>, value: impl Borrow<str>) {
    self.ui.label(label);
    self.right_align_value(value);
    self.ui.end_row();
  }


  fn right_align_value(&mut self, value: impl Borrow<str>) {
    self.right_align_label(value.borrow().separate_by_policy(self.number_separator_policy));
  }

  fn right_align_optional_value(&mut self, value: Option<impl Borrow<str>>) {
    if let Some(value) = value {
      self.right_align_label(value.borrow().separate_by_policy(self.number_separator_policy));
    } else {
      self.empty_label();
    }
  }

  fn right_align_label(&mut self, label: impl Into<WidgetText>) {
    self.ui.with_layout(Layout::right_to_left(), |ui| ui.label(label));
  }

  fn empty_label(&mut self) {
    self.right_align_label("-");
  }


  fn acceleration_row(&mut self, direction: Direction, acceleration: &PerDirection<AccelerationCalculated>, ctx: &Context) {
    self.right_align_label(format!("{}", direction));
    self.right_align_optional_value(acceleration.get(direction).acceleration_filled_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_filled_no_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_empty_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_empty_no_gravity.map(|a| format!("{:.2}", a)));
    self.acceleration_label(ctx);
    self.right_align_value(format!("{:.2}", acceleration.get(direction).force / 1000.0));
    self.label("kN");
    self.ui.end_row();
  }

  fn acceleration_label(&mut self, ctx: &Context) {
    let mut acceleration = LayoutJob::default();
    let color = ctx.style().visuals.text_color();
    acceleration.append("m/s", 0.0, TextFormat { font_id: TextStyle::Body.resolve(&ctx.style()), color, ..TextFormat::default() });
    acceleration.append("2", 0.0, TextFormat { font_id: TextStyle::Small.resolve(&ctx.style()), color, valign: Align::Min, ..TextFormat::default() });
    self.ui.label(acceleration);
  }

  fn power_row(&mut self, label: impl Into<WidgetText>, power_formatter: impl Fn(f64) -> String, duration_formatter: impl Fn(f64) -> String, power: &PowerCalculated) {
    self.ui.label(label);
    self.right_align_value(power_formatter(power.consumption));
    self.right_align_value(power_formatter(power.balance));
    self.right_align_optional_value(power.duration_battery.map(|d| duration_formatter(d)));
    self.ui.end_row();
  }

  fn hydrogen_row(&mut self, label: impl Into<WidgetText>, hydrogen_formatter: impl Fn(f64) -> String, duration_formatter: impl Fn(f64) -> String, hydrogen: &HydrogenCalculated) {
    self.ui.label(label);
    self.right_align_value(hydrogen_formatter(hydrogen.consumption));
    self.right_align_value(hydrogen_formatter(hydrogen.balance));
    self.right_align_optional_value(hydrogen.duration_tank.map(|d| duration_formatter(d)));
    if let Some(duration) = hydrogen.duration_engine {
      self.right_align_value(duration_formatter(duration));
    } else {
      self.empty_label();
    }
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