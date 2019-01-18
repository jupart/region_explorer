use std::fs::File;
use std::io::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

const TEXT_BORDER_SIZE: i32 = 3;
const TEXT_PADDING_SIZE: i32 = 3;
const BUTTON_SIZE: f64 = 16.0;
const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MapPoint {
    pub x: f64,
    pub y: f64,
    pub description: String,
}

impl MapPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, description: String::new() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedRegionData {
    name: String,
    image: String,
    description: String,
    points: Vec<MapPoint>,
}

pub struct RegionData {
    name: String,
    image: String,
    description: String,
    points: Rc<RefCell<Vec<MapPoint>>>,
}

impl RegionData {
    pub fn from_path(path: &str) -> Self {
        let mut region_file = File::open(path).expect(&format!("Error opening {}", path));
        let mut ron_data = String::new();
        region_file.read_to_string(&mut ron_data).unwrap();
        let region_data: SerializedRegionData = ron::de::from_bytes(ron_data.as_bytes()).expect(&format!("{} doesn't match expected RegionData format", path));
        Self {
            name: region_data.name,
            image: region_data.image,
            description: region_data.description,
            points: Rc::new(RefCell::new(region_data.points))
        }
    }

    fn save(&self) {

    }

    fn build_menu_box(&self) -> gtk::Box {
        // Menu to read and write the RegionData file
        let menu_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let check_button = gtk::CheckButton::new_with_label("Read only");
        let read_button = gtk::Button::new_with_label("Read");
        let write_button = gtk::Button::new_with_label("Write");
        read_button.connect_clicked(|button| {
            let open_dialog = gtk::FileChooserDialog::with_buttons::<gtk::Window>(
                Some("Open File"),
                None,
                gtk::FileChooserAction::Open,
                &[("_Cancel", gtk::ResponseType::Cancel), ("_Open", gtk::ResponseType::Accept)]
            );
        });
        write_button.connect_clicked(|button| {
            let save_as_dialog = gtk::FileChooserDialog::with_buttons::<gtk::Window>(
                Some("Save File As"),
                None,
                gtk::FileChooserAction::Save,
                &[("_Cancel", gtk::ResponseType::Cancel), ("_Save", gtk::ResponseType::Accept)]
            );
        });
        menu_box.pack_end(&read_button, false, true, 0);
        menu_box.pack_end(&check_button, false, true, 0);
        menu_box.pack_end(&write_button, false, true, 0);
        menu_box
    }

    fn get_scroll(scrolled_window: &gtk::ScrolledWindow) -> (f64, f64) {
        let h_scroll = scrolled_window.get_hadjustment().unwrap().get_value();
        let v_scroll = scrolled_window.get_vadjustment().unwrap().get_value();
        (h_scroll, v_scroll)
    }

    fn get_map_marker(point: &MapPoint) -> gtk::Button {
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
        marker.show();
        marker
    }

    fn build_map_overlay(&self) -> gtk::Overlay {
        // The map image and the overlaid buttons
        let overlay = gtk::Overlay::new();
        let points = Rc::clone(&self.points);
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
            let points = points.borrow();
            let point = points.get(point_index.unwrap()).expect("MapPoint for child doesn't exist");

            // Get scroll info
            let probably_scrolled_window = &parent.get_child().unwrap();
            let scrolled_window = probably_scrolled_window.downcast_ref::<gtk::ScrolledWindow>().expect("Overlay doesn't have ScrolledWindow as base child");
            let (h_scroll, v_scroll) = Self::get_scroll(scrolled_window);

            Some(gtk::Rectangle {
                x: (point.x - h_scroll - BUTTON_SIZE / 2.0) as i32,
                y: (point.y - v_scroll - BUTTON_SIZE / 2.0) as i32,
                width: BUTTON_SIZE as i32,
                height: BUTTON_SIZE as i32
            })
        });
        let map = gtk::ScrolledWindow::new(None, None);
        let image = gtk::Image::new_from_file("./resources/kellua saari.png");
        map.add(&image);
        map.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        let points = Rc::clone(&self.points);
        map.connect_button_press_event(move |map, event| {
            let right_click = 3;
            if event.get_button() == right_click {
                let probably_overlay = map.get_parent().unwrap();
                let overlay = probably_overlay.downcast_ref::<gtk::Overlay>().unwrap();
                let scrolled_window = map.downcast_ref::<gtk::ScrolledWindow>().expect("Overlay doesn't have ScrolledWindow as base child");
                let (h_scroll, v_scroll) = Self::get_scroll(scrolled_window);

                let (x, y) = event.get_position();
                let new_point = MapPoint::new(x + h_scroll, y + v_scroll);
                println!("Creating new point at {:?}", new_point);
                points.borrow_mut().push(new_point.clone());
                overlay.add_overlay(&Self::get_map_marker(&new_point));
            }
            Inhibit(false)
        });
        overlay.add(&map);

        let points = self.points.borrow();
        for point in points.iter() {
            let marker = Self::get_map_marker(&point);
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
