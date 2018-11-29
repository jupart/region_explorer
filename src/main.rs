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
use std::io::prelude::*;
use imgui::*;
use imgui_gfx_renderer::{Renderer, Shaders};
use std::time::Instant;
use gfx::{Device};
use glutin::{GlContext};

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const DESCRIPTION_CAPACITY: usize = 5000;
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const IMAGE_FRAME_WIDTH: f32 = 500.0;
const IMAGE_FRAME_HEIGHT: f32 = 540.0;
const TEXT_FRAME_WIDTH: f32 = 270.0;
const TEXT_FRAME_HEIGHT: f32 = 540.0;

#[derive(Serialize, Deserialize, Clone)]
struct MapPoint {
    x: f32,
    y: f32,
    description: String,
}

impl MapPoint {
    pub fn new(x: f32, y: f32, description: String) -> Self {
        Self { x, y, description, }
    }
}

#[derive(Serialize, Deserialize)]
struct RegionData {
    name: String,
    image: String,
    description: String,
    points: Vec<MapPoint>,
}

struct RegionWindow {
    name: String,
    region_description: String,
    current_description: String,
    image: ImTexture,
    image_name: String,
    image_size: (f32, f32),
    image_pos: (f32, f32),
    points: Vec<MapPoint>,
    selected_point: i32,
    zoom: f32,
    readonly: bool,
}

impl RegionWindow {
    fn mouse_in_region(&self, ui: &Ui) -> bool {
        let current_mouse_pos = ui.imgui().mouse_pos();
        (current_mouse_pos.0 < IMAGE_FRAME_WIDTH)
            && (current_mouse_pos.0 > 10.0)
            && (current_mouse_pos.1 > 40.0)
            && (current_mouse_pos.1 < IMAGE_FRAME_HEIGHT + 40.0)
    }

    fn mouse_in_frame_coords(&self, ui: &Ui) -> (f32, f32) {
        let current_mouse_pos = ui.imgui().mouse_pos();
        (current_mouse_pos.0 - 17.0, current_mouse_pos.1 - 54.0)
    }

    fn create_backup(&self) {
        println!("Created backup");
        std::fs::copy("./resources/kellua_saari.ron", "./resources/kellua_saari.backup").unwrap();
    }

    fn write_file(&self) {
        self.create_backup();
        println!("Write dat file bebe");

        let region = RegionData {
            name: self.name.clone(),
            image: self.image_name.clone(),
            description: self.region_description.clone(),
            points: self.points.clone(),
        };

        let pretty = ron::ser::PrettyConfig::default();
        let ron_string = ron::ser::to_string_pretty(&region, pretty).unwrap();
        std::fs::write("./resources/kellua_saari.ron", ron_string).unwrap();
    }

    pub fn do_ui(&mut self, ui: &Ui) {
        // TODO - zoom is working, but there are issues
        //      1. I'd like to zoom to mouse pos on scroll
        //      2. Radio buttons are not drawn in the right position at all zoom levels
        //
        // let scroll = ui.imgui().mouse_wheel();
        // if scroll != 0.0 {
        //     if self.mouse_in_region(&ui) {
        //         if scroll > 0.0 {
        //             self.zoom += 0.1;
        //         } else {
        //             self.zoom -= 0.1;
        //         }

        //         if self.zoom > 1.0 {
        //             self.zoom = 1.0;
        //         }
        //         if self.zoom < 0.1 {
        //             self.zoom = 0.1;
        //         }
        //         self.image_pos = ((-self.image_size.0 * self.zoom + IMAGE_FRAME_WIDTH) / 2.0, (-self.image_size.1 * self.zoom + IMAGE_FRAME_HEIGHT) / 2.0);
        //     }
        // }
        if ui.imgui().is_mouse_clicked(ImMouseButton::Right) {
            if self.mouse_in_region(&ui) && !self.readonly {
                let frame_click = self.mouse_in_frame_coords(&ui);
                let point_x = frame_click.0 - self.image_pos.0;
                let scaled_point_x = point_x / self.zoom;
                let point_y = frame_click.1 - self.image_pos.1;
                let scaled_point_y = point_y / self.zoom;
                self.points.push(MapPoint::new(scaled_point_x, scaled_point_y, "New shit".to_owned()));
            }
        }
        if ui.imgui().is_mouse_dragging(ImMouseButton::Middle) {
            let delta = ui.imgui().mouse_delta();
            self.image_pos.0 += delta.0;
            self.image_pos.1 += delta.1;
        }

        let mut desc = ImString::with_capacity(DESCRIPTION_CAPACITY);
        desc.push_str(self.current_description.as_str());

        ui.window(im_str!("Kellua Saari"))
            .position((0.0, 0.0), ImGuiCond::Once)
            .scroll_bar(false)
            .resizable(false)
            .scrollable(false)
            .size((WINDOW_WIDTH, WINDOW_HEIGHT), ImGuiCond::Once)
            .build(|| {
                // Headers
                ui.text(self.name.as_str());
                ui.same_line(IMAGE_FRAME_WIDTH + 20.0);

                // Is readonly?
                ui.checkbox(im_str!("Read-only"), &mut self.readonly);
                ui.same_line(IMAGE_FRAME_WIDTH + 244.0);

                // Write changed descriptions and new points to file
                if ui.button(im_str!("Write"), ImVec2::new(45.0, 18.0)) && !self.readonly {
                    self.write_file();
                }

                // Map
                ui.child_frame(im_str!("Map"), (IMAGE_FRAME_WIDTH, IMAGE_FRAME_HEIGHT))
                    .movable(false)
                    .show_scrollbar_with_mouse(false)
                    .show_scrollbar(false)
                    .scrollbar_horizontal(false)
                    .build(|| {
                        ui.set_cursor_pos(self.image_pos);
                        ui.image(self.image, ImVec2::new(self.image_size.0 * self.zoom, self.image_size.1 * self.zoom)).build();

                        let mut i = 0;
                        for point in &self.points {
                            let draw_point = (
                                self.image_pos.0 + (point.x - 4.0) * self.zoom,
                                self.image_pos.1 + (point.y - 10.0) * self.zoom
                            );
                            ui.set_cursor_pos(draw_point);
                            ui.push_id(i);
                            if ui.radio_button(im_str!(""), &mut self.selected_point, i) {
                                self.current_description = point.description.clone();
                            }
                            ui.pop_id();
                            i += 1;
                        }
                    });
                ui.same_line(IMAGE_FRAME_WIDTH + 20.0);

                // Description
                ui.child_frame(im_str!("Description"), (TEXT_FRAME_WIDTH, TEXT_FRAME_HEIGHT))
                    .scrollbar_horizontal(true)
                    .build(|| {
                        let changed = ui.input_text_multiline(im_str!("Input"), &mut desc, ImVec2::new(TEXT_FRAME_WIDTH, TEXT_FRAME_HEIGHT)).build();
                        if changed && !self.readonly {
                            let desc_str: &str = desc.as_ref();
                            self.current_description = desc_str.to_owned();
                            let current_point = self.points.get_mut(self.selected_point as usize);
                            if current_point.is_some() {
                                current_point.unwrap().description = self.current_description.clone();
                            } else {
                                self.region_description = self.current_description.clone();
                            }
                        }
                    });
            });
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let window = glutin::WindowBuilder::new()
        .with_title("Region Explorer")
        .with_dimensions(glutin::dpi::LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64));
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

    let region_data = get_region_data("kellua_saari.ron");

    let (tex, sampler, size) = load_texture(&mut factory, &region_data.image).unwrap();
    let image = renderer.textures().insert((tex, sampler));

    let mut region_window = RegionWindow {
        name: region_data.name,
        region_description: region_data.description.clone(),
        current_description: region_data.description,
        image: image,
        image_name: region_data.image,
        image_size: (size.0 as f32, size.1 as f32),
        image_pos: (0.0, 0.0),
        points: region_data.points,
        selected_point: -1,
        zoom: 0.6,
        readonly: false,
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

fn get_region_data(path: &str) -> RegionData {
    let mut resource_path = String::from("./resources/");
    resource_path.push_str(path);
    let mut region_file = File::open(resource_path).unwrap();
    let mut ron_data = String::new();
    region_file.read_to_string(&mut ron_data).unwrap();
    ron::de::from_bytes(ron_data.as_bytes()).unwrap()
}
