extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
// #[macro_use]
extern crate imgui;
extern crate imgui_gfx_renderer;
extern crate imgui_glutin_support;
extern crate image;

use imgui::*;
use imgui_gfx_renderer::{Renderer, Shaders};
use std::time::Instant;
use gfx::{Device};
use glutin::{GlContext};

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

fn main() {
    type ColorFormat = gfx::format::Rgba8;
    type DepthFormat = gfx::format::DepthStencil;

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let window = glutin::WindowBuilder::new()
        .with_title("Region Explorer")
        .with_dimensions(glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let (window, mut device, mut factory, mut main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window, context, &events_loop);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let shaders = {
        let version = device.get_info().shading_language;
        if version.is_embedded {
            if version.major >= 3 {
                Shaders::GlSlEs300
            } else {
                Shaders::GlSlEs100
            }
        } else if version.major >= 4 {
            Shaders::GlSl400
        } else if version.major >= 3 {
            if version.minor >= 2 {
                Shaders::GlSl150
            } else {
                Shaders::GlSl130
            }
        } else {
            Shaders::GlSl110
        }
    };

    let mut imgui = ImGui::init();
    {
        // Fix incorrect colors with sRGB framebuffer
        fn imgui_gamma_to_linear(col: ImVec4) -> ImVec4 {
            let x = col.x.powf(2.2);
            let y = col.y.powf(2.2);
            let z = col.z.powf(2.2);
            let w = 1.0 - (1.0 - col.w).powf(2.2);
            ImVec4::new(x, y, z, w)
        }

        let style = imgui.style_mut();
        for col in 0..style.colors.len() {
            style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
        }
    }
    imgui.set_ini_filename(None);

    // In the examples we only use integer DPI factors, because the UI can get very blurry
    // otherwise. This might or might not be what you want in a real application.
    let hidpi_factor = window.get_hidpi_factor().round();

    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size),
    );

    imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

    let mut renderer = Renderer::init(&mut imgui, &mut factory, shaders, main_color.clone())
        .expect("Failed to initialize renderer");

    let (tex, sampler, size) = load_texture(&mut factory, &include_bytes!("../resources/kellua saari.png")[..]).unwrap();
    let image = renderer.textures().insert((tex, sampler));

    imgui_glutin_support::configure_keys(&mut imgui);

    let mut last_frame = Instant::now();
    let mut quit = false;

    loop {
        events_loop.poll_events(|event| {
            use glutin::{
                Event,
                WindowEvent::{CloseRequested, Resized},
            };

            imgui_glutin_support::handle_event(
                &mut imgui,
                &event,
                window.get_hidpi_factor(),
                hidpi_factor,
            );

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    Resized(_) => {
                        gfx_window_glutin::update_views(&window, &mut main_color, &mut main_depth);
                        renderer.update_render_target(main_color.clone());
                    }
                    CloseRequested => quit = true,
                    _ => (),
                }
            }
        });
        if quit {
            break;
        }

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;

        imgui_glutin_support::update_mouse_cursor(&imgui, &window);

        let frame_size = imgui_glutin_support::get_frame_size(&window, hidpi_factor).unwrap();

        let ui = imgui.frame(frame_size, delta_s);
        ui.window(im_str!("Kellua Saari"))
            .scrollable(false)
            .horizontal_scrollbar(true)
            .build(|| {
                ui.image(image, ImVec2::new(size.0 as f32, size.1 as f32)).build();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos.0, mouse_pos.1));
            });


        encoder.clear(&main_color, CLEAR_COLOR);
        renderer.render(ui, &mut factory, &mut encoder).expect("Rendering failed");
        encoder.flush(&mut device);
        window.context().swap_buffers().unwrap();
        device.cleanup();
    }
}

fn load_texture<R, F>(factory: &mut F, data: &[u8]) -> Result<(gfx::handle::ShaderResourceView<R, [f32; 4]>, gfx::handle::Sampler<R>, (u32, u32)), String>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    use std::io::Cursor;
    use gfx::texture as t;
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, t::Mipmap::Provided, &[&img]).unwrap();
    let sampler = factory.create_sampler(t::SamplerInfo::new(t::FilterMethod::Scale, t::WrapMode::Tile));
    Ok((view, sampler, (width, height)))
}
