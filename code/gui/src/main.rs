#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::prelude::*;

use crate::app::App;

mod app;
mod widget;

fn main() {
  #[cfg(target_arch = "wasm32")] { // Setup panics to log to the console on WASM.
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
  }

  #[cfg(not(target_arch = "wasm32"))] { // Setup environment variables from .env on native.
    dotenv::dotenv().ok();
  }

  // Setup tracing.
  let layered = tracing_subscriber::registry();
  #[cfg(not(target_arch = "wasm32"))] {
    layered
      .with(
        tracing_subscriber::fmt::layer()
          .with_writer(std::io::stderr)
          .with_filter(tracing_subscriber::EnvFilter::from_env("LOG"))
      )
      .init();
  }
  #[cfg(target_arch = "wasm32")] {
    layered
      .with(tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default()))
      .init();
  }

  // Run application.
  #[cfg(not(target_arch = "wasm32"))] {
    let options = eframe::NativeOptions {
      min_window_size: Some(egui::Vec2::new(800.0, 600.0)),
      ..eframe::NativeOptions::default()
    };
    eframe::run_native(
      "Space Engineers Calculator",
      options,
      Box::new(|ctx| Box::new(App::new(ctx))),
    ).expect("failed to start eframe");
  }
  #[cfg(target_arch = "wasm32")] {
    use web_sys::HtmlLinkElement;
    use eframe::wasm_bindgen::JsCast;

    // Make HTML suitable for hosting an eframe application.
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    // Add style.css stylesheet.
    let head = document.head().expect("expecting a head on document");
    let link: HtmlLinkElement = document.create_element("link").unwrap().unchecked_into::<HtmlLinkElement>();
    link.set_rel("stylesheet");
    link.set_href("./style.css");
    head.append_child(&link).unwrap();
    // Add canvas to body.
    let body = document.body().expect("document expect to have have a body");
    let canvas = document.create_element("canvas").unwrap();
    let canvas_id = "canvas";
    canvas.set_id(canvas_id);
    body.append_child(&canvas).unwrap();

    // Start application in the canvas.
    let options = eframe::WebOptions {
      ..eframe::WebOptions::default()
    };
    wasm_bindgen_futures::spawn_local(async {
      eframe::WebRunner::new()
        .start(
          canvas_id,
          options,
          Box::new(|ctx| Box::new(App::new(ctx))),
        )
        .await
        .expect("failed to start eframe");
    });
  }
}
