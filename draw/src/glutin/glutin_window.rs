use crate::events::*;
use crate::window_properties::*;

use flo_stream::*;
use flo_render::*;
use flo_binding::*;

use glutin::context::{NotCurrentContext, NotCurrentGlContextSurfaceAccessor};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::{GlConfig};
use glutin::surface::{GlSurface};
use glutin_winit::{self, GlWindow};
use winit::dpi::{LogicalSize};
use winit::window::{Window, Fullscreen};
use futures::prelude::*;
use futures::task::{Poll, Context};
use gl;

use std::pin::*;

///
/// Manages the state of a Glutin window
///
pub struct GlutinWindow<TConfig> 
where
    TConfig: GlConfig + GetGlDisplay,
{
    /// The context for this window
    context: Option<NotCurrentContext>,

    /// The configuration from when the context was create
    gl_config: TConfig,

    /// The surface for the window
    surface: Option<<TConfig::Target as GlDisplay>::WindowSurface>,

    /// The window the context is attached to
    window: Option<Window>,

    /// The renderer for this window (or none if there isn't one yet)
    renderer: Option<GlRenderer>
}

impl<TConfig> GlutinWindow<TConfig> 
where
    TConfig: GlConfig + GetGlDisplay,
{
    ///
    /// Creates a new glutin window
    ///
    pub fn new(context: NotCurrentContext, gl_config: TConfig, window: Window) -> GlutinWindow<TConfig> {
        GlutinWindow {
            context:    Some(context),
            gl_config:  gl_config,
            surface:    None,
            window:     Some(window),
            renderer:   None
        }
    }
}

///
/// Sends render actions to a window
///
pub (super) async fn send_actions_to_window<RenderStream: Unpin+Stream<Item=Vec<RenderAction>>, EventPublisher: MessagePublisher<Message=DrawEvent>, TConfig>(window: GlutinWindow<TConfig>, render_actions: RenderStream, events: EventPublisher, window_properties: WindowProperties) 
where
    TConfig: GlConfig + GetGlDisplay,
{
    // Read events from the render actions list
    let mut window          = window;
    let mut events          = events;
    let mut window_actions  = WindowUpdateStream { 
        render_stream:      render_actions, 
        title_stream:       follow(window_properties.title),
        size:               follow(window_properties.size),
        fullscreen:         follow(window_properties.fullscreen),
        has_decorations:    follow(window_properties.has_decorations),
        mouse_pointer:      follow(window_properties.mouse_pointer)
    };

    while let Some(next_action) = window_actions.next().await {
        match next_action {
            WindowUpdate::Render(next_action)   => {
                // Do nothing if there are no actions
                if next_action.len() == 0 {
                    continue;
                }

                let show_frame_buffer = if next_action[next_action.len() - 1] == RenderAction::ShowFrameBuffer {
                    // Typically this is the last instruction
                    true
                } else {
                    // Search harder if it's not the last instruction
                    next_action.iter().any(|item| item == &RenderAction::ShowFrameBuffer)
                };

                unsafe {
                    // TODO: report errors if we can't set the context rather than just stopping mysteriously

                    // Make the current context current
                    let current_context = window.context.take().expect("Window context");
                    let current_surface = window.surface.take().unwrap_or_else(|| {
                        let surface_attributes = window.window.as_ref().unwrap().build_surface_attributes(<_>::default());
                        window.gl_config.display().create_window_surface(&window.gl_config, &surface_attributes).expect("Create new window surface")
                    });

                    let current_context = current_context.make_current(&current_surface);
                    let current_context = if let Ok(context) = current_context { context } else { break; };

                    // Get informtion about the current context
                    let size            = current_context.window().inner_size();
                    let width           = size.width as usize;
                    let height          = size.height as usize;

                    // Create the renderer (needs the OpenGL functions to be loaded)
                    if window.renderer.is_none() {
                        // Load the functions for the current context
                        // TODO: we're assuming they stay loaded to avoid loading them for every render, which might not be safe
                        // TODO: probably better to have the renderer load the functions itself (gl::load doesn't work well
                        // when we load GL twice, which could happen if we want to use the offscreen renderer)
                        gl::load_with(|symbol_name| {
                            current_context.get_proc_address(symbol_name)
                        });

                        // Create the renderer
                        window.renderer = Some(GlRenderer::new());
                    }

                    // Perform the rendering actions
                    if let Some(renderer) = &mut window.renderer {
                        renderer.prepare_to_render_to_active_framebuffer(width, height);
                        renderer.render(next_action);
                    }

                    // Swap buffers to finish the drawing
                    if show_frame_buffer {
                        current_context.swap_buffers().ok();
                    }

                    // Release the current context
                    let context     = current_context.make_not_current();
                    let context     = if let Ok(context) = context { context } else { break; };
                    window.context  = Some(context);
                    window.surface  = Some(surface);

                    // Notify that a new frame has been drawn
                    events.publish(DrawEvent::NewFrame).await;
                }
            }

            WindowUpdate::SetTitle(new_title)   => {
                window.context.as_ref().map(|ctxt| ctxt.window().set_title(&new_title));
            }

            WindowUpdate::SetSize((size_x, size_y)) => {
                window.context.as_ref().map(|ctxt| ctxt.window().set_inner_size(LogicalSize::new(size_x as f64, size_y as _)));
            }

            WindowUpdate::SetFullscreen(is_fullscreen) => {
                let fullscreen = if is_fullscreen { Some(Fullscreen::Borderless(None)) } else { None };
                window.context.as_ref().map(|ctxt| ctxt.window().set_fullscreen(fullscreen));
            }

            WindowUpdate::SetHasDecorations(decorations) => {
                window.context.as_ref().map(|ctxt| ctxt.window().set_decorations(decorations));
            }

            WindowUpdate::SetMousePointer(MousePointer::None) => {
                window.context.as_ref().map(|ctxt| ctxt.window().set_cursor_visible(false));
            }

            WindowUpdate::SetMousePointer(MousePointer::SystemDefault) => {
                window.context.as_ref().map(|ctxt| ctxt.window().set_cursor_visible(true));
            }
        }
    }

    // Window will close once the render actions are finished as we drop it here
}

///
/// The list of update events that can occur to a window
///
enum WindowUpdate {
    Render(Vec<RenderAction>),
    SetTitle(String),
    SetSize((u64, u64)),
    SetFullscreen(bool),
    SetHasDecorations(bool),
    SetMousePointer(MousePointer)
}

///
/// Stream that merges the streams from the window properties and the renderer into a single stream
///
struct WindowUpdateStream<TRenderStream, TTitleStream, TSizeStream, TFullscreenStream, TDecorationStream, TMousePointerStream> {
    render_stream:      TRenderStream,
    title_stream:       TTitleStream,
    size:               TSizeStream,
    fullscreen:         TFullscreenStream,
    has_decorations:    TDecorationStream,
    mouse_pointer:      TMousePointerStream
}

impl<TRenderStream, TTitleStream, TSizeStream, TFullscreenStream, TDecorationStream, TMousePointerStream> Stream for WindowUpdateStream<TRenderStream, TTitleStream, TSizeStream, TFullscreenStream, TDecorationStream, TMousePointerStream>
where
TRenderStream:          Unpin+Stream<Item=Vec<RenderAction>>,
TTitleStream:           Unpin+Stream<Item=String>,
TSizeStream:            Unpin+Stream<Item=(u64, u64)>,
TFullscreenStream:      Unpin+Stream<Item=bool>,
TDecorationStream:      Unpin+Stream<Item=bool>,
TMousePointerStream:    Unpin+Stream<Item=MousePointer> {
    type Item = WindowUpdate;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll each stream in turn to see if they have an item

        // Rendering instructions have priority
        match self.render_stream.poll_next_unpin(context) {
            Poll::Ready(Some(item)) => { return Poll::Ready(Some(WindowUpdate::Render(item))); }
            Poll::Ready(None)       => { return Poll::Ready(None); }
            Poll::Pending           => { }
        }

        // The various binding streams
        match self.title_stream.poll_next_unpin(context) {
            Poll::Ready(Some(item)) => { return Poll::Ready(Some(WindowUpdate::SetTitle(item))); }
            Poll::Ready(None)       => { return Poll::Ready(None); }
            Poll::Pending           => { }
        }

        match self.size.poll_next_unpin(context) {
            Poll::Ready(Some(item)) => { return Poll::Ready(Some(WindowUpdate::SetSize(item))); }
            Poll::Ready(None)       => { return Poll::Ready(None); }
            Poll::Pending           => { }
        }

        match self.fullscreen.poll_next_unpin(context) {
            Poll::Ready(Some(item)) => { return Poll::Ready(Some(WindowUpdate::SetFullscreen(item))); }
            Poll::Ready(None)       => { return Poll::Ready(None); }
            Poll::Pending           => { }
        }

        match self.has_decorations.poll_next_unpin(context) {
            Poll::Ready(Some(item)) => { return Poll::Ready(Some(WindowUpdate::SetHasDecorations(item))); }
            Poll::Ready(None)       => { return Poll::Ready(None); }
            Poll::Pending           => { }
        }

        match self.mouse_pointer.poll_next_unpin(context) {
            Poll::Ready(Some(item)) => { return Poll::Ready(Some(WindowUpdate::SetMousePointer(item))); }
            Poll::Ready(None)       => { return Poll::Ready(None); }
            Poll::Pending           => { }
        }

        // No stream matched anything
        Poll::Pending
    }
}
