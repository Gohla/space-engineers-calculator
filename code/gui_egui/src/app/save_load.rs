use eframe::App as AppT;
use egui::{Align2, Context, Layout, RichText, TextEdit, Window};
use egui_extras::{Size, TableBuilder};

use crate::App;
use crate::widget::UiExtensions;

impl App {
  pub fn show_load_window(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    if self.show_load_window {
      Window::new("Load")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .fixed_size([380.0, 600.0])
        .show(ctx, |ui| {
          let mut load_clicked = None;
          let mut delete_clicked = None;
          TableBuilder::new(ui)
            .striped(true)
            .cell_layout(Layout::left_to_right())
            .scroll(true)
            .column(Size::remainder().at_least(255.0))
            .column(Size::remainder().at_least(115.0))
            .body(|mut body| {
              for (name, calculator) in &self.saved_calculators {
                body.row(26.0, |mut row| {
                  row.col(|ui| {
                    let text = if Some(name) == self.current_calculator.as_ref() {
                      RichText::new(name).strong()
                    } else {
                      RichText::new(name)
                    };
                    ui.label(text);
                  });
                  row.col(|ui| {
                    if ui.button("Load").clicked() {
                      load_clicked = Some((name.clone(), calculator.clone()));
                    }
                    if ui.danger_button("Delete").clicked() {
                      delete_clicked = Some(name.clone());
                    }
                  });
                });
              }
            });
          if let Some((name, calculator)) = load_clicked {
            self.calculator = calculator;
            self.calculate();
            self.current_calculator = Some(name);
            self.current_calculator_saved = true;
            if let Some(storage) = frame.storage_mut() {
              self.save(storage);
            }

            self.enable_gui = true;
            self.show_load_window = false;
          }
          if let Some(name) = delete_clicked {
            self.show_load_window = false;
            self.show_delete_confirm_window = Some(name);
          }
          ui.separator();
          ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
              self.enable_gui = true;
              self.show_load_window = false;
            }
          });
        });
    }
  }

  pub fn show_load_confirm_window(&mut self, ctx: &Context) {
    if self.show_load_confirm_window {
      Window::new("Confirm Load")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .fixed_size([500.0, 250.0])
        .show(ctx, |ui| {
          ui.label("Are you sure you want to load a grid? The current grid has not been saved. Any unsaved data will be lost.");
          ui.separator();
          ui.horizontal(|ui| {
            if ui.danger_button("Continue").clicked() {
              self.show_load_confirm_window = false;
              self.show_load_window = true;
            }
            if ui.button("Cancel").clicked() {
              self.enable_gui = true;
              self.show_load_confirm_window = false;
            }
          });
        });
    }
  }

  pub fn show_delete_confirm_window(&mut self, ctx: &Context) {
    if self.show_delete_confirm_window.is_some() {
      Window::new("Confirm Delete")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .fixed_size([500.0, 250.0])
        .show(ctx, |ui| {
          if let Some(name) = &self.show_delete_confirm_window {
            ui.label(format!("Are you sure you want to delete grid '{}'? Any deleted data will be lost.", name));
          }
          ui.separator();
          ui.horizontal(|ui| {
            if ui.danger_button("Delete").clicked() {
              let name = self.show_delete_confirm_window.take().unwrap();
              self.saved_calculators.remove(&name);
              if Some(name) == self.current_calculator {
                self.current_calculator = None;
                self.current_calculator_saved = false;
              }

              self.show_delete_confirm_window = None;
              self.show_load_window = true;
            }
            if ui.button("Cancel").clicked() {
              self.show_delete_confirm_window = None;
              self.show_load_window = true;
            }
          });
        });
    }
  }

  pub fn show_save_as_window(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    if self.show_save_as_window.is_some() {
      Window::new("Save As")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .fixed_size([300.0, 250.0])
        .show(ctx, |ui| {
          ui.horizontal(|ui| {
            ui.label("Name");
            if let Some(name) = &mut self.show_save_as_window {
              TextEdit::singleline(name).desired_width(300.0).show(ui);
            }
            ui.end_row();
          });
          ui.separator();
          ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
              let name = self.show_save_as_window.take().unwrap();
              if self.saved_calculators.contains_key(&name) {
                self.show_save_as_window = None;
                self.show_save_as_confirm_window = Some(name)
              } else {
                self.saved_calculators.insert(name.clone(), self.calculator.clone());
                self.current_calculator = Some(name);
                self.current_calculator_saved = true;
                if let Some(storage) = frame.storage_mut() {
                  self.save(storage);
                }

                self.enable_gui = true;
                self.show_save_as_window = None;
              }
            }
            if ui.button("Cancel").clicked() {
              self.enable_gui = true;
              self.show_save_as_window = None;
            }
          });
        });
    }
  }

  pub fn show_save_as_confirm_window(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    if self.show_save_as_confirm_window.is_some() {
      Window::new("Confirm Save")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .fixed_size([500.0, 250.0])
        .show(ctx, |ui| {
          if let Some(name) = &self.show_save_as_confirm_window {
            ui.label(format!("A saved grid named '{}' already exists. Are you sure you want to overwrite '{}' with the current grid? Any overwritten data will be lost.", name, name));
          }
          ui.separator();
          ui.horizontal(|ui| {
            if ui.danger_button("Overwrite").clicked() {
              let name = self.show_save_as_confirm_window.take().unwrap();
              self.saved_calculators.insert(name.clone(), self.calculator.clone());
              self.current_calculator = Some(name);
              self.current_calculator_saved = true;
              if let Some(storage) = frame.storage_mut() {
                self.save(storage);
              }

              self.enable_gui = true;
              self.show_save_as_confirm_window = None;
            }
            if ui.button("Cancel").clicked() {
              self.enable_gui = true;
              self.show_save_as_confirm_window = None;
            }
          });
        });
    }
  }

  pub fn show_reset_confirm_window(&mut self, ctx: &Context) {
    if self.show_reset_confirm_window {
      Window::new("Confirm Reset")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .fixed_size([500.0, 250.0])
        .show(ctx, |ui| {
          ui.label("Are you sure you want to reset all grid data (left-side panel) to their defaults? Any unsaved data will be lost.");
          ui.separator();
          ui.horizontal(|ui| {
            if ui.danger_button("Reset").clicked() {
              self.enable_gui = true;
              self.show_reset_confirm_window = false;
              self.calculator = self.calculator_default.clone();
              self.calculate();
              self.current_calculator = None;
              self.current_calculator_saved = true; // True because the calculator is reset and not worth saving.
            }
            if ui.button("Cancel").clicked() {
              self.enable_gui = true;
              self.show_reset_confirm_window = false;
            }
          });
        });
    }
  }
}