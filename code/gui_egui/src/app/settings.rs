use egui::{Context, DragValue, Window};

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_settings_windows(&mut self, ctx: &Context) {
    self.show_settings_window(ctx);

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

  fn show_settings_window(&mut self, ctx: &Context) {
    let mut show_settings_window = self.show_settings_window;
    Window::new("Settings")
      .open(&mut show_settings_window)
      .auto_sized()
      .show(ctx, |ui| {
        ui.open_header_with_grid("Mods", |ui| {
          for m in self.data.mods.iter() {
            let id = m.0;
            ui.hyperlink_to(&m.1, format!("https://steamcommunity.com/workshop/filedetails/?id={}", id));
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
        ui.open_header_with_grid("GUI", |ui| {
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
      });
    self.show_settings_window = show_settings_window;
  }
}