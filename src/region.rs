use std::fs::File;
use std::io::prelude::*;

use gtk::prelude::*;

const TEXT_BORDER_SIZE: i32 = 3;
const TEXT_PADDING_SIZE: i32 = 3;
const BUTTON_SIZE: i32 = 20;
//pub const WINDOW_WIDTH: f32 = 800.0;
//pub const WINDOW_HEIGHT: f32 = 600.0;
//const DESCRIPTION_CAPACITY: usize = 5000;
//const IMAGE_FRAME_WIDTH: f32 = 500.0;
//const IMAGE_FRAME_HEIGHT: f32 = 540.0;
//const TEXT_FRAME_WIDTH: f32 = 270.0;
//const TEXT_FRAME_HEIGHT: f32 = 540.0;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MapPoint {
    pub x: f32,
    pub y: f32,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegionData {
    name: String,
    image: String,
    description: String,
    points: Vec<MapPoint>,
}

impl RegionData {
    pub fn from_path(path: &str) -> Self {
        let mut region_file = File::open(path).expect(&format!("Error opening {}", path));
        let mut ron_data = String::new();
        region_file.read_to_string(&mut ron_data).unwrap();
        ron::de::from_bytes(ron_data.as_bytes()).expect(&format!("{} doesn't match expected RegionData format", path))
    }

    fn build_menu_box(&self) -> gtk::Box {
        // Menu to read and write the RegionData file
        let menu_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let file_chooser = gtk::FileChooserButton::new("Open file", gtk::FileChooserAction::Open);
        let check_button = gtk::CheckButton::new_with_label("Read only");
        let read_button = gtk::Button::new_with_label("Read");
        let write_button = gtk::Button::new_with_label("Write");
        menu_box.add(&file_chooser);
        menu_box.pack_end(&read_button, false, true, 0);
        menu_box.pack_end(&check_button, false, true, 0);
        menu_box.pack_end(&write_button, false, true, 0);
        menu_box
    }

    fn get_scroll(scrolled_window: &gtk::ScrolledWindow) -> (i32, i32) {
        let h = scrolled_window.get_hscrollbar().expect("ScrolledWindow doesn't have a horizontal scrollbar");
        let v = scrolled_window.get_vscrollbar().expect("ScrolledWindow doesn't have a vertical scrollbar");
        let h_scroll = h.downcast_ref::<gtk::Scrollbar>().unwrap().get_value() as i32;
        let v_scroll = v.downcast_ref::<gtk::Scrollbar>().unwrap().get_value() as i32;
        (h_scroll, v_scroll)
    }

    fn get_map_marker(&self, point: &MapPoint) -> gtk::Button {
        let marker = gtk::Button::new();
        let popup = gtk::Popover::new(&marker);
        let text_buffer = gtk::TextBuffer::new(None);
        text_buffer.set_text(&point.description);
        let text_view = gtk::TextView::new_with_buffer(&text_buffer);
        text_view.set_wrap_mode(gtk::WrapMode::Word);
        text_view.set_border_window_size(gtk::TextWindowType::Left, TEXT_BORDER_SIZE);
        text_view.set_border_window_size(gtk::TextWindowType::Right, TEXT_BORDER_SIZE);
        text_view.set_border_window_size(gtk::TextWindowType::Top, TEXT_BORDER_SIZE);
        text_view.set_border_window_size(gtk::TextWindowType::Bottom, TEXT_BORDER_SIZE);
        text_view.set_left_margin(TEXT_PADDING_SIZE);
        text_view.set_right_margin(TEXT_PADDING_SIZE);
        popup.add(&text_view);
        popup.set_size_request(300, 300);
        marker.connect_clicked(move |_| {
            popup.show_all();
        });
        marker
    }

    fn build_map_overlay(&self) -> gtk::Overlay {
        // The map image and the overlaid buttons
        let overlay = gtk::Overlay::new();
        let copied_points = self.points.clone();
        overlay.connect_get_child_position(move |parent, child| {
            // Get position of child from self.points
            let mut point_index = None;
            for (i, widget) in parent.get_children().iter().enumerate() {
                if widget == child {
                    point_index = Some(i - 1);
                    break;
                }
            };
            if point_index.is_none() {
                // We didn't find a point match for some reason
                return None;
            }
            let point = copied_points.get(point_index.unwrap()).expect("MapPoint for child doesn't exist");

            // Get scroll info
            let kid = &parent.get_children()[0];
            let scrolled_window = kid.downcast_ref::<gtk::ScrolledWindow>().expect("Overlay doesn't have ScrolledWindow as base child");
            let (h_scroll, v_scroll) = Self::get_scroll(scrolled_window);

            Some(gtk::Rectangle {
                x: point.x as i32 - h_scroll - BUTTON_SIZE / 2,
                y: point.y as i32 - v_scroll - BUTTON_SIZE / 2,
                width: BUTTON_SIZE,
                height: BUTTON_SIZE
            })
        });
        let scrolled_window = gtk::ScrolledWindow::new(None, None);
        let image = gtk::Image::new_from_file("./resources/kellua saari.png");
        scrolled_window.add(&image);
        overlay.add(&scrolled_window);

        for point in &self.points {
            let marker = self.get_map_marker(&point);
            overlay.add_overlay(&marker);
        }

        overlay
    }

    fn create_window(&self, application: &gtk::Application) -> gtk::ApplicationWindow {
        let window = gtk::ApplicationWindow::new(application);
        window.set_title("float");
        window.set_border_width(5);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(800, 600);
        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });
        window
    }

    pub fn build_ui(&self, application: &gtk::Application) {
        let window = self.create_window(application);
        let menu_box = self.build_menu_box();
        let overlay = self.build_map_overlay();

        // Putting it all together
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
        main_box.add(&menu_box);
        main_box.pack_end(&overlay, true, true, 0);
        window.add(&main_box);
        window.show_all();
    }

//    fn mouse_in_region(&self, ui: &Ui) -> bool {
//        let current_mouse_pos = ui.imgui().mouse_pos();
//        (current_mouse_pos.0 < IMAGE_FRAME_WIDTH)
//            && (current_mouse_pos.0 > 10.0)
//            && (current_mouse_pos.1 > 40.0)
//            && (current_mouse_pos.1 < IMAGE_FRAME_HEIGHT + 40.0)
//    }

//    fn mouse_in_frame_coords(&self, ui: &Ui) -> (f32, f32) {
//        let current_mouse_pos = ui.imgui().mouse_pos();
//        (current_mouse_pos.0 - 17.0, current_mouse_pos.1 - 54.0)
//    }

//    fn create_backup(&self) {
//        std::fs::copy("./resources/kellua_saari.ron", "./resources/kellua_saari.backup").unwrap();
//    }

//    fn write_file(&self) {
//        self.create_backup();

//        let region = RegionData {
//            name: self.name.clone(),
//            image: self.image_name.clone(),
//            description: self.region_description.clone(),
//            points: self.points.clone(),
//        };

//        let pretty = ron::ser::PrettyConfig::default();
//        let ron_string = ron::ser::to_string_pretty(&region, pretty).unwrap();
//        std::fs::write("./resources/kellua_saari.ron", ron_string).unwrap();
//    }

//    fn remove_point(&mut self, point_to_delete: Option<i32>) {
//        if point_to_delete.is_none() {
//            return;
//        }
//        let point = point_to_delete.unwrap();

//        // Shift self.selected_point as needed, depending on position of point
//        if point == self.selected_point {
//            self.selected_point = -1;
//            self.current_description = self.region_description.clone();
//        } else if point < self.selected_point {
//            self.selected_point -= 1;
//        }
//        self.points.remove(point as usize);
//    }

//    fn handle_input(&mut self, ui: &Ui) {
//        if ui.imgui().is_key_pressed(VirtualKeyCode::Escape as usize) {
//            self.to_delete = false;
//        }

//        // TODO - zoom is working, but there are issues
//        //      1. I'd like to zoom to mouse pos on scroll
//        //      2. Radio buttons are not drawn in the right position at all zoom levels
//        //
//        // let scroll = ui.imgui().mouse_wheel();
//        // if scroll != 0.0 {
//        //     if self.mouse_in_region(&ui) {
//        //         if scroll > 0.0 {
//        //             self.zoom += 0.1;
//        //         } else {
//        //             self.zoom -= 0.1;
//        //         }

//        //         if self.zoom > 1.0 {
//        //             self.zoom = 1.0;
//        //         }
//        //         if self.zoom < 0.1 {
//        //             self.zoom = 0.1;
//        //         }
//        //         self.image_pos = ((-self.image_size.0 * self.zoom + IMAGE_FRAME_WIDTH) / 2.0, (-self.image_size.1 * self.zoom + IMAGE_FRAME_HEIGHT) / 2.0);
//        //     }
//        // }
//        if ui.imgui().is_mouse_clicked(ImMouseButton::Right) {
//            if self.mouse_in_region(&ui) && !self.readonly {
//                let frame_click = self.mouse_in_frame_coords(&ui);
//                let point_x = frame_click.0 - self.image_pos.0;
//                let scaled_point_x = point_x / self.zoom;
//                let point_y = frame_click.1 - self.image_pos.1;
//                let scaled_point_y = point_y / self.zoom;
//                self.points.push(MapPoint::new(scaled_point_x, scaled_point_y, "New shit".to_owned()));
//            }
//        }
//        if ui.imgui().is_mouse_dragging(ImMouseButton::Middle) {
//            let delta = ui.imgui().mouse_delta();
//            self.image_pos.0 += delta.0;
//            self.image_pos.1 += delta.1;
//        }
//    }

//    pub fn do_ui(&mut self, ui: &Ui) {
//        self.handle_input(ui);

//        let mut desc = ImString::with_capacity(DESCRIPTION_CAPACITY);
//        desc.push_str(self.current_description.as_str());

//        ui.window(im_str!(""))
//            .position((0.0, 0.0), ImGuiCond::Once)
//            .scroll_bar(false)
//            .resizable(false)
//            .scrollable(false)
//            .size((WINDOW_WIDTH, WINDOW_HEIGHT), ImGuiCond::Once)
//            .build(|| {
//                // Headers
//                ui.text(self.name.as_str());

//                // For deleting points
//                ui.same_line(IMAGE_FRAME_WIDTH - 100.0);
//                ui.checkbox(im_str!("Delete mode"), &mut self.to_delete);

//                // Is readonly?
//                ui.same_line(IMAGE_FRAME_WIDTH + 20.0);
//                ui.checkbox(im_str!("Read-only"), &mut self.readonly);

//                if self.to_delete {
//                    ui.imgui().set_mouse_cursor(ImGuiMouseCursor::Hand);
//                } else {
//                    ui.imgui().set_mouse_cursor(ImGuiMouseCursor::Arrow);
//                }
//                if self.readonly {
//                    self.to_delete = false;
//                }

//                // Write changed descriptions and new points to file
//                ui.same_line(IMAGE_FRAME_WIDTH + 244.0);
//                if ui.button(im_str!("Write"), ImVec2::new(45.0, 18.0)) && !self.readonly {
//                    self.write_file();
//                }

//                // Map
//                let mut point_to_delete: Option<i32> = None;
//                ui.child_frame(im_str!("Map"), (IMAGE_FRAME_WIDTH, IMAGE_FRAME_HEIGHT))
//                    .movable(false)
//                    .show_scrollbar_with_mouse(false)
//                    .show_scrollbar(false)
//                    .scrollbar_horizontal(false)
//                    .build(|| {
//                        ui.set_cursor_pos(self.image_pos);
//                        ui.image(self.image, ImVec2::new(self.image_size.0 * self.zoom, self.image_size.1 * self.zoom)).build();

//                        let mut i = 0;
//                        for point in &self.points {
//                            let draw_point = (
//                                self.image_pos.0 + (point.x - 4.0) * self.zoom,
//                                self.image_pos.1 + (point.y - 10.0) * self.zoom
//                            );
//                            ui.set_cursor_pos(draw_point);
//                            ui.push_id(i);

//                            let point_before_select = self.selected_point;
//                            if ui.radio_button(im_str!(""), &mut self.selected_point, i) {
//                                if !self.to_delete {
//                                    self.current_description = point.description.clone();
//                                } else {
//                                    self.selected_point = point_before_select;
//                                    point_to_delete = Some(i);
//                                }
//                            }
//                            ui.pop_id();
//                            i += 1;
//                        }
//                    });

//                self.remove_point(point_to_delete);

//                // Description
//                ui.same_line(IMAGE_FRAME_WIDTH + 20.0);
//                ui.child_frame(im_str!("Description"), (TEXT_FRAME_WIDTH, TEXT_FRAME_HEIGHT))
//                    .scrollbar_horizontal(true)
//                    .build(|| {
//                        let changed = ui.input_text_multiline(im_str!("Input"), &mut desc, ImVec2::new(TEXT_FRAME_WIDTH, TEXT_FRAME_HEIGHT)).build();
//                        if changed && !self.readonly {
//                            let desc_str: &str = desc.as_ref();
//                            self.current_description = desc_str.to_owned();
//                            let current_point = self.points.get_mut(self.selected_point as usize);
//                            if current_point.is_some() {
//                                current_point.unwrap().description = self.current_description.clone();
//                            } else {
//                                self.region_description = self.current_description.clone();
//                            }
//                        }
//                    });
//            });
//    }
}

