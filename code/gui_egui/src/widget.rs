use egui::{CollapsingHeader, CollapsingResponse, Grid, InnerResponse, Ui};

pub trait UiExtensions {
  fn open_header_with_grid<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>>;
  fn open_header<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R>;

  fn grid<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;
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

  fn grid<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Grid::new(id_source).striped(true).min_col_width(1.0).show(self, add_contents)
  }
}
