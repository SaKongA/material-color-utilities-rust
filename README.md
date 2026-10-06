# material-color-utilities-rust

A pure Rust implementation of Google's [Material Color Utilities](https://github.com/material-foundation/material-color-utilities) for Material Design 3 (M3).

[English](README.md) | [简体中文](README_ZH.md)

## Features

- **Zero Runtime Dependencies**: Pure Rust using only the standard library.
- **Reference Compatibility**: Faithfully ported from Google's C++ implementation with identical constants and numerical behavior.
- **Color Science & Models**: Full CAM16 and HCT (Hue, Chroma, Tone) solver.
- **Material 3 Dynamic Color**:
  - `DynamicScheme` with 9 preset variants (`TonalSpot`, `Vibrant`, `Expressive`, `Fidelity`, `Content`, `Monochrome`, `Neutral`, `Rainbow`, `FruitSalad`).
  - 54 system color roles in `MaterialDynamicColors`.
  - Contrast curves and tone delta constraints.
- **Quantization & Scoring**: Wu, WSMeans, and Celebi quantizers with M3 color ranking.
- **UI Friendly**: Convenient ARGB and RGBA conversions for UI frameworks like Slint.

## Quick Start

```rust
use material_color_utilities::{DynamicScheme, Hct, rgba_from_argb};

fn main() {
    // 1. Create a seed color
    let seed = Hct::from_int(0xff42_85f4); // Google Blue

    // 2. Generate Light and Dark Material 3 schemes
    let light = DynamicScheme::tonal_spot(seed, false, 0.0);
    let dark = DynamicScheme::tonal_spot(seed, true, 0.0);

    // 3. Access dynamic color roles
    println!("Light Primary: #{:06x}", light.primary() & 0x00ff_ffff);
    println!("Dark Primary:  #{:06x}", dark.primary() & 0x00ff_ffff);

    // 4. Convert to RGBA for UI rendering
    let (r, g, b, a) = rgba_from_argb(light.primary());
    println!("RGBA: ({r}, {g}, {b}, {a})");
}
```

## Preview

![Theme Preview](examples/example.png)

Preview color schemes directly in your terminal with 24-bit TrueColor:

```bash
cargo run --example theme_preview [HEX_COLOR]
```

## Testing

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

## License

Copyright 2026 SaKongA.

Licensed under the Apache License, Version 2.0.
