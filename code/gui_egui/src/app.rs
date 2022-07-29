use std::borrow::Borrow;
use std::fmt::Display;
use std::ops::{Deref, DerefMut, RangeInclusive};

use eframe::epaint::Rgba;
use eframe::Frame;
use egui::{Align, Button, CollapsingHeader, CollapsingResponse, Color32, ComboBox, Context, DragValue, Grid, InnerResponse, Layout, menu, Response, ScrollArea, SidePanel, TextFormat, TextStyle, TopBottomPanel, Ui, Visuals, WidgetText};
use egui::emath::Numeric;
use egui::text::LayoutJob;
use thousands::{Separable, SeparatorPolicy};

use secalc_core::data::blocks::GridSize;
use secalc_core::data::Data;
use secalc_core::grid::{AccelerationCalculated, CountPerDirection, Direction, GridCalculated, GridCalculator, HydrogenCalculated, PerDirection, PowerCalculated};

// App

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
  #[serde(skip)] data: Data,
  #[serde(skip)] calculator_default: GridCalculator,
  #[serde(skip)] calculated: GridCalculated,
  #[serde(skip)] number_separator_policy: SeparatorPolicy<'static>,

  calculator: GridCalculator,
  grid_size: GridSize,
}

impl App {
  pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
    let mut app = if let Some(storage) = ctx.storage {
      eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
    } else {
      Self::default()
    };
    app.calculate();
    app
  }

  fn calculate(&mut self) {
    self.calculated = self.calculator.calculate(&self.data);
  }
}

impl Default for App {
  fn default() -> Self {
    let data = {
      let bytes: &[u8] = include_bytes!("../../../data/data.json");
      Data::from_json(bytes).expect("Cannot read data")
    };
    let calculator_default = GridCalculator::default();
    let calculated = GridCalculated::default();
    let number_separator_policy = SeparatorPolicy {
      separator: "·",
      groups: &[3],
      digits: thousands::digits::ASCII_DECIMAL,
    };

    let calculator = GridCalculator::default();
    let grid_size = GridSize::default();

    Self { data, calculator, calculator_default, calculated, number_separator_policy, grid_size }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    TopBottomPanel::top("Menu Panel")
      .resizable(false)
      .show(ctx, |ui| {
        menu::bar(ui, |ui| {
          ui.menu_button("Grid", |ui| {
            ui.button("Save [nyi]")
          });
        });
      });
    SidePanel::left("Calculator Panel")
      .resizable(false)
      .min_width(512.0)
      .show(ctx, |ui| {
        ScrollArea::vertical()
          .auto_shrink([false; 2])
          .show(ui, |ui| {
            if self.show_calculator(ui) {
              self.calculate();
            }
          });
      });
    SidePanel::left("Results Panel")
      .resizable(false)
      .min_width(512.0)
      .show(ctx, |ui| {
        ScrollArea::vertical()
          .auto_shrink([false; 2])
          .show(ui, |ui| {
            self.show_results(ui, ctx);
          });
      });
  }

  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    eframe::set_value(storage, eframe::APP_KEY, self);
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
      ComboBox::from_id_source("Grid Size")
        .selected_text(format!("{}", self.grid_size))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.grid_size, GridSize::Small, "Small");
          ui.selectable_value(&mut self.grid_size, GridSize::Large, "Large");
        });
      ui.open_header_with_grid("Storage", |ui| {
        let mut ui = CalculatorUi::new(ui, self.number_separator_policy, block_edit_size);
        for block in self.data.blocks.containers.values().filter(|b| b.size == self.grid_size) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        for block in self.data.blocks.cockpits.values().filter(|b| b.size == self.grid_size && b.has_inventory) {
          ui.edit_count_row(block.name(&self.data.localization), self.calculator.blocks.entry(block.id.clone()).or_default());
        }
        changed |= ui.changed
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
    });
    changed
  }

  fn show_results(&mut self, ui: &mut Ui, ctx: &Context) {
    ui.open_header_with_grid("Mass", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      ui.show_row("Empty", format!("{} kg", self.calculated.total_mass_empty.round()));
      ui.show_row("Filled", format!("{} kg", self.calculated.total_mass_filled.round()));
    });
    ui.open_header_with_grid("Volume", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      ui.show_row("Any", format!("{} L", self.calculated.total_volume_any));
      ui.show_row("Ore", format!("{} L", self.calculated.total_volume_ore));
      ui.show_row("Ice", format!("{} L", self.calculated.total_volume_ice));
      ui.show_row("Ore-only", format!("{} L", self.calculated.total_volume_ore_only));
      ui.show_row("Ice-only", format!("{} L", self.calculated.total_volume_ice_only));
    });
    ui.open_header_with_grid("Items", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      ui.show_row("Ore", format!("{} #", self.calculated.total_items_ore.round()));
      ui.show_row("Ice", format!("{} #", self.calculated.total_items_ice.round()));
      ui.show_row("Steel Plate", format!("{} #", self.calculated.total_items_steel_plate.round()));
    });
    ui.open_header_with_grid("Force", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      for direction in Direction::iter() {
        ui.force_row(*direction, &self.calculated.acceleration);
      }
    });
    ui.open_header_with_grid("Acceleration", |ui| {
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
      ui.label("");
      ui.label("Filled");
      ui.label("");
      ui.label("Empty");
      ui.label("");
      ui.label("");
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
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
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
      let mut ui = CalculatedUi::new(ui, self.number_separator_policy);
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
    self.ui.with_layout(Layout::right_to_left(Align::Center), |ui| ui.label(label));
  }

  fn empty_label(&mut self) {
    self.right_align_label("-");
  }


  fn force_row(&mut self, direction: Direction, acceleration: &PerDirection<AccelerationCalculated>) {
    self.right_align_label(format!("{}", direction));
    self.right_align_value(format!("{:.2}", acceleration.get(direction).force / 1000.0));
    self.label("kN");
    self.ui.end_row();
  }


  fn acceleration_row(&mut self, direction: Direction, acceleration: &PerDirection<AccelerationCalculated>, ctx: &Context) {
    self.right_align_label(format!("{}", direction));
    self.right_align_optional_value(acceleration.get(direction).acceleration_filled_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_filled_no_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_empty_gravity.map(|a| format!("{:.2}", a)));
    self.right_align_optional_value(acceleration.get(direction).acceleration_empty_no_gravity.map(|a| format!("{:.2}", a)));
    self.acceleration_label(ctx);
    self.ui.end_row();
  }

  fn acceleration_label(&mut self, ctx: &Context) {
    let mut acceleration = LayoutJob::default();
    acceleration.append("m/s", 0.0, TextFormat { color: Color32::BLACK, ..TextFormat::default() });
    acceleration.append("2", 0.0, TextFormat { font_id: TextStyle::Small.resolve(&ctx.style()), color: Color32::BLACK, valign: Align::Min, ..TextFormat::default() });
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