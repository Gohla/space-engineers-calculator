use std::fmt::Debug;

use iced::{Application, Command, Element};

use secalc_core::data::Data;
use secalc_core::grid::GridCalculator;

use crate::page::{load, load_confirm_discard, main, save_as, save_overwrite_confirm};
use crate::storage::CalculatorStorage;

pub struct App {
  current_page: Page,
  main_page: main::Page,
  calculator: GridCalculator,
  calculator_name: Option<String>,
  calculator_modified: bool,
  calculator_storage: CalculatorStorage,
  data: Data,
}

#[derive(Debug)]
pub enum Page {
  Main,
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

  fn load() -> Page {
    Page::Load(load::Page::default())
  }
}

#[derive(Clone, Debug)]
pub enum Message {
  MainPage(main::Message),
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
    let calculator = GridCalculator::default();
    let current_page = Page::Main;
    let main_page = main::Page::new(&data, &calculator);
    let calculator_name = None;
    let calculator_modified = false;
    let calculator_storage = CalculatorStorage::default();
    Self {
      current_page,
      main_page,
      calculator,
      calculator_name,
      calculator_modified,
      calculator_storage,
      data,
    }
  }
}

impl Application for App {
  type Message = Message;

  fn new() -> (Self, Command<Message>) {
    (Self::default(), Command::none())
  }

  fn title(&self) -> String {
    String::from("Space Engineers Calculator")
  }

  fn update(&mut self, message: Message) -> Command<Message> {
    match (&mut self.current_page, message) {
      (Page::Main, Message::MainPage(m)) => match self.main_page.update(m, &mut self.calculator, &self.data) {
        Some(main::Action::CalculatorModified) => self.calculator_modified = true,
        Some(main::Action::Save) => if let Some(name) = self.calculator_name.clone() {
          self.save(name);
          self.current_page = Page::Main;
        } else {
          self.current_page = Page::save_as(self.calculator_name.clone());
        },
        Some(main::Action::SaveAs) => self.current_page = Page::save_as(self.calculator_name.clone()),
        Some(main::Action::Load) => if self.calculator_modified {
          self.current_page = Page::load_confirm_discard();
        } else {
          self.current_page = Page::load();
        },
        None => {},
      },
      (Page::SaveAs(page), Message::SaveAsPage(ref m)) => match page.update(m.clone()) {
        Some(save_as::Action::Save(name)) => if self.calculator_storage.contains(&name) {
          self.current_page = Page::save_as_overwrite_confirm(name.clone());
        } else {
          self.save(name.clone());
          self.current_page = Page::Main;
        },
        Some(save_as::Action::Cancel) => self.current_page = Page::Main,
        None => {},
      },
      (Page::SaveAsOverwriteConfirm(page), Message::SaveAsOverwriteConfirmPage(ref m)) => match page.update(m.clone()) {
        save_overwrite_confirm::Action::Cancel => self.current_page = Page::Main,
        save_overwrite_confirm::Action::Overwrite(name) => {
          self.save(name.clone());
          self.current_page = Page::Main;
        },
      },
      (Page::LoadConfirmDiscard(page), Message::LoadConfirmDiscardPage(ref m)) => match page.update(m.clone()) {
        load_confirm_discard::Action::Cancel => self.current_page = Page::Main,
        load_confirm_discard::Action::Discard => self.current_page = Page::load(),
      },
      (Page::Load(page), Message::LoadPage(ref m)) => match page.update(m.clone()) {
        Some(load::Action::Load(name)) => {
          if let Some(calculator) = self.calculator_storage.get(&name) {
            self.calculator = calculator.clone();
            self.calculator_name = Some(name);
            self.calculator_modified = false;
            self.main_page.reload_input(calculator, &self.data);
          } else {
            panic!("[BUG] Load page requested load of stored calculator with name '{}', but no stored calculator with that name exists", name);
          }
          self.current_page = Page::Main;
        },
        Some(load::Action::Cancel) => self.current_page = Page::Main,
        None => {},
      },
      (page, m) => panic!("[BUG] Requested update with message '{:?}', but that message cannot be handled by the current page '{:?}' or the application itself", m, page),
    }
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    match &mut self.current_page {
      Page::Main => self.main_page.view().map(Message::MainPage),
      Page::SaveAs(page) => page.view().map(Message::SaveAsPage),
      Page::SaveAsOverwriteConfirm(page) => page.view().map(Message::SaveAsOverwriteConfirmPage),
      Page::LoadConfirmDiscard(page) => page.view().map(Message::LoadConfirmDiscardPage),
      Page::Load(page) => page.view().map(Message::LoadPage),
    }
  }
}

impl App {
  fn save(&mut self, name: String) {
    self.calculator_name = Some(name.clone());
    self.calculator_modified = false;
    self.calculator_storage.set(name, self.calculator.clone());
  }
}