# Space Engineers Calculator

Calculator for designing Space Engineers ships. A live version of this calculator can be found at https://secalc.gohla.nl.

## Requirements

The calculator is written in Rust. Installation instructions for [Rust can be found here](https://www.rust-lang.org/tools/install).

### Web

To build and/or run the web version, you need to add the `wasm32-unknown-unknown` target to your rust installation with:

```
rustup target add wasm32-unknown-unknown
```

And you need to [install trunk](https://trunkrs.dev/#install).

## Running

### Windows/Linux/macOS

Run the native calculator GUI with:

```
cargo run --bin secalc_gui
```

### Web

Run the web calculator GUI with:

```
trunk serve
```

This will start a local web server, with the link to the webserver being shown in stdout.
