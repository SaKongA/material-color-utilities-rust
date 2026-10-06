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

//! Contrast curve representation for contrast adaptation.

use crate::utils::lerp;

/// A class containing a value that changes with the contrast level.
///
/// Usually represents the contrast requirements for a dynamic color on its
/// background. The four values correspond to contrast levels -1.0, 0.0, 0.5,
/// and 1.0, respectively.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastCurve {
    /// Value for contrast level -1.0 (low contrast).
    pub low: f64,
    /// Value for contrast level 0.0 (normal / default contrast).
    pub normal: f64,
    /// Value for contrast level 0.5 (medium contrast).
    pub medium: f64,
    /// Value for contrast level 1.0 (high contrast).
    pub high: f64,
}

impl ContrastCurve {
    /// Creates a new `ContrastCurve` object.
    pub const fn new(low: f64, normal: f64, medium: f64, high: f64) -> Self {
        Self {
            low,
            normal,
            medium,
            high,
        }
    }

    /// Returns the value at a given contrast level.
    ///
    /// # Arguments
    /// * `contrast_level` - The contrast level: -1.0 is lowest, 0.0 is normal, 1.0 is highest.
    ///
    /// # Returns
    /// The value (for contrast ratios, a number between 1.0 and 21.0).
    pub fn get(&self, contrast_level: f64) -> f64 {
        if contrast_level <= -1.0 {
            self.low
        } else if contrast_level < 0.0 {
            lerp(self.low, self.normal, (contrast_level - (-1.0)) / 1.0)
        } else if contrast_level < 0.5 {
            lerp(self.normal, self.medium, (contrast_level - 0.0) / 0.5)
        } else if contrast_level < 1.0 {
            lerp(self.medium, self.high, (contrast_level - 0.5) / 0.5)
        } else {
            self.high
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_curve_bounds() {
        let curve = ContrastCurve::new(1.0, 3.0, 4.5, 7.0);
        assert_eq!(curve.get(-2.0), 1.0);
        assert_eq!(curve.get(-1.0), 1.0);
        assert_eq!(curve.get(0.0), 3.0);
        assert_eq!(curve.get(0.5), 4.5);
        assert_eq!(curve.get(1.0), 7.0);
        assert_eq!(curve.get(2.0), 7.0);
    }

    #[test]
    fn test_contrast_curve_interpolation() {
        let curve = ContrastCurve::new(1.0, 3.0, 4.5, 7.0);
        assert!((curve.get(-0.5) - 2.0).abs() < 1e-6);
        assert!((curve.get(0.25) - 3.75).abs() < 1e-6);
        assert!((curve.get(0.75) - 5.75).abs() < 1e-6);
    }
}
