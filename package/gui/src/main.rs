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
    dotenvy::dotenv().ok();
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
      viewport: egui::ViewportBuilder {
        min_inner_size: Some(egui::Vec2::new(800.0, 600.0)),
        ..Default::default()
      },
      ..Default::default()
    };
    eframe::run_native(
      "Space Engineers Calculator",
      options,
      Box::new(|ctx| Box::new(App::new(ctx))),
    ).expect("failed to start eframe");
  }
  #[cfg(target_arch = "wasm32")] {
    // Start application in the canvas.
    let options = eframe::WebOptions {
      ..eframe::WebOptions::default()
    };
    wasm_bindgen_futures::spawn_local(async {
      eframe::WebRunner::new()
        .start(
          "canvas",
          options,
          Box::new(|ctx| Box::new(App::new(ctx))),
        )
        .await
        .expect("failed to start eframe");
    });
  }
}
