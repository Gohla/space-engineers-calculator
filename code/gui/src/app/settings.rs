use eframe::App as AppT;
use egui::{Align2, Context, DragValue, ScrollArea, Window};
use egui::output::OpenUrl;

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_settings_windows(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    // Return immediately if GUI is disabled. Using disabled windows is not possible as they can 
    // overlap on top of modal windows.
    if !self.enable_gui { return; }
    
    self.show_settings_window(ctx, frame);

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
    let mut show_settings_window = self.show_settings_window;
    let mut close_settings_window = false;
    Window::new("Settings")
      .open(&mut show_settings_window)
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
                if ui.link(&m.1).clicked() {
                  let url = format!("https://steamcommunity.com/workshop/filedetails/?id={}", id);
                  ctx.output().open_url = Some(OpenUrl { url, new_tab: true });
                }
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
            close_settings_window = true;
          }
        });
      });
    self.show_settings_window = show_settings_window && !close_settings_window;
  }
}