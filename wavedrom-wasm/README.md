# WaveDrom-Wasm

A editor website for wavedrom-rs. 

## Build

The build uses the [standalone Tailwind CSS
binary](https://github.com/tailwindlabs/tailwindcss/releases/latest), [Minify
HTML](https://github.com/wilsonzlin/minify-html) and [Minify
JS](https://github.com/wilsonzlin/minify-js).

```bash
cd wavedrom-wasm
cargo build --release
cp ../target/wasm32-unknown-unknown/release/wavedrom_wasm.wasm wavedrom.wasm

# For Development
tailwindcss -i index.scss -o index.css

# For Production
tailwindcss -i index.scss -o index.css --minify
minify-html index.html -o index.html
minify-js index.js -o index.js
```

The resulting `index.html` should be opening through some webserver to be able
to load WebAssembly. For example, with the `python -m http.server` command.