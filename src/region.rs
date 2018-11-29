use imgui::*;

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;
const DESCRIPTION_CAPACITY: usize = 5000;
const IMAGE_FRAME_WIDTH: f32 = 500.0;
const IMAGE_FRAME_HEIGHT: f32 = 540.0;
const TEXT_FRAME_WIDTH: f32 = 270.0;
const TEXT_FRAME_HEIGHT: f32 = 540.0;

#[derive(Serialize, Deserialize, Clone)]
pub struct MapPoint {
    pub x: f32,
    pub y: f32,
    pub description: String,
}

impl MapPoint {
    pub fn new(x: f32, y: f32, description: String) -> Self {
        Self { x, y, description, }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RegionData {
    pub name: String,
    pub image: String,
    pub description: String,
    pub points: Vec<MapPoint>,
}

pub struct RegionWindow {
    pub name: String,
    pub region_description: String,
    pub current_description: String,
    pub image: ImTexture,
    pub image_name: String,
    pub image_size: (f32, f32),
    pub image_pos: (f32, f32),
    pub points: Vec<MapPoint>,
    pub selected_point: i32,
    pub zoom: f32,
    pub readonly: bool,
}

impl RegionWindow {
    pub fn new(name: String, desc: String, img: ImTexture, img_name: String, img_size: (u32, u32), points: Vec<MapPoint>) -> Self {
        Self {
            name,
            region_description: desc.clone(),
            current_description: desc,
            image: img,
            image_name: img_name,
            image_size: (img_size.0 as f32, img_size.1 as f32),
            image_pos: (0.0, 0.0),
            points,
            selected_point: -1,
            zoom: 0.6,
            readonly: false,
        }
    }

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

