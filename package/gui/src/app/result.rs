use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

use egui::{Align, Context, Layout, RichText, TextFormat, TextStyle, Ui, Vec2, WidgetText};
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
      ui.vertical(|ui| {
        ui.open_collapsing_header_with_grid("Wheel Force", |ui| {
          let mut ui = ResultUi::new(ui, self.number_separator_policy);
          ui.show_row("Force", format!("{:.2}", self.calculated.wheel_force / 1000.0), "kN");
        });
      });
    });
    ui.horizontal(|ui| {
      ui.open_collapsing_header_with_grid("Thruster Acceleration & Force", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.label("Direction");
        ui.vertical_separator_unpadded();
        ui.label("Filled");
        ui.label("");
        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Empty");
        ui.label("");
        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Force");
        ui.end_row();

        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Gravity");
        ui.vertical_separator_unpadded();
        ui.label("No grav.");
        ui.vertical_separator_unpadded();
        ui.label("Gravity");
        ui.vertical_separator_unpadded();
        ui.label("No grav.");
        ui.vertical_separator_unpadded();
        ui.label("");
        ui.end_row();

        for direction in Direction::items() {
          ui.acceleration_row(direction, &self.calculated.thruster_acceleration, ctx);
        }
      });
    });
    ui.open_collapsing_header("Power", |ui| {
      ui.grid_unstriped("Power Grid 1", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_row("Generation:", format!("{:.2}", self.calculated.power_generation), "MW");
        ui.horizontal_separator_unpadded();
        ui.horizontal_separator_unpadded();
        ui.end_row();
      });
      ui.allocate_space(Vec2::new(0.0, 1.0));
      ui.grid("Power Grid 2", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.label("Group Name");
        ui.vertical_separator_unpadded();
        ui.label("Consumption");
        ui.label("");
        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Balance");
        ui.vertical_separator_unpadded();
        ui.label("Duration");
        ui.label("");
        ui.end_row();

        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Group");
        ui.vertical_separator_unpadded();
        ui.label("Total");
        ui.vertical_separator_unpadded();
        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label(RichText::new("Batteries").underline())
          .on_hover_text_at_pointer("Duration until batteries are empty at the total consumption in the row. Does not take into account charging the batteries via any means.");
        ui.vertical_separator_unpadded();
        ui.label(RichText::new("Engines").underline())
          .on_hover_text_at_pointer("Duration until hydrogen engines are empty at the total consumption in the row. Does not take into account filling the engines via generators or tanks.");
        ui.end_row();

        let power_formatter = |v| format!("{:.2}", v);
        ui.power_row("Idle", power_formatter, &self.calculated.power_idle);
        ui.power_row("Charge Railguns", power_formatter, &self.calculated.power_railgun_charge);
        ui.power_row("+ Utility", power_formatter, &self.calculated.power_upto_utility);
        ui.power_row("+ Wheel Suspensions", power_formatter, &self.calculated.power_upto_wheel_suspension);
        ui.power_row("+ Charge Jump Drives", power_formatter, &self.calculated.power_upto_jump_drive_charge);
        ui.power_row("+ O2/H2 Generators", power_formatter, &self.calculated.power_upto_generator);
        ui.power_row("+ Up/Down Thrusters", power_formatter, &self.calculated.power_upto_up_down_thruster);
        ui.power_row("+ Front/Back Thrusters", power_formatter, &self.calculated.power_upto_front_back_thruster);
        ui.power_row("+ Left/Right Thrusters", power_formatter, &self.calculated.power_upto_left_right_thruster);
        ui.power_row("+ Charge Batteries", power_formatter, &self.calculated.power_upto_battery_charge);
      });
    });
    ui.horizontal(|ui| {
      ui.open_collapsing_header_with_grid("Railgun", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        let railgun = self.calculated.railgun.as_ref();
        ui.show_optional_row("Capacity:", railgun.map(|r| format!("{:.2}", r.capacity)), "MWh");
        ui.show_optional_row("Maximum Input:", railgun.map(|r| format!("{:.2}", r.maximum_input)), "MW");
        ui.show_optional_duration_row("Charge Duration:", railgun.and_then(|r| r.charge_duration));
      });
      ui.open_collapsing_header_with_grid("Jump Drive", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        let jump_drive = self.calculated.jump_drive.as_ref();
        ui.show_optional_row("Capacity:", jump_drive.map(|j| format!("{:.2}", j.capacity)), "MWh");
        ui.show_optional_duration_row("Charge Duration:", jump_drive.and_then(|j| j.charge_duration));
        ui.show_optional_row("Maximum Input:", jump_drive.map(|j| format!("{:.2}", j.maximum_input)), "MW");
        ui.show_optional_row("Max Range (Empty):", jump_drive.map(|j| format!("{:.2}", j.max_distance_empty)), "km");
        ui.show_optional_row("Max Range (Filled):", jump_drive.map(|j| format!("{:.2}", j.max_distance_filled)), "km");
      });
      ui.open_collapsing_header_with_grid("Battery", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        let battery = self.calculated.battery.as_ref();
        ui.show_optional_row("Capacity:", battery.map(|b| format!("{:.2}", b.capacity)), "MWh");
        ui.show_optional_row("Maximum Input:", battery.map(|b| format!("{:.2}", b.maximum_input)), "MW");
        ui.show_optional_row("Maximum Output:", battery.map(|b| format!("{:.2}", b.maximum_output)), "MW");
        ui.show_optional_duration_row("Charge Duration:", battery.and_then(|b| b.charge_duration));
      });
    });
    ui.open_collapsing_header("Hydrogen", |ui| {
      ui.grid_unstriped("Hydrogen Grid 1", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.show_row("Generation:", format!("{}", self.calculated.hydrogen_generation.round()), "L/s");
        ui.horizontal_separator_unpadded();
        ui.horizontal_separator_unpadded();
        ui.end_row();
      });
      ui.allocate_space(Vec2::new(0.0, 1.0));
      ui.grid("Hydrogen Grid 2", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        ui.label("Group Name");
        ui.vertical_separator_unpadded();
        ui.label("Consumption");
        ui.label("");
        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Balance");
        ui.label("");
        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Duration");
        ui.end_row();

        ui.label("");
        ui.vertical_separator_unpadded();
        ui.label("Group");
        ui.vertical_separator_unpadded();
        ui.label("Total");
        ui.vertical_separator_unpadded();
        ui.label(RichText::new("w/o Tanks").underline())
          .on_hover_text_at_pointer("Hydrogen balance in the row, without tanks providing hydrogen.");
        ui.vertical_separator_unpadded();
        ui.label(RichText::new("w Tanks").underline())
          .on_hover_text_at_pointer("Hydrogen balance in the row, with tanks providing hydrogen.");
        ui.vertical_separator_unpadded();
        ui.label(RichText::new("Tanks").underline())
          .on_hover_text_at_pointer("Duration until hydrogen tanks are empty at the total consumption in the row. Does not take into account filling the tank via generators or other tanks.");
        ui.end_row();

        let hydrogen_formatter = |v| format!("{:.2}", v);
        ui.hydrogen_row("Idle", hydrogen_formatter, &self.calculated.hydrogen_idle);
        ui.hydrogen_row("Fill Engines", hydrogen_formatter, &self.calculated.hydrogen_engine_fill);
        ui.hydrogen_row("+ Up/Down Thrusters", hydrogen_formatter, &self.calculated.hydrogen_upto_up_down_thruster);
        ui.hydrogen_row("+ Front/Back Thrusters", hydrogen_formatter, &self.calculated.hydrogen_upto_front_back_thruster);
        ui.hydrogen_row("+ Left/Right Thrusters", hydrogen_formatter, &self.calculated.hydrogen_upto_left_right_thruster);
        ui.hydrogen_row("+ Fill Tanks", hydrogen_formatter, &self.calculated.hydrogen_upto_tank_fill);
      });
    });
    ui.horizontal(|ui| {
      ui.open_collapsing_header_with_grid("Hydrogen Tank", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        let hydrogen_tank = self.calculated.hydrogen_tank.as_ref();
        ui.show_optional_row("Capacity:", hydrogen_tank.map(|c| format!("{}", c.capacity.round())), "L");
        ui.show_optional_row("Maximum Input:", hydrogen_tank.map(|c| format!("{}", c.maximum_input.round())), "L/s");
        ui.show_optional_row("Maximum Output:", hydrogen_tank.map(|c| format!("{}", c.maximum_output.round())), "L/s");
        ui.show_optional_duration_row("Fill Duration:", hydrogen_tank.and_then(|t| t.fill_duration));
      });
      ui.open_collapsing_header_with_grid("Hydrogen Engine", |ui| {
        let mut ui = ResultUi::new(ui, self.number_separator_policy);
        let hydrogen_engine = self.calculated.hydrogen_engine.as_ref();
        ui.show_optional_row("Capacity:", hydrogen_engine.map(|c| format!("{}", c.capacity.round())), "L");
        ui.show_optional_row("Maximum Fuel Consumption:", hydrogen_engine.map(|c| format!("{}", c.maximum_fuel_consumption.round())), "L/s");
        ui.show_optional_row("Maximum Output:", hydrogen_engine.map(|c| format!("{:.2}", c.maximum_output)), "MW");
        ui.show_optional_row("Maximum Refilling Input:", hydrogen_engine.map(|c| format!("{}", c.maximum_refilling_input.round())), "L/s");
        ui.show_optional_duration_row("Fill Duration:", hydrogen_engine.and_then(|e| e.fill_duration));
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
    self.ui.with_layout(Layout::right_to_left(Align::Center), |ui| ui.label(label));
  }


  fn right_align_value_with_unit(&mut self, value: impl Borrow<str>, unit: impl Into<WidgetText>) {
    self.ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
    let acceleration_label = self.acceleration_layout_job(ctx);
    self.right_align_label(format!("{}", direction));
    self.ui.vertical_separator_unpadded();
    self.right_align_optional_value_with_unit(acceleration.get(direction).acceleration_filled_gravity.map(|a| format!("{:.2}", a)), acceleration_label.clone());
    self.ui.vertical_separator_unpadded();
    self.right_align_optional_value_with_unit(acceleration.get(direction).acceleration_filled_no_gravity.map(|a| format!("{:.2}", a)), acceleration_label.clone());
    self.ui.vertical_separator_unpadded();
    self.right_align_optional_value_with_unit(acceleration.get(direction).acceleration_empty_gravity.map(|a| format!("{:.2}", a)), acceleration_label.clone());
    self.ui.vertical_separator_unpadded();
    self.right_align_optional_value_with_unit(acceleration.get(direction).acceleration_empty_no_gravity.map(|a| format!("{:.2}", a)), acceleration_label);
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(format!("{:.2}", acceleration.get(direction).force / 1000.0), "kN");
    self.ui.end_row();
  }

  fn acceleration_layout_job(&mut self, ctx: &Context) -> LayoutJob {
    let mut acceleration = LayoutJob::default();
    let color = ctx.style().visuals.text_color();
    acceleration.append("m/s", 0.0, TextFormat { font_id: TextStyle::Body.resolve(&ctx.style()), color, ..TextFormat::default() });
    acceleration.append("2", 0.0, TextFormat { font_id: TextStyle::Small.resolve(&ctx.style()), color, valign: Align::Min, ..TextFormat::default() });
    acceleration
  }

  fn power_row(&mut self, label: impl Into<WidgetText>, power_formatter: impl Fn(f64) -> String, power: &PowerCalculated) {
    self.ui.label(label);
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(power_formatter(power.consumption), "MW");
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(power_formatter(power.total_consumption), "MW");
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(power_formatter(power.balance), "MW");
    self.ui.vertical_separator_unpadded();
    self.right_align_optional_duration(power.battery_duration);
    self.ui.vertical_separator_unpadded();
    self.right_align_optional_duration(power.engine_duration);
    self.ui.end_row();
  }

  fn hydrogen_row(&mut self, label: impl Into<WidgetText>, hydrogen_formatter: impl Fn(f64) -> String, hydrogen: &HydrogenCalculated) {
    self.ui.label(label);
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(hydrogen_formatter(hydrogen.consumption), "L/s");
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(hydrogen_formatter(hydrogen.total_consumption), "L/s");
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(hydrogen_formatter(hydrogen.balance_without_tank), "L/s");
    self.ui.vertical_separator_unpadded();
    self.right_align_value_with_unit(hydrogen_formatter(hydrogen.balance_with_tank), "L/s");
    self.ui.vertical_separator_unpadded();
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
