use egui::{Button, CollapsingHeader, CollapsingResponse, Color32, Grid, Id, InnerResponse, Response, Stroke, Ui, WidgetText};
use egui::collapsing_header::CollapsingState;
use egui::output::OpenUrl;

pub trait UiExtensions {
  fn open_collapsing_header_with_grid<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>>;
  fn open_collapsing_header<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R>;

  fn open_collapsing_state<HR, BR>(
    &mut self,
    id_source: impl std::hash::Hash,
    add_header: impl FnOnce(&mut Ui) -> HR,
    add_body: impl FnOnce(&mut Ui) -> BR
  ) -> (Response, InnerResponse<HR>, Option<InnerResponse<BR>>);

  fn grid<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;

  fn danger_button(&mut self, text: impl Into<WidgetText>) -> Response;

  fn url(&mut self, url: impl Into<String>) -> Response;

  fn url_link(&mut self, label: impl Into<WidgetText>, url: impl Into<String>) -> Response;
}

impl UiExtensions for Ui {
  fn open_collapsing_header_with_grid<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>> {
    CollapsingHeader::new(header).default_open(true).show(self, |ui| {
      Grid::new(format!("{} Grid", header)).striped(true).min_col_width(1.0).show(ui, add_contents)
    })
  }

  fn open_collapsing_header<R>(&mut self, header: &str, add_body: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R> {
    CollapsingHeader::new(header).default_open(true).show(self, add_body)
  }


  fn open_collapsing_state<HR, BR>(
    &mut self,
    id_source: impl std::hash::Hash,
    add_header: impl FnOnce(&mut Ui) -> HR,
    add_body: impl FnOnce(&mut Ui) -> BR
  ) -> (Response, InnerResponse<HR>, Option<InnerResponse<BR>>) {
    CollapsingState::load_with_default_open(self.ctx(), Id::new(id_source), true)
      .show_header(self, add_header)
      .body(add_body)
  }


  fn grid<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Grid::new(id_source).striped(true).min_col_width(1.0).show(self, add_contents)
  }

  fn danger_button(&mut self, text: impl Into<WidgetText>) -> Response {
    self.add(Button::new(text).stroke(Stroke::new(0.5, Color32::RED)))
  }

  fn url(&mut self, url: impl Into<String>) -> Response {
    let url = url.into();
    let response = self.link(&url);
    if response.clicked() {
      self.ctx().output().open_url = Some(OpenUrl { url, new_tab: true });
    }
    response
  }

  fn url_link(&mut self, label: impl Into<WidgetText>, url: impl Into<String>) -> Response {
    let response = self.link(label);
    if response.clicked() {
      self.ctx().output().open_url = Some(OpenUrl { url: url.into(), new_tab: true });
    }
    response
  }
}
