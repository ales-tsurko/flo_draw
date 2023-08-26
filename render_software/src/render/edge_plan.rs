use super::renderer::*;
use super::render_source_trait::*;
use super::edgeplan_region_renderer::*;
use super::frame_size::*;
use super::scanline_renderer::*;
use super::u8_frame_renderer::*;

use crate::edgeplan::*;
use crate::pixel::*;
use crate::scanplan::*;

impl<TEdge> EdgePlan<TEdge>
where
    TEdge: EdgeDescriptor,
{
    ///
    /// Renders an edge plan to an 8-bit RGBA buffer (must contain width*height pixels)
    ///
    pub fn render_whole_frame<TPixel>(&self, data: &PixelProgramDataCache<TPixel>, width: usize, height: usize, gamma: f64, target: &mut [U8RgbaPremultipliedPixel])
    where
        TPixel: 'static + Default + Send + Copy + AlphaBlend + ToGammaColorSpace<U8RgbaPremultipliedPixel>,
    {
        // TODO:
        //      * Add a way to choose the scan planner to use
        //      * Add a trait to make the frame renderer from a target type and a source region renderer
        //      * Add a trait for creating the region renderer from a type (eg, EdgePlan in this case) and a scan planner
        //      * Some way to do away with the `for<'a> &'a ...` constraints on the region planners

        let scanline_renderer       = ScanlineRenderer::new(data);
        let scanline_planner        = PixelScanPlanner::<TEdge>::default();
        let edge_region_renderer    = EdgePlanRegionRenderer::new(scanline_planner, scanline_renderer);
        let frame_renderer          = U8FrameRenderer::new(edge_region_renderer);

        (&frame_renderer).render(&GammaFrameSize { width, height, gamma }, self, target);
    }
}

impl<'a, TEdge, TScanPlanner, TProgramRunner, TPixel> RenderSource<'a, TScanPlanner, TProgramRunner, TPixel> for EdgePlan<TEdge>
where
    TEdge:          'a + EdgeDescriptor,
    TPixel:         'static + Send + Copy + AlphaBlend,
    TScanPlanner:   'a + ScanPlanner<Edge=TEdge>,
    TProgramRunner: 'a + PixelProgramRunner<TPixel>,
{
    /// The region renderer takes instances of this type and uses them to generate pixel values in a region
    // TODO: using a reference here (required due to some later borrowing requirements) doesn't work
    type RegionRenderer = EdgePlanRegionRenderer<TEdge, TScanPlanner, ScanlineRenderer<'a, TPixel, TProgramRunner>>;

    ///
    /// Builds a region renderer that can read from this type and output pixels along rows
    ///
    fn create_region_renderer(planner: TScanPlanner, pixel_runner: &'a TProgramRunner) -> Self::RegionRenderer {
        let scanline_renderer   = ScanlineRenderer::new(pixel_runner);
        let region_renderer     = EdgePlanRegionRenderer::<TEdge, TScanPlanner, ScanlineRenderer<'a, TPixel, TProgramRunner>>::new(planner, scanline_renderer);

        region_renderer
    }
}
