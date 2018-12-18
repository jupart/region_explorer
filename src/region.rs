use std::fs::File;
use std::io::prelude::*;

use gtk::prelude::*;

const TEXT_BORDER_SIZE: i32 = 3;
const TEXT_PADDING_SIZE: i32 = 3;
const BUTTON_SIZE: i32 = 20;
const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MapPoint {
    pub x: f32,
    pub y: f32,
    pub description: String,
}

impl MapPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, description: String::new() }
    }
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
        let map = gtk::ScrolledWindow::new(None, None);
        let image = gtk::Image::new_from_file("./resources/kellua saari.png");
        map.add(&image);
        map.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        map.connect_button_press_event(|map, event| {
            let right_click = 3;
            if event.get_button() == right_click {
                let probably_overlay = map.get_parent().unwrap();
                let overlay = probably_overlay.downcast_ref::<gtk::Overlay>().unwrap();
                let probably_scrollbar = overlay.get_child().unwrap();
                let scrollbar = probably_scrollbar.downcast_ref::<gtk::Scrollbar>().unwrap();

                let (x, y) = event.get_position();
                let new_point = MapPoint::new(x as f32, y as f32);
                println!("Creating new point at {:?}", new_point);
                // overlay.add_overlay(&self.get_map_marker(&new_point));
            }
            Inhibit(false)
        });
        overlay.add(&map);

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
        window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT);
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
}

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
