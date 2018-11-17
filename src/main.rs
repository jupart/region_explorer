extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
// #[macro_use]
extern crate imgui;
extern crate imgui_gfx_renderer;
extern crate imgui_glutin_support;
extern crate image;
extern crate ron;
#[macro_use]
extern crate serde;

use std::fs::File;
use std::collections::HashMap;
use std::io::prelude::*;
use imgui::*;
use imgui_gfx_renderer::{Renderer, Shaders};
use std::time::Instant;
use gfx::{Device};
use glutin::{GlContext};

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const DESCRIPTION_CAPACITY: usize = 5000;

#[derive(Serialize, Deserialize)]
struct RegionData {
    name: String,
    image: String,
    description: String,
    points: HashMap<(u32, u32), String>,
}

struct RegionWindow {
    name: String,
    description: String,
    image: ImTexture,
    image_size: (f32, f32),
    points: HashMap<(u32, u32), String>,
    // image_pos: (f32, f32),
    // update_description: bool,
    // zoom: f32,
    // last_mouse_pos: Option<(f32, f32)>,
    // to_drag: (f32, f32),
}

impl RegionWindow {
    /// If returns false, this window is done, remove it from window list
    pub fn do_ui(&mut self, ui: &Ui) -> bool {
        // let current_mouse_pos = ui.imgui().mouse_pos();
        // if ui.imgui().is_mouse_clicked(ImMouseButton::Right) {
        //     println!("Right clicked!");
        // }
        // if ui.imgui().is_mouse_dragging(ImMouseButton::Middle) {
        //     println!("Yo we draggin'");
        //     if self.last_mouse_pos.is_some() {
        //         let first = self.last_mouse_pos.unwrap().0 - current_mouse_pos.0;
        //         let second = self.last_mouse_pos.unwrap().1 - current_mouse_pos.1;
        //         self.to_drag = (first, second);
        //     }
        // }
        // self.last_mouse_pos = Some(current_mouse_pos);

        let mut desc = ImString::with_capacity(DESCRIPTION_CAPACITY);
        desc.push_str(self.description.as_str());

        ui.window(im_str!("Kellua Saari"))
            .scroll_bar(false)
            .resizable(false)
            .scrollable(false)
            .size((800.0, 600.0), ImGuiCond::Once)
            .build(|| {
                // Headers
                ui.text(self.name.as_str());
                ui.same_line(520.0);
                ui.text("Description");
                ui.same_line(750.0);

                // Save description
                let save = ui.small_button(im_str!("Save"));
                if save {
                    println!("Save it baby");
                }

                // Map
                ui.child_frame(im_str!("Map"), (500.0, 540.0))
                    .always_show_vertical_scroll_bar(false)
                    .scrollbar_horizontal(false)
                    .build(|| {
                        ui.image(self.image, ImVec2::new(self.image_size.0 * 1.0, self.image_size.1 * 1.0)).build();
                    });
                ui.same_line(520.0);

                // Description
                ui.child_frame(im_str!("Description"), (270.0, 540.0))
                    .build(|| {
                        let changed = ui.input_text_multiline(im_str!("Input"), &mut desc, ImVec2::new(270.0, 540.0)).build();
                        if changed {
                        }
                    });
            });
        true
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let window = glutin::WindowBuilder::new()
        .with_title("Region Explorer")
        .with_dimensions(glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let (window, mut device, mut factory, mut main_color, mut main_depth) =
        gfx_window_glutin::init::<gfx::format::Rgba8, gfx::format::DepthStencil>(window, context, &events_loop);
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

    let region_data = get_region_data("./resources/kellua_saari.ron".to_owned());

    let (tex, sampler, size) = load_texture(&mut factory, &region_data.image).unwrap();
    let image = renderer.textures().insert((tex, sampler));

    let mut region_window = RegionWindow {
        name: region_data.name,
        description: region_data.description,
        image: image,
        image_size: (size.0 as f32, size.1 as f32),
        points: region_data.points,
    };

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
        region_window.do_ui(&ui);
        // ui.show_demo_window(&mut true);

        encoder.clear(&main_color, CLEAR_COLOR);
        renderer.render(ui, &mut factory, &mut encoder).expect("Rendering failed");
        encoder.flush(&mut device);
        window.context().swap_buffers().unwrap();
        device.cleanup();
    }
}

fn load_texture<R, F>(factory: &mut F, image_path: &String) -> Result<(gfx::handle::ShaderResourceView<R, [f32; 4]>, gfx::handle::Sampler<R>, (u32, u32)), String>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    use gfx::texture as t;
    let mut resource_path = String::from("./resources/");
    resource_path.push_str(image_path);

    let img = image::open(resource_path).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, t::Mipmap::Provided, &[&img]).unwrap();
    let sampler = factory.create_sampler(t::SamplerInfo::new(t::FilterMethod::Scale, t::WrapMode::Tile));
    Ok((view, sampler, (width, height)))
}

fn get_region_data(path: String) -> RegionData {
    let mut region_file = File::open(path).unwrap();
    let mut ron_data = String::new();
    region_file.read_to_string(&mut ron_data).unwrap();
    ron::de::from_bytes(ron_data.as_bytes()).unwrap()
}
