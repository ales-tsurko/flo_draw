use super::scanline_plan::*;
use crate::edgeplan::*;

use std::ops::{Range};

///
/// Describes how to transform the x positions in the edge plan to the viewport x positions
///
#[derive(Clone, Copy)]
pub struct ScanlineTransform {
    /// Value to add to the x coordinates before transforming
    offset: f64,

    /// The scale to apply to the 
    scale: f64,

    /// The reciprocal of the scale
    scale_recip: f64,
}

impl ScanlineTransform {
    ///
    /// Creates a scanline transform that maps from the specified source x range to pixel values of 0..pixel_width
    ///
    #[inline]
    pub fn for_region(source_x_range: Range<f64>, pixel_width: usize) -> Self {
        ScanlineTransform {
            offset:         -source_x_range.start,
            scale:          (pixel_width as f64) / (source_x_range.start-source_x_range.end),
            scale_recip:    (source_x_range.start-source_x_range.end) / (pixel_width as f64),
        }
    }
}

///
/// A scan planner is an algorithm that discovers where along a scanline to render pixels using pixel programs
///
pub trait ScanPlanner : Send + Sync {
    /// The type of edge stored in the edge plan for this planner
    type Edge: EdgeDescriptor;

    ///
    /// For every scanline in `y_positions`, use the edge plan to find the intercepts at a set of y-positions, clipped to the specified x-range, and
    /// generating the output in the `scanlines` array.
    ///
    /// The y-position is copied into the scanlines array, and the scanlines are always generated in the same order that they are requested in.
    ///
    fn plan_scanlines(&self, edge_plan: &EdgePlan<Self::Edge>, y_positions: &[f64], x_range: Range<f64>, scanlines: &mut [(f64, ScanlinePlan)]);
}
