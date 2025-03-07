# Building Hyperspeedcube

**Hyperspeedcube requires the latest version of the Rust compiler.** If you are getting errors building Hyperspeedcube, check that `cargo --version` matches the latest stable version on [whatrustisit.com](https://www.whatrustisit.com/).

If you tell me you have trouble building HSC and it turns out you _don't_ have the latest version of the Rust compiler, I will bop you on the head with a 24-cell. You have been warned!

## Building on Linux or macOS

1. Download/install Cargo.
2. On Linux, install build-time dependencies: `sudo apt install cmake libglib2.0-dev libatk1.0-dev libgtk-3-dev libxkbcommon-x11-dev`
3. Clone this project and build/run:

```sh
git clone https://github.com/HactarCE/Hyperspeedcube
cd Hyperspeedcube
cargo run --release
```

The first build may take ~10 minutes or more. Remove `--release` to disable optimizations, which makes building faster but Hyperspeedcube may run slower.

## Building on Windows

1. Download/install [Rustup](https://www.rust-lang.org/tools/install).
2. Download this project and extract it somewhere.
3. Open a terminal in the folder where you extracted Hyperspeedcube (it should have `Cargo.toml` in it) and build it using `cargo build --release` or run it using `cargo run --release`.

The first build may take ~10 minutes or more. Remove `--release` to disable optimizations, which makes building faster but Hyperspeedcube may run slower.

## Building for web

1. Follow instructions above to run the native version first.
2. Install wasm32 target with `rustup target add wasm32-unknown-unknown`
3. Install Trunk with `cargo install --locked trunk`
4. Run `trunk serve` to build and serve on <http://127.0.0.1:8080>. Trunk will rebuild automatically if you edit the project. Open <http://127.0.0.1:8080/index.html#dev> in a browser.

If you get an error on `trunk serve` about failing to downloat wasm-bindgen, try installing wasm-bindgen-cli with `cargo install --locked wasm-bindgen-cli --version 0.2.83`. In case I haven't updated this guide, check `Cargo.toml` (or `hyperspeedcube/Cargo.toml`) for the version in use.

Note that `assets/sw.js` script will try to cache the app, and loads the cached version when it cannot connect to server allowing the app to work offline (like PWA). Appending `#dev` to `index.html` will skip this caching, allowing to load the latest builds during development.
