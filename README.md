# euclider
[![Build Status](https://travis-ci.org/Limeth/euclider.svg?branch=master)](https://travis-ci.org/Limeth/euclider)
[![Clippy Linting Result](https://clippy.bashy.io/github/Limeth/euclider/master/badge.svg)](https://clippy.bashy.io/github/Limeth/euclider/master/log)

A non-euclidean ray tracing prototype written in Rust.

# Installation
1. Install the Rust language via [Rustup](https://www.rustup.rs/)
2. Clone this repository and `cd` into it
3. Run `cargo run --release`

# Controls

* Mouse - camera rotation
* Mouse wheel - resolution adjustment
* [`W`/`A`/`S`/`D`/`Shift`/`Control`] - camera movement
* [`Esc`] - exit

# Preview of version 0.5.0

![Reflection](preview_2_reflection.png)
A sphere reflecting another sphere.

# Preview of version 0.4.0

![A perlin noise surface](preview_1_perlin.png)
An implementation of a perlin noise HSV surface on a sphere and a test shape.
