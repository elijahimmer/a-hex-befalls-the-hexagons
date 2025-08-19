run:
  cargo fmt
  cargo run -F debug,dev

build:
  cargo fmt
  cargo build -F debug,dev

op_monsters:
  cargo run --profile op_monsters --features op_monsters

wayland:
  cargo fmt
  cargo run --profile wayland -F debug,dev,bevy/wayland

release:
  cargo build --release

wasm:
  trunk serve --cargo-profile wasm --no-default-features --features debug

wasm-release:
  -rm game.zip
  trunk build --cargo-profile wasm-release --no-default-features
  zip game.zip dist -r

wasm-release-run: wasm-release
  trunk serve --cargo-profile wasm-release --no-default-features

clean:
  -rm game.zip result
  trunk clean
  cargo clean
