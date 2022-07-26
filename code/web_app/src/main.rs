use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use secalc_core::data::Data;
use secalc_core::grid::GridCalculator;

fn main() {
  yew::start_app::<App>();
}

struct App {}

impl Component for App {
  type Message = ();
  type Properties = ();

  fn create(_ctx: &Context<Self>) -> Self {
    Self {}
  }
  fn view(&self, _ctx: &Context<Self>) -> Html {
    html! {
      <Calculator></Calculator>
    }
  }
}

#[derive(Default)]
struct Calculator {
  data: Data,
  grid_calculator: GridCalculator,
}

impl Component for Calculator {
  type Message = Option<String>;
  type Properties = ();

  fn create(_ctx: &Context<Self>) -> Self {
    let data = {
      let bytes: &[u8] = include_bytes!("../../../data/data.json");
      Data::from_json(bytes).expect("Cannot read data")
    };
    Self {
      data,
      grid_calculator: GridCalculator::default(),
    }
  }
  fn view(&self, ctx: &Context<Self>) -> Html {
    let handle_input = ctx.link().callback(move |input_event: InputEvent| {
      input_event.dyn_into::<HtmlInputElement>().unwrap().value()
    });
    html! {
      <>
        <h1>{ "Options" }</h1>
        <p>{"Gravity Multiplier"}</p>
        <input oninput={handle_input}/>
      </>
    }
  }
}
