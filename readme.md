# FiveM WASM runtime
[Wasmtime](https://wasmtime.dev) runtime that adds ability to create and use WASM files on [FiveM](https://fivem.net) severs and clients in additional to js and lua.

This is a main repository implementing the runtime and containing bindings for Rust.

[The fork](https://github.com/zottce/fivem) contains only [C++ component](https://github.com/ZOTTCE/fivem/tree/wasm/code/components/citizen-scripting-wasm) that links and calls a static library built in Rust.

## Modules
* [`examples/basic-client`](examples/basic-client/) and [`examples/basic-server`](examples/basic-server/) - an example shows how to use bindings to access FiveM.
* [`bindings`](bindings/) - Rust bindings to WASM runtime to create mods.
* [`natives-gen`](natives-gen/) - a generator for natives.

## Building
* Install [the Rust compiler](https://rust-lang.org) and WASM toolchain (wasm32-wasi)
* Install `cargo-wasi` to build example or your scripts.
* Clone the FiveM fork with all submodules (including this repo).
* Build `vendor/fivem-wasm` with flag `--package cfx-component-glue`
* Use [this guide to build FiveM](https://github.com/citizenfx/fivem/blob/master/docs/building.md).
* Enjoy WASM in your FiveM server!

## TODOs
* Wait till there will be ability to use `std::net::TcpStream` and othe net utils to build a good server.
