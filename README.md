# TCSS 360 project
(name WIP)

## Playing
The game is compiled into web assembly, and uploaded
to itch.io, so it can be played in the browsed at:

https://elijahimmer.itch.io/a-hex-befalls


Alternatively, you can [compile and run the app natively](#Compilation)

## Compilation
### Linux

If you have `nix` (https://nixos.org/), you can
simplify do `nix build` and it should build properly.

Otherwise,
This project requires the libraries

- `alsa-lib`
- `vulkan`
- `vulkan-loader`
- `libX11`
- `libXcursor`
- `libXi`
- `libXrandr`
- `libxkbcommon`
- `udev`

And the following tools:
- `rustc`
- `cargo`
- `pkg-config`
- `cmake`

Once all of these are available (you should have many already installed), run:
```sh
cargo run --release
```

This will download all of the rust dependencies

### Mac

(This is Work In Progress)

You will need rust installed: https://www.rust-lang.org/tools/install

After that, you should be able to

```sh
cargo run --release
```

### Windows
(This is Work In Progress)

You will need rust installed: https://www.rust-lang.org/tools/install
It is non-trivial like on every other platform, so best of luck.

After that, you should be able to

```sh
cargo run --release
```

## Licensing
Everything in this project is licensed under the MIT license, except that which is
in the `assets/fonts` directory.

