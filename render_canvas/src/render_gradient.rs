use flo_canvas as canvas;
use flo_render as render;

///
/// Ued to indicate the state of a gradient: these are loaded as 1-dimensional textures when they are used
///
#[derive(Clone)]
pub enum RenderGradient {
    Defined(Vec<canvas::GradientOp>),
    Ready(render::TextureId, Vec<canvas::GradientOp>),
}
