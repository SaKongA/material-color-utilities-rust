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

//! A Rust implementation of Google's Material Color Utilities for Material 3
//! dynamic color and color science.

pub mod blend;
pub mod cam;
pub mod contrast;
pub mod dislike;
pub mod palettes;
pub mod quantize;
pub mod score;
pub mod temperature;
pub mod utils;

pub use blend::{blend_cam16_ucs, blend_harmonize, blend_hct_hue};
pub use cam::{Cam, Hct, ViewingConditions};
pub use contrast::{darker, darker_unsafe, lighter, lighter_unsafe, ratio_of_tones, ratio_of_ys};
pub use dislike::{fix_if_disliked, is_disliked};
pub use palettes::{CorePalettes, KeyColor, TonalPalette};
pub use quantize::{quantize_celebi, quantize_wsmeans, quantize_wu, Lab, QuantizerResult};
pub use score::{ranked_suggestions, ScoreOptions};
pub use temperature::TemperatureCache;
pub use utils::{Argb, Vec3};
