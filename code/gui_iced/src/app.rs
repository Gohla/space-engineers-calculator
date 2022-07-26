use std::fmt::Debug;

use iced::{Application, Command, Element, executor};
use log::error;

use secalc_core::data::Data;
use secalc_core::grid::GridCalculator;

use crate::page::{grid_calc, load, load_confirm_discard, save_as, save_overwrite_confirm};
use crate::storage::Storage;

pub struct App {
  data: Data,
  storage: Storage,
  current_page: Page,
  grid_calc_page: grid_calc::Page,
}

#[derive(Debug)]
pub enum Page {
  GridCalc,
  SaveAs(save_as::Page),
  SaveAsOverwriteConfirm(save_overwrite_confirm::Page),
  LoadConfirmDiscard(load_confirm_discard::Page),
  Load(load::Page),
}

impl Page {
  fn save_as(name: Option<String>) -> Page {
    Page::SaveAs(save_as::Page::new(name))
  }

  fn save_as_overwrite_confirm(name: String) -> Page {
    Page::SaveAsOverwriteConfirm(save_overwrite_confirm::Page::new(name))
  }

  fn load_confirm_discard() -> Page {
    Page::LoadConfirmDiscard(load_confirm_discard::Page::default())
  }

  fn load(storage: &Storage) -> Page {
    Page::Load(load::Page::new(storage))
  }
}

#[derive(Clone, Debug)]
pub enum Message {
  GridCalcPage(grid_calc::Message),
  SaveAsPage(save_as::Message),
  SaveAsOverwriteConfirmPage(save_overwrite_confirm::Message),
  LoadConfirmDiscardPage(load_confirm_discard::Message),
  LoadPage(load::Message),
}

impl Default for App {
  fn default() -> Self {
    let data = {
      let bytes: &[u8] = include_bytes!("../../../data/data.json");
      Data::from_json(bytes).expect("Cannot read data")
    };
    let storage = Storage::load()
      .unwrap_or_default()
      .unwrap_or_default();
    let current_page = Page::GridCalc;
    let grid_calc_page = grid_calc::Page::new(&data, &GridCalculator::default(), &storage.calculator);
    Self {
      data,
      storage,
      current_page,
      grid_calc_page,
    }
  }
}

impl Application for App {
  type Executor = executor::Default;
  type Message = Message;
  type Flags = ();

  fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
    (Self::default(), Command::none())
  }

  fn title(&self) -> String {
    String::from("Space Engineers Calculator")
  }

  fn update(&mut self, message: Message) -> Command<Message> {
    match (&mut self.current_page, message) {
      (Page::GridCalc, Message::GridCalcPage(m)) => match self.grid_calc_page.update(m, &mut self.storage.calculator, &self.data) {
        Some(grid_calc::Action::CalculatorModified) => self.storage.calculator_modified = true,
        Some(grid_calc::Action::Save) => if let Some(name) = self.storage.calculator_name.clone() {
          self.storage.save_calculator(name)
            .unwrap_or_else(|e| error!("[BUG] Could not save storage: {}", e));
          self.current_page = Page::GridCalc;
        } else {
          self.current_page = Page::save_as(self.storage.calculator_name.clone());
        },
        Some(grid_calc::Action::SaveAs) => self.current_page = Page::save_as(self.storage.calculator_name.clone()),
        Some(grid_calc::Action::Load) => if self.storage.calculator_modified {
          self.current_page = Page::load_confirm_discard();
        } else {
          self.current_page = Page::load(&self.storage);
        },
        None => {},
      },
      (Page::SaveAs(page), Message::SaveAsPage(ref m)) => match page.update(m.clone()) {
        Some(save_as::Action::Save(name)) => if self.storage.contains_saved_calculator(&name) {
          self.current_page = Page::save_as_overwrite_confirm(name.clone());
        } else {
          self.storage.save_calculator(name.clone())
            .unwrap_or_else(|e| error!("[BUG] Could not save storage: {}", e));
          self.current_page = Page::GridCalc;
        },
        Some(save_as::Action::Cancel) => self.current_page = Page::GridCalc,
        None => {},
      },
      (Page::SaveAsOverwriteConfirm(page), Message::SaveAsOverwriteConfirmPage(ref m)) => match page.update(m.clone()) {
        save_overwrite_confirm::Action::Cancel => self.current_page = Page::GridCalc,
        save_overwrite_confirm::Action::Overwrite(name) => {
          self.storage.save_calculator(name.clone())
            .unwrap_or_else(|e| error!("[BUG] Could not save storage: {}", e));
          self.current_page = Page::GridCalc;
        },
      },
      (Page::LoadConfirmDiscard(page), Message::LoadConfirmDiscardPage(ref m)) => match page.update(m.clone()) {
        load_confirm_discard::Action::Cancel => self.current_page = Page::GridCalc,
        load_confirm_discard::Action::Discard => self.current_page = Page::load(&self.storage),
      },
      (Page::Load(page), Message::LoadPage(ref m)) => match page.update(m.clone()) {
        Some(load::Action::Load(name)) => {
          self.storage.load_calculator(name)
            .unwrap_or_else(|e| error!("[BUG] Could not load calculator: {}", e));
          self.grid_calc_page.reload_input(&self.storage.calculator, &self.data);
          self.current_page = Page::GridCalc;
        },
        Some(load::Action::Cancel) => self.current_page = Page::GridCalc,
        None => {},
      },
      (page, m) => error!("[BUG] Requested update with message '{:?}', but that message cannot be handled by the current page '{:?}' or the application itself", m, page),
    }
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    match &mut self.current_page {
      Page::GridCalc => self.grid_calc_page.view().map(Message::GridCalcPage),
      Page::SaveAs(page) => page.view().map(Message::SaveAsPage),
      Page::SaveAsOverwriteConfirm(page) => page.view().map(Message::SaveAsOverwriteConfirmPage),
      Page::LoadConfirmDiscard(page) => page.view().map(Message::LoadConfirmDiscardPage),
      Page::Load(page) => page.view().map(Message::LoadPage),
    }
  }
}
