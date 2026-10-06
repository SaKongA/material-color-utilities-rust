/*
 * Copyright 2026 SaKongA
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use material_color_utilities::{rgba_from_argb, Argb, DynamicScheme, Hct, TonalPalette};
use std::env;

/// Formats an ARGB color as a 24-bit TrueColor ANSI colored block.
fn color_swatch(argb: Argb, label: &str) -> String {
    let (r, g, b, _) = rgba_from_argb(argb);
    // Choose high contrast foreground for the text inside swatch
    let luma = 0.299 * (r as f64) + 0.587 * (g as f64) + 0.114 * (b as f64);
    let fg_code = if luma > 140.0 { "30" } else { "37" };
    format!(
        "\x1b[48;2;{r};{g};{b}m\x1b[{fg_code}m {label:^24} \x1b[0m #{:06x}",
        argb & 0x00ff_ffff
    )
}

/// Prints a row of tones for a given TonalPalette.
fn print_tonal_row(name: &str, palette: &TonalPalette) {
    print!("{name:<16}: ");
    let tones = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 99, 100];
    for &tone in &tones {
        let argb = palette.get(tone as f64);
        let (r, g, b, _) = rgba_from_argb(argb);
        print!("\x1b[48;2;{r};{g};{b}m  \x1b[0m");
    }
    println!();
}

fn print_scheme_roles(title: &str, scheme: &DynamicScheme) {
    println!("\n=== {title} ===");
    let roles = [
        ("Primary", scheme.primary(), "OnPrimary", scheme.on_primary()),
        ("PrimaryContainer", scheme.primary_container(), "OnPrimaryContainer", scheme.on_primary_container()),
        ("Secondary", scheme.secondary(), "OnSecondary", scheme.on_secondary()),
        ("SecondaryContainer", scheme.secondary_container(), "OnSecondaryContainer", scheme.on_secondary_container()),
        ("Tertiary", scheme.tertiary(), "OnTertiary", scheme.on_tertiary()),
        ("TertiaryContainer", scheme.tertiary_container(), "OnTertiaryContainer", scheme.on_tertiary_container()),
        ("Surface", scheme.surface(), "OnSurface", scheme.on_surface()),
        ("SurfaceVariant", scheme.surface_variant(), "OnSurfaceVariant", scheme.on_surface_variant()),
        ("Background", scheme.background(), "OnBackground", scheme.on_background()),
        ("Error", scheme.error(), "OnError", scheme.on_error()),
    ];

    for (bg_name, bg_val, fg_name, fg_val) in roles {
        let swatch_bg = color_swatch(bg_val, bg_name);
        let swatch_fg = color_swatch(fg_val, fg_name);
        println!("{swatch_bg}  |  {swatch_fg}");
    }
}

fn main() {
    // Default seed: Material 3 Deep Purple (#6750A4)
    let default_hex = "6750A4";
    let seed_hex = env::args().nth(1).unwrap_or_else(|| default_hex.to_string());
    let clean_hex = seed_hex.trim_start_matches('#');
    let seed_argb: Argb = match u32::from_str_radix(clean_hex, 16) {
        Ok(val) => 0xff00_0000 | val,
        Err(_) => {
            eprintln!("Invalid hex color '{seed_hex}', using default #{default_hex}");
            0xff67_50a4
        }
    };

    let seed_hct = Hct::from_int(seed_argb);
    println!("Material Color Utilities - Theme Preview");
    println!("--------------------------------------------------");
    println!(
        "Seed Color : {} (Hue: {:.1}°, Chroma: {:.1}, Tone: {:.1})",
        color_swatch(seed_argb, "Seed Color"),
        seed_hct.hue(),
        seed_hct.chroma(),
        seed_hct.tone()
    );

    // Generate dynamic schemes
    let light_scheme = DynamicScheme::tonal_spot(seed_hct, false, 0.0);
    let dark_scheme = DynamicScheme::tonal_spot(seed_hct, true, 0.0);

    // Display Tonal Palettes
    println!("\n=== Core Tonal Palettes (Tones 0 -> 100) ===");
    print_tonal_row("Primary", &light_scheme.primary_palette);
    print_tonal_row("Secondary", &light_scheme.secondary_palette);
    print_tonal_row("Tertiary", &light_scheme.tertiary_palette);
    print_tonal_row("Neutral", &light_scheme.neutral_palette);
    print_tonal_row("NeutralVariant", &light_scheme.neutral_variant_palette);

    // Display Light and Dark roles
    print_scheme_roles("Light Theme Roles", &light_scheme);
    print_scheme_roles("Dark Theme Roles", &dark_scheme);

    println!("\nTip: Run `cargo run --example theme_preview <HEX_COLOR>` to test different seed colors!");
}
