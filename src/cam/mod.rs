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

//! CAM16 color appearance model and HCT color space modules.

#[allow(clippy::module_inception)]
pub mod cam;
pub mod hct;
pub mod hct_solver;
pub mod viewing_conditions;

pub use cam::{
    cam_distance, cam_from_int, cam_from_int_and_viewing_conditions,
    cam_from_jch_and_viewing_conditions, cam_from_ucs_and_viewing_conditions,
    cam_from_xyz_and_viewing_conditions, int_from_cam, int_from_cam_and_viewing_conditions,
    int_from_hcl, Cam,
};
pub use hct::Hct;
pub use hct_solver::{solve_to_cam, solve_to_int};
pub use viewing_conditions::{
    create_viewing_conditions, default_with_background_lstar, ViewingConditions,
    DEFAULT_VIEWING_CONDITIONS,
};
