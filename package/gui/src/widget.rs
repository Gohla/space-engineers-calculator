use egui::{Button, CollapsingHeader, CollapsingResponse, Color32, Grid, Id, InnerResponse, Response, Sense, Stroke, Ui, vec2, Widget, WidgetText};
use egui::collapsing_header::CollapsingState;
use egui::output::OpenUrl;

pub trait UiExtensions {
  fn open_collapsing_header_with_grid<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>>;
  fn open_collapsing_header<R>(&mut self, header: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R>;

  fn open_collapsing_state<HR, BR>(
    &mut self,
    id_source: impl std::hash::Hash,
    add_header: impl FnOnce(&mut Ui) -> HR,
    add_body: impl FnOnce(&mut Ui) -> BR,
  ) -> (Response, InnerResponse<HR>, Option<InnerResponse<BR>>);

  fn grid<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;
  fn grid_unstriped<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;

  fn danger_button(&mut self, text: impl Into<WidgetText>) -> Response;

  fn url(&mut self, url: impl Into<String>) -> Response;
  fn url_link(&mut self, label: impl Into<WidgetText>, url: impl Into<String>) -> Response;

  fn horizontal_separator_unpadded(&mut self);
  fn vertical_separator_unpadded(&mut self);
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
    add_body: impl FnOnce(&mut Ui) -> BR,
  ) -> (Response, InnerResponse<HR>, Option<InnerResponse<BR>>) {
    CollapsingState::load_with_default_open(self.ctx(), Id::new(id_source), true)
      .show_header(self, add_header)
      .body(add_body)
  }


  fn grid<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Grid::new(id_source).striped(true).min_col_width(0.0).min_row_height(0.0).show(self, add_contents)
  }

  fn grid_unstriped<R>(&mut self, id_source: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Grid::new(id_source).striped(false).min_col_width(0.0).min_row_height(0.0).show(self, add_contents)
  }

  fn danger_button(&mut self, text: impl Into<WidgetText>) -> Response {
    self.add(Button::new(text).stroke(Stroke::new(0.5, Color32::RED)))
  }

  fn url(&mut self, url: impl Into<String>) -> Response {
    let url = url.into();
    let response = self.link(&url);
    if response.clicked() {
      self.ctx().output_mut(|o| o.open_url.replace(OpenUrl { url, new_tab: true }));
    }
    response
  }

  fn url_link(&mut self, label: impl Into<WidgetText>, url: impl Into<String>) -> Response {
    let response = self.link(label);
    if response.clicked() {
      self.ctx().output_mut(|o| o.open_url.replace(OpenUrl { url: url.into(), new_tab: true }));
    }
    response
  }

  fn horizontal_separator_unpadded(&mut self) {
    self.add(HorizontalSeparator);
  }

  fn vertical_separator_unpadded(&mut self) {
    self.add(VerticalSeparator);
  }
}

pub struct HorizontalSeparator;

impl Widget for HorizontalSeparator {
  fn ui(self, ui: &mut Ui) -> Response {
    let available_width = ui.available_width();
    let size = vec2(available_width, 0.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(response.rect) {
      let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
      let (min, max) = rect.x_range().into_inner();
      let spacing_div_two = ui.style().spacing.item_spacing.x / 2.0;
      let x = min - spacing_div_two..=max + spacing_div_two; // Draw half spacing left and right, ignoring spacing.
      ui.painter().hline(x, rect.center().y, stroke);
    }
    response
  }
}

pub struct VerticalSeparator;

impl Widget for VerticalSeparator {
  fn ui(self, ui: &mut Ui) -> Response {
    let available_height = ui.available_height();
    let size = vec2(0.0, available_height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(response.rect) {
      let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
      let (min, max) = rect.y_range().into_inner();
      let spacing_div_two = ui.style().spacing.item_spacing.y / 2.0;
      let y = min - spacing_div_two..=max + spacing_div_two; // Draw half spacing above and below, ignoring spacing.
      ui.painter().vline(rect.center().x, y, stroke);
    }
    response
  }
}
