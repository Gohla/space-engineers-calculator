# Space Engineers Calculator

Calculator for designing Space Engineers ships. A live version of this calculator can be found at https://secalc.gohla.nl.

## Requirements

The calculator is written in Rust. Installation instructions for [Rust can be found here](https://www.rust-lang.org/tools/install).

### Web

To build and/or run the web version, you need to add the `wasm32-unknown-unknown` target to your rust installation with:

```
rustup target add wasm32-unknown-unknown
```

And you need to install the [wasm-bindgen CLI](https://rustwasm.github.io/wasm-bindgen/reference/cli.html) with:

```
cargo install -f wasm-bindgen-cli
```

Finally, you need to install [wasm-server-runner](https://github.com/jakobhellermann/wasm-server-runner) with:

```
cargo install wasm-server-runner
```

## Running

### Windows/Linux/macOS

Run the native calculator GUI with:

```
cargo run --bin secalc_gui
```

### Web

Run the web calculator GUI with:

```
cd code/gui/web
cargo run --bin secalc_gui --target wasm32-unknown-unknown --target-dir ../../../target-wasm
```

This will start a local web server, with the link to the webserver being shown in stdout.

To build the web calculator GUI for hosting on a different website, run:

```
cargo build --package secalc_gui --target wasm32-unknown-unknown --target-dir target-wasm --release
wasm-bindgen --out-dir code/gui/web/wasm_out --target web --no-typescript target-wasm/wasm32-unknown-unknown/release/secalc_gui.wasm
```

Or if you have [PowerShell](https://docs.microsoft.com/en-us/powershell/) installed, run:

```
./build_wasm.ps1
```

Then, you can copy the contents of `code/gui/web` to a web server to host it there.
