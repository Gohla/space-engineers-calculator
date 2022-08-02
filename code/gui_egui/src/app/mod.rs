use eframe::epaint::Rgba;
use egui::{Align, Align2, Button, CentralPanel, Color32, Context, Frame, Layout, menu, Rounding, ScrollArea, Separator, Style, Vec2, Visuals, Window};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use thousands::SeparatorPolicy;

use secalc_core::data::blocks::GridSize;
use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

mod calculator;
mod result;
mod settings;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
  #[serde(skip)] data: Data,
  #[serde(skip)] number_separator_policy: SeparatorPolicy<'static>,
  #[serde(skip)] calculator_default: GridCalculator,
  #[serde(skip)] calculated: GridCalculated,
  #[serde(skip)] style_default: Style,

  #[serde(skip)] enable_gui: bool,
  #[serde(skip)] show_reset_confirm_window: bool,

  #[serde(skip)] show_settings_window: bool,
  #[serde(skip)] show_debug_gui_settings_window: bool,
  #[serde(skip)] show_debug_gui_inspection_window: bool,
  #[serde(skip)] show_debug_gui_memory_window: bool,

  dark_mode: bool,
  font_size_modifier: i32,
  increase_contrast: bool,

  calculator: GridCalculator,
  grid_size: GridSize,
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
      show_reset_confirm_window: false,
      increase_contrast: false,

      show_settings_window: false,
      show_debug_gui_settings_window: false,
      show_debug_gui_inspection_window: false,
      show_debug_gui_memory_window: false,

      dark_mode: false,
      font_size_modifier: 0,

      calculator: GridCalculator::default(),
      grid_size: GridSize::default(),
    }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    let central_frame = Frame::none().fill(ctx.style().visuals.window_fill()).inner_margin(Margin::same(4.0));
    CentralPanel::default().frame(central_frame).show(ctx, |ui| {
      ui.add_enabled_ui(self.enable_gui, |ui| {
        StripBuilder::new(ui)
          .size(Size::exact(20.0))
          .size(Size::exact(1.0))
          .size(Size::remainder())
          .vertical(|mut strip| {
            // Top panel with menu
            strip.cell(|ui| {
              ui.add_enabled_ui(self.enable_gui, |ui| {
                menu::bar(ui, |ui| {
                  ui.menu_button("Grid", |ui| {
                    if ui.button("Save").clicked() {
                      if let Some(storage) = frame.storage_mut() {
                        self.save(storage);
                      }
                      ui.close_menu();
                    }
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
                  });
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
                  ui.with_layout(Layout::right_to_left(), |ui| {
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
                    ScrollArea::vertical()
                      .id_source("Calculator Scroll")
                      .auto_shrink([false; 2])
                      .show(ui, |ui| {
                        if self.show_calculator(ui) {
                          self.calculate();
                        }
                      });
                  });
                  // Vertical line
                  strip.cell(|ui| { ui.add(Separator::default().spacing(0.0).vertical()); });
                  // Result (calculated)
                  strip.cell(|ui| {
                    ScrollArea::vertical()
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

    // Modal windows
    if self.show_reset_confirm_window {
      Window::new("Confirm Reset")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .auto_sized()
        .show(ctx, |ui| {
          ui.label("Are you sure you want to reset all grid data (left-side panel) to their defaults? Any unsaved data will be lost.");
          ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
              self.enable_gui = true;
              self.show_reset_confirm_window = false;
              self.calculator = self.calculator_default.clone();
              self.calculate();
            }
            if ui.button("Cancel").clicked() {
              self.enable_gui = true;
              self.show_reset_confirm_window = false;
            }
          });
        });
    }

    // Non-modal windows
    let mut show_settings_window = self.show_settings_window;
    Window::new("Settings")
      .open(&mut show_settings_window)
      .show(ctx, |ui| { self.show_settings(ui, ctx) });
    self.show_settings_window = show_settings_window;
    Window::new("GUI Settings")
      .open(&mut self.show_debug_gui_settings_window)
      .show(ctx, |ui| { ctx.settings_ui(ui) });
    Window::new("GUI Inspection")
      .open(&mut self.show_debug_gui_inspection_window)
      .show(ctx, |ui| { ctx.inspection_ui(ui) });
    Window::new("GUI Memory")
      .open(&mut self.show_debug_gui_memory_window)
      .show(ctx, |ui| { ctx.memory_ui(ui) });
  }

  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    eframe::set_value(storage, eframe::APP_KEY, self);
  }

  fn clear_color(&self, visuals: &Visuals) -> Rgba {
    visuals.window_fill().into()
  }
}
