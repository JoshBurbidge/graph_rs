#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_imports {
    pub use glutin::{
        config::ConfigTemplateBuilder,
        context::{ContextAttributesBuilder, PossiblyCurrentContext},
        display::GetGlDisplay,
        prelude::*,
        surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
    };
    pub use glutin_winit::DisplayBuilder;
    #[allow(deprecated)]
    pub use raw_window_handle::HasRawWindowHandle;
    pub use std::num::NonZeroU32;
    pub use winit::{dpi::PhysicalSize, window::WindowAttributes};
}
#[cfg(not(target_arch = "wasm32"))]
use non_wasm_imports::*;

use femtovg::renderer::OpenGl;
use femtovg::Canvas;
use winit::event_loop::EventLoop;
use winit::window::Window;

use super::handler::MyApplicationHandler;
use crate::grapher::equation::Polynomial;

pub fn init_canvas<T>(
    event_loop: &EventLoop<T>,
    equations: Vec<Polynomial>,
) -> MyApplicationHandler {
    #[cfg(not(target_arch = "wasm32"))]
    let (current_context, canvas, window, surface) = init_native(event_loop);

    #[cfg(target_arch = "wasm32")]
    let (canvas, window) = init_wasm(event_loop);

    let default_scale = 50.;

    window.focus_window();
    let app = MyApplicationHandler::new(
        window,
        #[cfg(not(target_arch = "wasm32"))]
        current_context,
        #[cfg(not(target_arch = "wasm32"))]
        surface,
        canvas,
        default_scale,
        equations,
    );

    app
}

#[cfg(not(target_arch = "wasm32"))]
fn init_native<T>(
    event_loop: &EventLoop<T>,
) -> (
    PossiblyCurrentContext,
    Canvas<OpenGl>,
    Window,
    Surface<WindowSurface>,
) {
    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let window_attr = WindowAttributes::default()
        .with_inner_size(PhysicalSize::new(1000., 600.))
        .with_title("graph_rs");
    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attr));

    let (window, gl_config) = display_builder
        .build(event_loop, template, |mut configs| configs.next().unwrap())
        .unwrap();

    let window = window.unwrap();

    let gl_display = gl_config.display();

    #[allow(deprecated)]
    let context_attributes = ContextAttributesBuilder::new().build(Some(
        window
            .raw_window_handle()
            .expect("raw window handle failed"),
    ));

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap()
    });

    #[allow(deprecated)]
    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window
            .raw_window_handle()
            .expect("raw window handle failed"),
        NonZeroU32::new(1000).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let current_context = not_current_gl_context
        .take()
        .unwrap()
        .make_current(&surface)
        .unwrap();

    let renderer =
        unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
            .expect("Cannot create renderer");

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(1000, 600, window.scale_factor() as f32);

    (current_context, canvas, window, surface)
}

#[cfg(target_arch = "wasm32")]
fn init_wasm<T>(event_loop: &EventLoop<T>) -> (Canvas<OpenGl>, Window) {
    use wasm_bindgen::JsCast;
    use winit::platform::web::WindowAttributesExtWebSys;

    let html_canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let renderer = OpenGl::new_from_html_canvas(&html_canvas).expect("Cannot create renderer");

    let window_attrs = Window::default_attributes().with_canvas(Some(html_canvas));
    #[allow(deprecated)]
    let window = event_loop.create_window(window_attrs).unwrap();
    // could maybe call these init functions from the resume handler, then event loop would be active

    let canvas = Canvas::new(renderer).expect("Cannot create canvas");

    (canvas, window)
}
