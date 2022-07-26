use eframe::Frame;
use egui::Context;

pub struct App {}

impl App {
  pub fn new(_ctx: &eframe::CreationContext<'_>) -> Self {
    Self {}
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    egui::Window::new("Space Engineers Calculator").show(ctx, |_ui| {});
  }
}
