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

//! Dynamic color theme variant enumeration.

/// Themes for Dynamic Color.
///
/// Variants represent different styling algorithms for generating dynamic palettes
/// from a single seed color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// A purely greyscale theme with no chroma.
    Monochrome,
    /// A close-to-greyscale theme with subtle saturation.
    Neutral,
    /// The default Material theme with moderate vibrancy and tonal fidelity.
    TonalSpot,
    /// A theme with high chroma and punchy accent colors.
    Vibrant,
    /// A playful theme with rotated secondary and tertiary hues.
    Expressive,
    /// A theme prioritizing color accuracy matching the source input.
    Fidelity,
    /// A theme designed for media content where content colors dominate.
    Content,
    /// A rainbow theme with high-chroma primary and secondary hues.
    Rainbow,
    /// A dual-hue playful theme.
    FruitSalad,
}
