use std::collections::{HashMap, HashSet};

use eframe::epaint::Rgba;
use egui::{Align, Button, CentralPanel, Color32, Context, Frame, Layout, menu, Rounding, ScrollArea, Separator, Style, Vec2, Visuals};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use thousands::SeparatorPolicy;

use secalc_core::data::blocks::GridSize;
use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

mod calculator;
mod result;
mod window;
mod save_load;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
  #[serde(skip)] data: Data,
  #[serde(skip)] number_separator_policy: SeparatorPolicy<'static>,
  #[serde(skip)] calculator_default: GridCalculator,
  #[serde(skip)] calculated: GridCalculated,
  #[serde(skip)] style_default: Style,

  #[serde(skip)] enable_gui: bool,
  #[serde(skip)] show_load_window: bool,
  #[serde(skip)] show_load_confirm_window: bool,
  #[serde(skip)] show_delete_confirm_window: Option<String>,
  #[serde(skip)] show_save_as_window: Option<String>,
  #[serde(skip)] show_save_as_confirm_window: Option<String>,
  #[serde(skip)] show_reset_confirm_window: bool,

  #[serde(skip)] show_settings_window: bool,
  #[serde(skip)] show_about_window: bool,
  #[serde(skip)] show_debug_gui_settings_window: bool,
  #[serde(skip)] show_debug_gui_inspection_window: bool,
  #[serde(skip)] show_debug_gui_memory_window: bool,

  first_time: bool,
  enabled_mod_ids: HashSet<u64>,
  dark_mode: bool,
  font_size_modifier: i32,
  increase_contrast: bool,

  calculator: GridCalculator,
  grid_size: GridSize,

  saved_calculators: HashMap<String, GridCalculator>,
  current_calculator: Option<String>,
  current_calculator_saved: bool,
}

impl App {
  pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
    let mut app = if let Some(storage) = ctx.storage {
      let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
      app.apply_style(&ctx.egui_ctx);
      app
    } else {
      let mut app = Self::default();
      app.dark_mode = ctx.egui_ctx.style().visuals.dark_mode;
      app
    };
    app.calculate();
    app
  }

  fn calculate(&mut self) {
    self.calculated = self.calculator.calculate(&self.data);
  }

  fn apply_style(&mut self, ctx: &Context) {
    let mut style = (*ctx.style()).clone(); // Clone entire style, not the Arc.
    // Text style
    for (text_style, font_id) in style.text_styles.iter_mut() {
      if let Some(default_font_id) = self.style_default.text_styles.get(text_style) {
        font_id.size = default_font_id.size + self.font_size_modifier as f32;
      }
    }
    // Spacing
    style.spacing.item_spacing = Vec2::new(8.0, 2.0);
    style.spacing.button_padding = Vec2::new(4.0, 2.0);
    // Visuals
    let mut visuals = if self.dark_mode {
      let mut dark = Visuals::dark();
      if self.increase_contrast {
        dark.override_text_color = Some(Color32::from_rgb(210, 210, 210));
        dark.widgets.noninteractive.bg_fill = Color32::from_rgb(20, 20, 20);
      }
      dark
    } else {
      let mut light = Visuals::light();
      if self.increase_contrast {
        light.override_text_color = Some(Color32::from_rgb(0, 0, 0));
        light.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
      }
      light
    };
    visuals.widgets.noninteractive.rounding = Rounding::none();
    visuals.widgets.inactive.rounding = Rounding::none();
    visuals.widgets.hovered.rounding = Rounding::none();
    visuals.widgets.active.rounding = Rounding::none();
    visuals.widgets.open.rounding = Rounding::none();
    visuals.window_rounding = Rounding::none();
    style.visuals = visuals;
    // Apply style
    ctx.set_style(style);
  }
}

impl Default for App {
  fn default() -> Self {
    let data = {
      let bytes: &[u8] = include_bytes!("../../../../data/data.json");
      Data::from_json(bytes).expect("Cannot read data")
    };
    let number_separator_policy = SeparatorPolicy {
      separator: "·",
      groups: &[3],
      digits: thousands::digits::ASCII_DECIMAL,
    };
    Self {
      data,
      number_separator_policy,
      calculator_default: GridCalculator::default(),
      calculated: GridCalculated::default(),
      style_default: Style::default(),

      enable_gui: true,
      show_load_window: false,
      show_load_confirm_window: false,
      show_delete_confirm_window: None,
      show_save_as_window: None,
      show_save_as_confirm_window: None,
      show_reset_confirm_window: false,

      show_settings_window: false,
      show_about_window: false,
      show_debug_gui_settings_window: false,
      show_debug_gui_inspection_window: false,
      show_debug_gui_memory_window: false,

      first_time: true,

      enabled_mod_ids: Default::default(),
      dark_mode: true,
      font_size_modifier: 4,
      increase_contrast: false,

      calculator: GridCalculator::default(),
      grid_size: GridSize::default(),

      saved_calculators: Default::default(),
      current_calculator: None,
      current_calculator_saved: false,
    }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    let central_frame = Frame::none().fill(ctx.style().visuals.window_fill()).inner_margin(Margin::same(4.0));
    CentralPanel::default().frame(central_frame).show(ctx, |ui| {
      ui.add_enabled_ui(self.enable_gui, |ui| {
        StripBuilder::new(ui)
          .size(Size::exact(20.0 + (self.font_size_modifier.max(0) as f32 / 2.0)))
          .size(Size::exact(1.0))
          .size(Size::remainder())
          .vertical(|mut strip| {
            // Top panel with menu
            strip.cell(|ui| {
              ui.add_enabled_ui(self.enable_gui, |ui| {
                menu::bar(ui, |ui| {
                  ui.menu_button("Grid", |ui| {
                    if ui.button("Save").clicked() {
                      if let Some(name) = &self.current_calculator {
                        self.saved_calculators.insert(name.clone(), self.calculator.clone());
                        self.current_calculator_saved = true;
                      } else {
                        self.enable_gui = false;
                        self.show_save_as_window = Some(String::new());
                      }
                      if let Some(storage) = frame.storage_mut() {
                        self.save(storage);
                      }
                      ui.close_menu();
                    }
                    if ui.button("Save As").clicked() {
                      self.enable_gui = false;
                      let name = if let Some(name) = &self.current_calculator {
                        name.clone()
                      } else {
                        String::new()
                      };
                      self.show_save_as_window = Some(name);
                      ui.close_menu();
                    }
                    if ui.button("Load").clicked() {
                      if !self.current_calculator_saved {
                        self.enable_gui = false;
                        self.show_load_confirm_window = true;
                      } else {
                        self.enable_gui = false;
                        self.show_load_window = true;
                      }
                      ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Reset").clicked() {
                      self.enable_gui = false;
                      self.show_reset_confirm_window = true;
                      ui.close_menu();
                    }
                  });
                  ui.menu_button("Window", |ui| {
                    if ui.checkbox(&mut self.show_settings_window, "Settings").clicked() {
                      ui.close_menu();
                    }
                    if ui.checkbox(&mut self.show_about_window, "About").clicked() {
                      ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Debug", |ui| {
                      if ui.checkbox(&mut self.show_debug_gui_settings_window, "GUI Settings").clicked() {
                        ui.close_menu();
                      }
                      if ui.checkbox(&mut self.show_debug_gui_inspection_window, "GUI Inspections").clicked() {
                        ui.close_menu();
                      }
                      if ui.checkbox(&mut self.show_debug_gui_memory_window, "GUI Memory").clicked() {
                        ui.close_menu();
                      }
                    });
                  });
                  ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.dark_mode {
                      if ui.add(Button::new("☀")).clicked() {
                        self.dark_mode = false;
                        self.apply_style(ctx);
                      }
                    } else {
                      if ui.add(Button::new("🌙")).clicked() {
                        self.dark_mode = true;
                        self.apply_style(ctx);
                      }
                    }
                  });
                });
              });
            });
            // Horizontal line
            strip.cell(|ui| { ui.add(Separator::default().spacing(0.0).horizontal()); });
            // Main content panel
            strip.strip(|strip_builder| {
              let layout = Layout::top_down(Align::LEFT);
              strip_builder
                .cell_layout(layout)
                .size(Size::remainder())
                .size(Size::exact(1.0))
                .size(Size::remainder())
                .horizontal(|mut strip| {
                  // Calculator
                  strip.cell(|ui| {
                    ScrollArea::both()
                      .id_source("Calculator Scroll")
                      .auto_shrink([false; 2])
                      .show(ui, |ui| {
                        if self.show_calculator(ui) {
                          self.calculate();
                          self.current_calculator_saved = false;
                        }
                      });
                  });
                  // Vertical line
                  strip.cell(|ui| { ui.add(Separator::default().spacing(0.0).vertical()); });
                  // Result (calculated)
                  strip.cell(|ui| {
                    ScrollArea::both()
                      .id_source("Result Scroll")
                      .auto_shrink([false; 2])
                      .show(ui, |ui| {
                        self.show_results(ui, ctx);
                      });
                  });
                });
            });
          });
      });
    });
    // Windows
    self.show_save_load_reset_windows(ctx, frame);
    self.show_settings_windows(ctx, frame);
  }

  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    eframe::set_value(storage, eframe::APP_KEY, self);
  }

  fn clear_color(&self, visuals: &Visuals) -> Rgba {
    visuals.window_fill().into()
  }
}
