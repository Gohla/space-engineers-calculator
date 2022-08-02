use egui::{Context, DragValue, Ui};

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_settings(&mut self, ui: &mut Ui, ctx: &Context) {
    ui.open_header_with_grid("GUI", |ui| {
      ui.label("Font size modifier");
      if ui.add(DragValue::new(&mut self.font_size_modifier).clamp_range(-2..=10)).changed() {
        self.apply_style(ctx);
      }
      ui.end_row();
      ui.label("Increase contrast");
      if ui.checkbox(&mut self.increase_contrast, "").changed() {
        self.apply_style(ctx);
      }
      ui.end_row();
    });
  }
}