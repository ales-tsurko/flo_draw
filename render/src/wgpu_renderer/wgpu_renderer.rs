use super::wgpu_shader::*;
use super::render_target::*;
use super::renderer_state::*;
use super::pipeline_configuration::*;

use crate::action::*;
use crate::buffer::*;

use wgpu;

use std::ops::{Range};
use std::sync::*;
use std::collections::{HashMap};

///
/// Renderer that uses the `wgpu` abstract library as a render target
///
pub struct WgpuRenderer {
    /// A reference to the device that this will render to
    device: Arc<wgpu::Device>,

    /// The command queue for the device
    queue: Arc<wgpu::Queue>,

    /// The surface that this renderer will target
    target_surface: Arc<wgpu::Surface>,

    /// The shaders that have been loaded for this renderer
    shaders: HashMap<WgpuShader, Arc<wgpu::ShaderModule>>,

    /// The vertex buffers for this renderer
    vertex_buffers: Vec<Option<wgpu::Buffer>>,

    /// The index buffers for this renderer
    index_buffers: Vec<Option<wgpu::Buffer>>,

    /// The textures for this renderer
    textures: Vec<Option<Arc<wgpu::Texture>>>,

    /// The render targets for this renderer
    render_targets: Vec<Option<RenderTarget>>,

    /// The cache of render pipeline states used by this renderer
    pipeline_states: HashMap<PipelineConfiguration, wgpu::RenderPipeline>,
}

impl WgpuRenderer {
    ///
    /// Creates a new WGPU renderer
    ///
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, target_surface: Arc<wgpu::Surface>) -> WgpuRenderer {
        WgpuRenderer {
            device:             device,
            queue:              queue,
            target_surface:     target_surface,
            shaders:            HashMap::new(),
            vertex_buffers:     vec![],
            index_buffers:      vec![],
            textures:           vec![],
            render_targets:     vec![],
            pipeline_states:    HashMap::new(),
        }
    }

    ///
    /// Performs some rendering instructions and returns the resulting command buffer
    ///
    pub fn render_to_buffer<Actions: IntoIterator<Item=RenderAction>>(&mut self, actions: Actions) {
        // Create the render state
        let mut render_state    = RendererState {
        };

        // Evaluate the actions
        for action in actions {
            use self::RenderAction::*;

            match action {
                SetTransform(matrix)                                                            => { self.set_transform(matrix, &mut render_state); }
                CreateVertex2DBuffer(id, vertices)                                              => { self.create_vertex_buffer_2d(id, vertices); }
                CreateIndexBuffer(id, indices)                                                  => { self.create_index_buffer(id, indices); }
                FreeVertexBuffer(id)                                                            => { self.free_vertex_buffer(id); }
                FreeIndexBuffer(id)                                                             => { self.free_index_buffer(id); }
                BlendMode(blend_mode)                                                           => { self.blend_mode(blend_mode, &mut render_state); }
                CreateRenderTarget(render_id, texture_id, Size2D(width, height), render_type)   => { self.create_render_target(render_id, texture_id, width, height, render_type); }
                FreeRenderTarget(render_id)                                                     => { self.free_render_target(render_id); }
                SelectRenderTarget(render_id)                                                   => { self.select_render_target(render_id, &mut render_state); }
                RenderToFrameBuffer                                                             => { self.select_main_frame_buffer(&mut render_state); }
                DrawFrameBuffer(render_id, region, Alpha(alpha))                                => { self.draw_frame_buffer(render_id, region, alpha, &mut render_state); }
                ShowFrameBuffer                                                                 => { /* This doesn't double-buffer so nothing to do */ }
                CreateTextureBgra(texture_id, Size2D(width, height))                            => { self.create_bgra_texture(texture_id, width, height); }
                CreateTextureMono(texture_id, Size2D(width, height))                            => { self.create_mono_texture(texture_id, width, height); }
                Create1DTextureBgra(texture_id, Size1D(width))                                  => { self.create_bgra_1d_texture(texture_id, width); }
                Create1DTextureMono(texture_id, Size1D(width))                                  => { self.create_mono_1d_texture(texture_id, width); }
                WriteTextureData(texture_id, Position2D(x1, y1), Position2D(x2, y2), data)      => { self.write_texture_data_2d(texture_id, x1, y1, x2, y2, data); }
                WriteTexture1D(texture_id, Position1D(x1), Position1D(x2), data)                => { self.write_texture_data_1d(texture_id, x1, x2, data); }
                CreateMipMaps(texture_id)                                                       => { self.create_mipmaps(texture_id, &mut render_state); }
                CopyTexture(src_texture, tgt_texture)                                           => { self.copy_texture(src_texture, tgt_texture, &mut render_state); }
                FilterTexture(texture, filter)                                                  => { self.filter_texture(texture, filter, &mut render_state); }
                FreeTexture(texture_id)                                                         => { self.free_texture(texture_id); }
                Clear(color)                                                                    => { self.clear(color, &mut render_state); }
                UseShader(shader_type)                                                          => { self.use_shader(shader_type, &mut render_state); }
                DrawTriangles(buffer_id, buffer_range)                                          => { self.draw_triangles(buffer_id, buffer_range, &mut render_state); }
                DrawIndexedTriangles(vertex_buffer, index_buffer, num_vertices)                 => { self.draw_indexed_triangles(vertex_buffer, index_buffer, num_vertices, &mut render_state); }
            }
        }
    }
    
    fn set_transform(&mut self, matrix: Matrix, render_state: &mut RendererState) {

    }
    
    fn create_vertex_buffer_2d(&mut self, VertexBufferId(vertex_id): VertexBufferId, vertices: Vec<Vertex2D>) {

    }
    
    fn create_index_buffer(&mut self, IndexBufferId(index_id): IndexBufferId, indices: Vec<u16>) {

    }
    
    fn free_vertex_buffer(&mut self, VertexBufferId(vertex_id): VertexBufferId) {

    }
    
    fn free_index_buffer(&mut self, IndexBufferId(index_id): IndexBufferId) {

    }
    
    fn blend_mode(&mut self, blend_mode: BlendMode, state: &mut RendererState) {

    }
    
    fn create_render_target(&mut self, RenderTargetId(render_id): RenderTargetId, TextureId(texture_id): TextureId, width: usize, height: usize, render_target_type: RenderTargetType) {

    }
    
    fn free_render_target(&mut self, RenderTargetId(render_id): RenderTargetId) {

    }
    
    fn select_render_target(&mut self, RenderTargetId(render_id): RenderTargetId, state: &mut RendererState) {

    }
    
    fn select_main_frame_buffer(&mut self, state: &mut RendererState) {

    }
    
    fn draw_frame_buffer(&mut self, RenderTargetId(source_buffer): RenderTargetId, region: FrameBufferRegion, alpha: f64, state: &mut RendererState) {

    }
    
    fn create_bgra_texture(&mut self, TextureId(texture_id): TextureId, width: usize, height: usize) {

    }
    
    fn create_mono_texture(&mut self, TextureId(texture_id): TextureId, width: usize, height: usize) {

    }
    
    fn create_bgra_1d_texture(&mut self, TextureId(texture_id): TextureId, width: usize) {

    }
    
    fn create_mono_1d_texture(&mut self, TextureId(texture_id): TextureId, width: usize) {

    }
    
    fn write_texture_data_2d(&mut self, TextureId(texture_id): TextureId, x1: usize, y1: usize, x2: usize, y2: usize, data: Arc<Vec<u8>>) {

    }
    
    fn write_texture_data_1d(&mut self, TextureId(texture_id): TextureId, x1: usize, x2: usize, data: Arc<Vec<u8>>) {

    }
    
    fn create_mipmaps(&mut self, TextureId(texture_id): TextureId, state: &mut RendererState) {

    }
    
    fn copy_texture(&mut self, TextureId(src_texture_id): TextureId, TextureId(tgt_texture_id): TextureId, state: &mut RendererState) {

    }
    
    fn filter_texture(&mut self, TextureId(texture_id): TextureId, filter: Vec<TextureFilter>, state: &mut RendererState) {

    }
    
    fn free_texture(&mut self, TextureId(texture_id): TextureId) {

    }
    
    fn clear(&mut self, color: Rgba8, state: &mut RendererState) {

    }
    
    fn use_shader(&mut self, shader_type: ShaderType, state: &mut RendererState) {

    }
    
    fn draw_triangles(&mut self, VertexBufferId(vertex_buffer_id): VertexBufferId, range: Range<usize>, state: &mut RendererState) {

    }
    
    fn draw_indexed_triangles(&mut self, VertexBufferId(vertex_buffer_id): VertexBufferId, IndexBufferId(index_buffer_id): IndexBufferId, num_vertices: usize, state: &mut RendererState) {

    }
}
