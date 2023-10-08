use super::drawing_state::*;
use super::layer::*;
use super::pixel_programs::*;

use crate::pixel::*;
use crate::pixel_programs::*;

use flo_sparse_array::*;

use flo_canvas as canvas;

///
/// A `CanvasDrawing` represents the state of a drawing after a series of `Draw` commands have been processed
///
pub struct CanvasDrawing<TPixel, const N: usize>
where
    TPixel: 'static + Send + Sync + Pixel<N>,
{
    /// The gamma correction value for the current drawing
    pub (super) gamma:              f64,

    /// The height in pixels of the target (used for things like line_width_pixels)
    pub (super) height_pixels:      f64,

    /// The program data ID for the program used to render the background
    pub (super) background:         PixelProgramDataId,

    /// The namespace for the current set of IDs
    pub (super) current_namespace:  canvas::NamespaceId,

    /// The layer that we're currently writing to
    pub (super) current_layer:      LayerHandle,

    /// The current drawing state
    pub (super) current_state:      DrawingState,

    /// Maps layer handles to layers
    pub (super) layers:             SparseArray<Layer>,

    /// The layers in order
    pub (super) ordered_layers:     Vec<LayerHandle>,

    /// The next layer handle to allocate
    pub (super) next_layer_handle:  LayerHandle,

    /// Used to store the pixel programs used by this drawing
    pub (super) program_cache:      CanvasPixelPrograms<TPixel, N>,

    /// Used to store the data for the pixel program used by this drawing
    pub (super) program_data_cache: PixelProgramDataCache<TPixel>,

    /// States that have been pushed by PushState
    pub (super) state_stack:        Vec<DrawingState>,
}

impl<TPixel, const N: usize> CanvasDrawing<TPixel, N> 
where
    TPixel: 'static + Send + Sync + Pixel<N>,
{
    ///
    /// Creates a blank canvas drawing
    ///
    pub fn empty() -> Self {
        // Create an empty initial layer
        let mut layers = SparseArray::<Layer>::empty();
        let initial_layer = Layer::default();

        layers.insert(0, initial_layer);

        // Create the program and data cache
        let mut program_cache   = CanvasPixelPrograms::default();
        let mut data_cache      = program_cache.create_data_cache();

        // Default background colour is solid white
        let background          = program_cache.program_cache.store_program_data(&program_cache.solid_color, &mut data_cache, SolidColorData(TPixel::white()));

        CanvasDrawing {
            gamma:              2.2,
            height_pixels:      1080.0,
            background:         background,
            current_namespace:  canvas::NamespaceId::default(),
            current_layer:      LayerHandle(0),
            current_state:      DrawingState::default(),
            layers:             layers,
            ordered_layers:     vec![LayerHandle(0)],
            next_layer_handle:  LayerHandle(1),
            program_cache:      program_cache,
            program_data_cache: data_cache,
            state_stack:        vec![],
        }
    }

    ///
    /// Sets the height in pixels of the target for this drawing
    ///
    /// (This is used for pixel-precise operations like `LineWidthPixels()`)
    ///
    pub fn set_pixel_height(&mut self, pixel_height: f64) {
        self.height_pixels = pixel_height;
    }

    ///
    /// Updates the state of this drawing with some drawing instructions
    ///
    pub fn draw(&mut self, drawing: impl IntoIterator<Item=canvas::Draw>) {
        for instruction in drawing {
            use canvas::Draw::*;

            match instruction {
                StartFrame                                          => { /* For flow control outside of the renderer */ },
                ShowFrame                                           => { /* For flow control outside of the renderer */ },
                ResetFrame                                          => { /* For flow control outside of the renderer */ },

                Namespace(namespace)                                => { self.current_namespace = namespace; },

                ClearCanvas(color)                                  => { self.clear_canvas(TPixel::from_color(color, self.gamma)); },
                Layer(layer_id)                                     => { self.select_layer(layer_id); },
                LayerBlend(layer_id, blend_mode)                    => { self.layer_blend(layer_id, blend_mode); },
                LayerAlpha(layer_id, alpha)                         => { self.layer_alpha(layer_id, alpha as f64); },
                ClearLayer                                          => { self.clear_layer(self.current_layer); },
                ClearAllLayers                                      => { self.clear_all_layers(); },
                SwapLayers(layer_1, layer_2)                        => { self.swap_layers(layer_1, layer_2); },

                Path(path_op)                                       => { self.current_state.path_op(path_op); },
                Fill                                                => { self.fill(); },
                Stroke                                              => { self.stroke(); },

                LineWidth(width)                                    => { self.current_state.line_width(width as _); },
                LineWidthPixels(width_pixels)                       => { self.current_state.line_width_pixels(width_pixels as _, self.height_pixels as _); },
                LineJoin(join_style)                                => { self.current_state.line_join(join_style); },
                LineCap(cap_style)                                  => { self.current_state.line_cap(cap_style); },
                NewDashPattern                                      => { /* todo!() - dash patterns not supported yet */ },
                DashLength(_dash_length)                            => { /* todo!() - dash patterns not supported yet */ },
                DashOffset(_dash_offset)                            => { /* todo!() - dash patterns not supported yet */ },
                FillColor(fill_color)                               => { self.current_state.fill_solid_color(fill_color, &mut self.program_data_cache); },
                FillTexture(texture, (x1, y1), (x2, y2))            => { /* todo!() */ },
                FillGradient(gradient, (x1, y1), (x2, y2))          => { /* todo!() */ },
                FillTransform(transform)                            => { /* todo!() */ },
                StrokeColor(stroke_color)                           => { self.current_state.stroke_solid_color(stroke_color, &mut self.program_data_cache); },
                WindingRule(winding_rule)                           => { self.current_state.winding_rule(winding_rule); },
                BlendMode(blend_mode)                               => { /* todo!() */ },

                IdentityTransform                                   => { self.current_state.identity_transform(); },
                CanvasHeight(height)                                => { self.current_state.canvas_height(height); },
                CenterRegion((x1, y1), (x2, y2))                    => { self.current_state.center_region((x1, y1), (x2, y2)); },
                MultiplyTransform(transform)                        => { self.current_state.multiply_transform(transform); },

                Unclip                                              => { self.current_state.clip_path = DrawingClipRegion::None; },
                Clip                                                => { self.set_clipping_path(); },
                Store                                               => { self.store_layer_edges(); },
                Restore                                             => { self.restore_layer_edges(); },
                FreeStoredBuffer                                    => { self.free_stored_edges(); },
                PushState                                           => { self.push_state() },
                PopState                                            => { self.pop_state() },

                Sprite(sprite_id)                                   => { /* todo!() */ },
                MoveSpriteFrom(sprite_id)                           => { /* todo!() */ },
                ClearSprite                                         => { /* todo!() */ },
                SpriteTransform(transform)                          => { /* todo!() */ },
                DrawSprite(sprite_id)                               => { /* todo!() */ },
                DrawSpriteWithFilters(sprite_id, filters)           => { /* todo!() */ },

                Texture(texture_id, texture_op)                     => { /* todo!() */ },
                Gradient(gradient_id, gradient_op)                  => { /* todo!() */ },

                Font(_font_id, _font_op)                            => { /* Use the glyph and font streams in flo_canvas */ },
                BeginLineLayout(_x, _y, _alignment)                 => { /* Use the glyph and font streams in flo_canvas */ },
                DrawLaidOutText                                     => { /* Use the glyph and font streams in flo_canvas */ },
                DrawText(_font_id, _text, _x, _y)                   => { /* Use the glyph and font streams in flo_canvas */ },
            }
        }

        // TODO: really want to defer this until we get to the point where we are actually planning to render something
        // (It's more efficient to only call this immediately before a render, in case there are things on the canvas that are never ultimately rendered)
        self.prepare_to_render();
    }

    ///
    /// Prepares the layers in this drawing for rendering
    ///
    #[cfg(feature="multithreading")]
    fn prepare_to_render(&mut self) {
        use rayon::prelude::*;

        let mut layers = self.layers.iter_mut()
            .map(|(_, layer)| layer)
            .collect::<Vec<_>>();

        // Prepare each layer for rendering
        layers.par_iter_mut()
            .for_each(|layer| layer.edges.prepare_to_render());
    }

    ///
    /// Prepares the layers in this drawing for rendering
    ///
    #[cfg(not(feature="multithreading"))]
    fn prepare_to_render(&mut self) {
        // Prepare each layer for rendering
        self.layers.iter_mut()
            .for_each(|(_, layer)| layer.edges.prepare_to_render());
    }

    ///
    /// Returns the program runner for this canvas drawing
    ///
    pub fn program_runner(&self) -> &PixelProgramDataCache<TPixel> {
        &self.program_data_cache
    }

    ///
    /// Clears the canvas
    ///
    pub (super) fn clear_canvas(&mut self, new_background_color: TPixel) {
        // Clear the state stack
        while self.state_stack.len() > 0 {
            self.pop_state();
        }

        // Create an empty set of layers, containing only layer 0
        let mut layers = SparseArray::<Layer>::empty();
        let initial_layer = Layer::default();

        layers.insert(0, initial_layer);

        self.current_state.release_all_programs(&mut self.program_data_cache);

        // Reset the state of the canvas
        self.current_layer      = LayerHandle(0);
        self.layers             = layers;
        self.current_state      = DrawingState::default();
        self.ordered_layers     = vec![LayerHandle(0)];
        self.current_namespace  = canvas::NamespaceId::default();
        self.next_layer_handle  = LayerHandle(1);

        // Free the old program data
        self.program_data_cache.free_all_data();

        // Create a new background colour
        let background = self.program_cache.program_cache.store_program_data(&self.program_cache.solid_color, &mut self.program_data_cache, SolidColorData(new_background_color));
        self.background = background;
    }
}
