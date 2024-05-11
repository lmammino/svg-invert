# svg-invert

[![Build Status](https://github.com/lmammino/svg-invert/actions/workflows/rust.yml/badge.svg)](https://github.com/lmammino/svg-invert/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/lmammino/svg-invert/graph/badge.svg?token=2a5OOr6Um4)](https://codecov.io/gh/lmammino/svg-invert)
[![Crates.io](https://img.shields.io/crates/v/svg-invert.svg)](https://crates.io/crates/svg-invert)

A CLI utility that inverts colors in an SVG file.

---

## 💁‍♂️ Use case

If you ever need to invert all the colors (fill and stroke) in an SVG file, this simple CLI utility is for you.

My original use case was to be able to invert black and white-ish diagrams so that they can be used in both light and dark mode in an e-book, but I reckon this utility can fulfill many other use cases. [Let me know](https://twitter.com/loige) what you'll use it for! 🚀


## 🛠️ Installation

You can install `svg-invert` using precompiled binaries (if available for your operative system and architecture) or by compiling it from source.

### Using precompiled binaries

You can download precompiled binaries from the [releases page](https://github.com/lmammino/svg-invert/releases) and place them in a directory that is in your `PATH`.

If you have [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) in your system, you can use it to install the latest version of `svg-invert`:

```bash
cargo binstall svg-invert
```

This method has the advantage of automatically placing the binary in a directory that is in your `PATH`. Also, if a binary is not available for your operative system and architecture, `cargo binstall` will compile it for you (assuming you have all the necessary Rust build toolchain in your system).

### Compiling from source

If you have the Rust toolchain installed in your system, you can compile `svg-invert` from source using `cargo`:

```bash
cargo install svg-invert
```

## 👩‍🏫 Usage

Right now, `svg-invert` offers only a very simple and minimal interface: data in from stdin and data out to stdout:

```bash
svg-invert < some-lovely.svg > inverted-some-lovely.svg
```

or, if you like Unix pipes:

```bash
cat some-lovely.svg | svg-invert > inverted-some-lovely.svg
```

For example, if this is the content of `some-lovely.svg`:

![A lovely crab with a Lambda hat](./examples/some-lovely.svg)

After running the command, this will be the content of `inverted-some-lovely.svg`:

![A lovely crab with a Lambda hat with inverted colours](./examples/inverted-some-lovely.svg)

> [!NOTE]
> If think this little crab is cute, you should check out my book [Crafting Lambda Functions in Rust](https://rust-lambda.com) where you can learn how to build serverless applications with Rust and AWS Lambda! 🦀🚀 Fun fact: This utility was actually born as a way to create SVGs diagrams that look good on both the light and dark version of the e-book!

## Usage as a library

You can use this utility also as part of you own Rust program by using it as a library.

You can install the library with:

```bash
cargo add svg-invert
```

Then you can use it in your Rust program like this:

```rust
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use svg_invert::invert_svg;

let reader = BufReader::new(stdin());
let writer = BufWriter::new(stdout());
match invert_svg(reader, writer) {
  Ok(_) => {
    // makes sure to flush stdout before exiting
    // since we are using a BufWriter
    match stdout().flush() {
      Ok(_) => {}
      Err(e) => {
        eprintln!("Error: {e}");
      }
    }
  }
  Err(e) => {
    eprintln!("Error: {e}");
  }
}
```

## Processing multiple images

[svg_invert::invert_svg](fn.invert_svg.html) is a shortcut for creating a new [InvertSvg](struct.InvertSvg.html) instance and calling its [invert_svg](struct.InvertSvg.html#method.invert_svg) method.

If you intend to process multiple SVG images in the same program, it might be beneficial to create a single [InvertSvg](struct.InvertSvg.html) instance and reuse it.

In fact, an [InvertSvg](struct.InvertSvg.html) instance holds an internal cache that stores the inverted colors of the SVG images it processes.
So, by reusing the same instance, you might have some performance gains if your images happen to share the same colors.

```rust
use svg_invert::InvertSvg;

let invert_svg = InvertSvg::new();
// call invert_svg.invert_svg(...) multiple times
```

## 👷 Contributing

Everyone is very welcome to contribute to this project.
You can contribute just by submitting bugs or suggesting improvements by
[opening an issue on GitHub](https://github.com/lmammino/svg-invert/issues).


## 👩‍⚖️ License

Licensed under [MIT License](LICENSE). © Luciano Mammino.