use super::canvas_drawing::*;

use crate::edges::*;
use crate::edgeplan::*;
use crate::pixel::*;
use crate::pixel_programs::*;

use flo_canvas as canvas;
use flo_canvas::curves::line::*;
use flo_canvas::curves::bezier::*;

use smallvec::*;

///
/// A brush represents what will be used to fill in the next region 
///
#[derive(Clone)]
pub enum Brush {
    /// Basic solid colour brush (will be drawn opaque so the image behind will be hidden)
    OpaqueSolidColor(canvas::Color),

    /// Transparent solid colour brush (will be blended with the image behind)
    TransparentSolidColor(canvas::Color),
}

///
/// Represents the active drawing state for a canvas drawing
///
#[derive(Clone)]
pub struct DrawingState {
    /// The shape descriptor that will be used for filling the next shape (or None if we haven't allocated data for it yet)
    pub (super) fill_program: Option<ShapeDescriptor>,

    /// The shape descriptor that will be used for filling the stroke of the next shape (or None if we haven't allocated data for it yet)
    pub (super) stroke_program: Option<ShapeDescriptor>,

    /// The brush to select next time fill_program is None
    pub (super) next_fill_brush: Brush,

    /// The brush to select next time stroke_program is None
    pub (super) next_stroke_brush: Brush,

    /// The current position along the path
    pub (super) path_position: Coord2,

    /// The edges of the current path in this drawing state
    pub (super) path_edges: Vec<Curve<Coord2>>,

    /// Indexes of the points where the subpaths starts
    pub (super) subpaths: Vec<usize>,
}

impl Default for DrawingState {
    fn default() -> Self {
        DrawingState { 
            fill_program:       None,
            stroke_program:     None,
            next_fill_brush:    Brush::OpaqueSolidColor(canvas::Color::Rgba(0.0, 0.0, 0.0, 1.0)),
            next_stroke_brush:  Brush::OpaqueSolidColor(canvas::Color::Rgba(0.0, 0.0, 0.0, 1.0)),
            path_position:      Coord2::origin(),
            path_edges:         vec![],
            subpaths:           vec![0],
        }
    }
}

impl DrawingState {
    ///
    /// Ensures that a program location is released (sets it to None)
    ///
    /// The state holds on to the programs it's going to use, so they have to be released before they can be changed
    ///
    #[inline]
    pub fn release_program<TPixel, const N: usize>(program: &mut Option<ShapeDescriptor>, data_cache: &mut PixelProgramDataCache<TPixel>) 
    where
        TPixel: Send + Pixel<N>,
    {
        if let Some(mut program) = program.take() {
            for program_data in program.programs.drain(..) {
                data_cache.release_program_data(program_data);
            }
        }
    }

    ///
    /// Releases any pixel program data that is being retained by this state
    ///
    pub fn release_all_programs<TPixel, const N: usize>(&mut self, data_cache: &mut PixelProgramDataCache<TPixel>) 
    where
        TPixel: Send + Pixel<N>,
    {
        Self::release_program(&mut self.fill_program, data_cache);
        Self::release_program(&mut self.stroke_program, data_cache);
    }

    ///
    /// Updates the state so that the next shape added will use a solid fill colour 
    ///
    pub fn fill_solid_color<TPixel, const N: usize>(&mut self, colour: canvas::Color, data_cache: &mut PixelProgramDataCache<TPixel>) 
    where
        TPixel: Send + Pixel<N>,
    {
        // This clears the fill program so we allocate data for it next time
        Self::release_program(&mut self.fill_program, data_cache);

        // Choose opaque or transparent for the brush based on the alpha component
        if colour.alpha_component() >= 1.0 {
            self.next_fill_brush = Brush::OpaqueSolidColor(colour);
        } else {
            self.next_fill_brush = Brush::TransparentSolidColor(colour);
        }
    }

    ///
    /// Updates the state so that the next shape added will use a solid fill colour 
    ///
    pub fn stroke_solid_color<TPixel, const N: usize>(&mut self, colour: canvas::Color, data_cache: &mut PixelProgramDataCache<TPixel>)
    where
        TPixel: Send + Pixel<N>,
    {
        // This clears the stroke program so we allocate data for it next time
        Self::release_program(&mut self.stroke_program, data_cache);

        // Choose opaque or transparent for the brush based on the alpha component
        if colour.alpha_component() >= 1.0 {
            self.next_stroke_brush = Brush::OpaqueSolidColor(colour);
        } else {
            self.next_stroke_brush = Brush::TransparentSolidColor(colour);
        }
    }

    ///
    /// Dispatches a path operation
    ///
    #[inline]
    pub fn path_op(&mut self, path_op: canvas::PathOp) {
        use canvas::PathOp::*;

        match path_op {
            NewPath                                             => self.path_new(),
            Move(x, y)                                          => self.path_move(x as _, y as _),
            Line(x, y)                                          => self.path_line(x as _, y as _),
            BezierCurve(((cp1x, cp1y), (cp2x, cp2y)), (dx, dy)) => self.path_bezier_curve((cp1x as _, cp1y as _), (cp2x as _, cp2y as _), (dx as _, dy as _)),
            ClosePath                                           => self.path_close(),
        }
    }

    ///
    /// Start a new path
    ///
    pub fn path_new(&mut self) {
        self.path_edges.clear();
        self.subpaths.clear();
        self.subpaths.push(0);
    }

    ///
    /// Moves to start a new subpath
    ///
    pub fn path_move(&mut self, x: f64, y: f64) {
        // Start a new subpath if we've generated any new edges
        if self.subpaths.pop() != Some(self.path_edges.len()) {
            self.subpaths.push(self.path_edges.len());
        }

        // Set the 'last position'
        self.path_position = Coord2(x, y);
    }

    ///
    /// Draws a line to a position
    ///
    pub fn path_line(&mut self, x: f64, y: f64) {
        // Create a line from the last position
        let next_pos    = Coord2(x, y);
        let line        = (self.path_position, next_pos);

        // Store as a bezier curve
        self.path_edges.push(line_to_bezier(&line));

        // Update the position
        self.path_position = next_pos;
    }

    ///
    /// Draws a bezier curve to a position
    ///
    pub fn path_bezier_curve(&mut self, cp1: (f64, f64), cp2: (f64, f64), end: (f64, f64)) {
        // Convert the points
        let cp1 = Coord2(cp1.0, cp1.1);
        let cp2 = Coord2(cp2.0, cp2.1);
        let end = Coord2(end.0, end.1);

        // Create a curve
        let curve = Curve::from_points(self.path_position, (cp1, cp2), end);
        self.path_edges.push(curve);

        // Update the last position
        self.path_position = end;
    }

    ///
    /// Closes the current path
    ///
    pub fn path_close(&mut self) {
        // If the path has 0 edges, we can't close it
        if let Some(subpath_idx) = self.subpaths.last().copied() {
            // Are building a subpath (should always be true)
            if subpath_idx < self.path_edges.len() {
                // Subpath has some path components in it
                let start_point = self.path_edges[subpath_idx].start_point();

                // Want to close by drawing a line from the end of last_curve to the subpath start
                if start_point != self.path_position {
                    self.path_line(start_point.x(), start_point.y());
                }
            }
        }
    }
}

impl<TPixel, const N: usize> CanvasDrawing<TPixel, N>
where
    TPixel: 'static + Send + Sync + Pixel<N>,
{
    ///
    /// Creates a shape descriptor from a brush
    ///
    /// The z-index of this descriptor will be set to 0: this should be updated later on
    ///
    pub fn create_shape_descriptor(&mut self, brush: &Brush) -> ShapeDescriptor
    where
        TPixel: 'static + Send + Sync + Pixel<N>,
    {
        use Brush::*;

        let gamma           = self.gamma;
        let program_cache   = &self.program_cache;
        let data_cache      = &mut self.program_data_cache;

        let descriptor = match brush {
            OpaqueSolidColor(color) => {
                let brush_data = program_cache.program_cache.store_program_data(&program_cache.solid_color, data_cache, SolidColorData(TPixel::from_color(*color, gamma)));

                ShapeDescriptor {
                    programs:   smallvec![brush_data],
                    is_opaque:  true,
                    z_index:    0
                }
            }

            TransparentSolidColor(color) => {
                let brush_data = program_cache.program_cache.store_program_data(&program_cache.source_over_color, data_cache, SolidColorData(TPixel::from_color(*color, gamma)));

                ShapeDescriptor {
                    programs:   smallvec![brush_data],
                    is_opaque:  false,
                    z_index:    0
                }
            }
        };

        descriptor
    }


    ///
    /// Adds the current path as a filled path to the current layer
    ///
    pub fn fill(&mut self) {
        // Fetch or create the fill shape descriptor
        let mut shape_descriptor = if let Some(shape_descriptor) = &mut self.current_state.fill_program {
            shape_descriptor.clone()
        } else {
            let shape_descriptor = self.create_shape_descriptor(&self.current_state.next_fill_brush.clone());
            self.current_state.fill_program = Some(shape_descriptor.clone());

            shape_descriptor
        };

        // Retrieve the current layer
        let layers          = &mut self.layers;
        let current_state   = &mut self.current_state;

        let current_layer = layers.get_mut(self.current_layer.0).unwrap();

        // Retain the programs in the shape descriptor and add them to the layer
        for data_id in shape_descriptor.programs.iter().copied() {
            self.program_data_cache.retain_program_data(data_id);
            current_layer.used_data.push(data_id);
        }

        // Set the z-index for the shape descriptor
        let z_index                 = current_layer.z_index;
        shape_descriptor.z_index    = z_index;
        current_layer.z_index += 1;

        // Write the edges using this program
        let shape_id = ShapeId::new();
        current_layer.edges.declare_shape_description(shape_id, shape_descriptor);

        // TODO: add as even-odd or non-zero depending on the current winding rule
        for edge in current_state.path_edges.iter() {
            current_layer.edges.add_edge(Box::new(EvenOddBezierCurveEdge::new(shape_id, edge.clone())));
        }

        // Generate lines for unclosed subpaths
        for subpath_idx in 0..current_state.subpaths.len() {
            // The subpath start and end index (inclusive)
            let start_idx   = current_state.subpaths[subpath_idx];
            let end_idx     = if subpath_idx+1 < current_state.subpaths.len() { current_state.subpaths[subpath_idx+1] } else { current_state.path_edges.len() };

            // Ignore zero-length paths
            if end_idx <= start_idx { continue; }
            let end_idx = end_idx - 1;

            // Get the start and end point of the subpath
            let start_point = current_state.path_edges[start_idx].start_point();
            let end_point   = current_state.path_edges[end_idx].end_point();

            // Add a line edge if they don't match
            // TODO: respect the winding rule
            if start_point != end_point {
                current_layer.edges.add_edge(Box::new(EvenOddBezierCurveEdge::<Curve<Coord2>>::new(shape_id, line_to_bezier(&(end_point, start_point)))));
            }
        }
    }
}
