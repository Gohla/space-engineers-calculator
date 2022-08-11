use eframe::App as AppT;
use egui::{Align2, Context, DragValue, Grid, RichText, ScrollArea, Window};

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_settings_windows(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    // Return immediately if GUI is disabled. Using disabled windows is not possible as they can 
    // overlap on top of modal windows.
    if !self.enable_gui { return; }

    self.show_settings_window(ctx, frame);
    self.show_about_window(ctx);

    // EGUI Debug windows
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

  fn show_settings_window(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    let mut show = self.show_settings_window;
    let mut close = false;
    Window::new("Settings")
      .open(&mut show)
      .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
      .collapsible(false)
      .fixed_size([400.0, 400.0])
      .show(ctx, |ui| {
        ScrollArea::vertical()
          .auto_shrink([false; 2])
          .show(ui, |ui| {
            ui.open_collapsing_header_with_grid("GUI", |ui| {
              ui.label("Dark mode");
              if ui.checkbox(&mut self.dark_mode, "").changed() {
                self.apply_style(ctx);
              }
              ui.end_row();
              ui.label("Font size modifier");
              if ui.add(DragValue::new(&mut self.font_size_modifier).clamp_range(-4..=16)).changed() {
                self.apply_style(ctx);
              }
              ui.end_row();
              ui.label("Increase contrast");
              if ui.checkbox(&mut self.increase_contrast, "").changed() {
                self.apply_style(ctx);
              }
              ui.end_row();
            });
            ui.open_collapsing_header_with_grid("Mods", |ui| {
              for m in self.data.mods.iter() {
                let id = m.0;
                ui.url_link(&m.1, format!("https://steamcommunity.com/workshop/filedetails/?id={}", id));
                let mut enabled = self.enabled_mod_ids.contains(&m.0);
                if ui.checkbox(&mut enabled, "").changed() {
                  if enabled {
                    self.enabled_mod_ids.insert(id);
                  } else {
                    self.enabled_mod_ids.remove(&id);
                  }
                }
                ui.end_row();
              }
            });
          });
        ui.separator();
        ui.horizontal(|ui| {
          if ui.button("Save").clicked() {
            if let Some(storage) = frame.storage_mut() {
              self.save(storage);
            }
          }
          if ui.button("Close").clicked() {
            close = true;
          }
        });
      });
    self.show_settings_window = show && !close;
  }

  fn show_about_window(&mut self, ctx: &Context) {
    if self.first_time {
      self.show_about_window = true;
      self.first_time = false;
    }

    let mut show = self.show_about_window;
    let mut close = false;
    Window::new("About")
      .open(&mut show)
      .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
      .collapsible(false)
      .fixed_size([700.0, 600.0])
      .show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
          ui.label(RichText::new("About").strong());
          ui.label(ABOUT_TEXT);
        });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
          ui.label(RichText::new("Storage").strong());
          ui.label(STORAGE_TEXT);
        });
        ui.separator();
        Grid::new("Links Grid").show(ui, |ui| {
          ui.label(RichText::new("Home").strong());
          ui.url_link("github.com/Gohla/space-engineers-calculator", "https://github.com/Gohla/space-engineers-calculator");
          ui.end_row();
          ui.label(RichText::new("Questions").strong());
          ui.url_link("github.com/Gohla/space-engineers-calculator/discussions/categories/q-a", "https://github.com/Gohla/space-engineers-calculator/discussions/categories/q-a");
          ui.end_row();
          ui.label(RichText::new("Ideas").strong());
          ui.url_link("github.com/Gohla/space-engineers-calculator/discussions/categories/ideas", "https://github.com/Gohla/space-engineers-calculator/discussions/categories/ideas");
          ui.end_row();
          ui.label(RichText::new("Report Issue").strong());
          ui.url_link("github.com/Gohla/space-engineers-calculator/issues", "https://github.com/Gohla/space-engineers-calculator/issues");
          ui.end_row();
        });
        ui.separator();
        ui.horizontal(|ui| {
          if ui.button("Close").clicked() {
            close = true;
          }
        });
      });
    self.show_about_window = show && !close;
  }
}

const ABOUT_TEXT: &'static str = "Space Engineers Calculator is a handy app to calculate whether \
your grid (ship) has enough thrust, power generation, and hydrogen generation to keep up. It also \
calculates charging durations, maximum jump distances, and more.

This calculator does not simulate a grid down to the level of armor blocks, as that would be \
infeasible. Instead, only the most impactful blocks are simulated. The mass of blocks not covered \
in the calculator can be simulated by increasing 'Additional Mass'.

Grids are saved and loaded through the 'Grid' menu. Settings are modified through the \
'Window -> Settings' menu, where GUI settings such as font size are changed, and where several \
impactful mods can be enabled. The button in the top right corer switches between light and dark \
mode.

All numbers can be changed either by text editing, or by dragging the number. When dragging, hold \
Shift to make the number change slower.";

#[cfg(target_arch = "wasm32")]
const STORAGE_TEXT: &'static str = "The data in this calculator is stored whenever you press \
'Save' anywhere, in the Local Storage of your browser. If you clean your Local Storage (for this \
website), all data will be lost.";

#[cfg(not(target_arch = "wasm32"))]
const STORAGE_TEXT: &'static str = "The data in this calculator is stored whenever you press \
'Save' anywhere, in a user-directory appropriate for your operating system.";
