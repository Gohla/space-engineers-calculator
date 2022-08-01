use eframe::epaint::Rgba;
use eframe::Frame;
use egui::{Align, Align2, Button, CentralPanel, Context, Layout, menu, ScrollArea, Separator, TopBottomPanel, Visuals, Window};
use egui_extras::{Size, StripBuilder};
use thousands::SeparatorPolicy;

use secalc_core::data::blocks::GridSize;
use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

mod calculator;
mod result;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
  #[serde(skip)] data: Data,
  #[serde(skip)] number_separator_policy: SeparatorPolicy<'static>,
  #[serde(skip)] calculator_default: GridCalculator,
  #[serde(skip)] calculated: GridCalculated,

  #[serde(skip)] enable_gui: bool,
  #[serde(skip)] show_reset_confirm_window: bool,
  #[serde(skip)] show_debug_gui_settings_window: bool,
  #[serde(skip)] show_debug_gui_inspection_window: bool,
  #[serde(skip)] show_debug_gui_memory_window: bool,

  calculator: GridCalculator,
  grid_size: GridSize,
  dark_mode: bool,
}

impl App {
  pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
    let mut app = if let Some(storage) = ctx.storage {
      let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
      app.set_dark_mode(app.dark_mode, &ctx.egui_ctx);
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

  fn set_dark_mode(&mut self, dark_mode: bool, ctx: &Context) {
    self.dark_mode = dark_mode;
    let mut style = (*ctx.style()).clone(); // Clone entire style, not the Arc.
    style.visuals = if dark_mode { Visuals::dark() } else { Visuals::light() };
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

      enable_gui: true,
      show_reset_confirm_window: false,
      show_debug_gui_settings_window: false,
      show_debug_gui_inspection_window: false,
      show_debug_gui_memory_window: false,

      calculator: GridCalculator::default(),
      grid_size: GridSize::default(),
      dark_mode: false,
    }
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, frame: &mut Frame) {
    // Top panel
    TopBottomPanel::top("Top Panel")
      .resizable(false)
      .show(ctx, |ui| {
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
                  self.set_dark_mode(false, ctx);
                }
              } else {
                if ui.add(Button::new("🌙")).clicked() {
                  self.set_dark_mode(true, ctx);
                }
              }
            });
          });
        });
      });

    // Main content
    CentralPanel::default()
      .show(ctx, |ui| {
        ui.add_enabled_ui(self.enable_gui, |ui| {
          let layout = Layout::top_down(Align::LEFT);
          StripBuilder::new(ui)
            .cell_layout(layout)
            .size(Size::remainder())
            .size(Size::exact(1.0))
            .size(Size::remainder())
            .horizontal(|mut strip| {
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
              strip.cell(|ui| {
                ui.add(Separator::default().spacing(0.0).vertical());
              });
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
