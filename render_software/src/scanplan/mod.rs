//!
//! # ScanPlan
//!
//! The 'scan-plan' is a low-level model of a rasterized scene. It represents each line of the final result as a 'plan'
//! of programs to apply to ranges of pixels. This plan can be executed to generate the final scene, or combined with
//! other plans to create more complex renders.
//!
//! There are a few advantages of making this plan: notably it avoids overdraw (where individual pixels are rendered
//! multiple times) and it makes it easy to efficiently mix colours using f32 precision. This can make rendering faster
//! for complex scenes as work can be avoided rendering pixels that will be obscured later on, and it makes it easy to
//! parallize both the rendering and the generation tasks. Less complex scenes may render more slowly due to the extra
//! work involved, however.
//!

mod scanspan;
mod scanline_plan;

pub use scanspan::*;
pub use scanline_plan::*;
