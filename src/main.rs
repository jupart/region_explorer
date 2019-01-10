extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate serde;

use std::rc::Rc;
use std::cell::RefCell;
use relm::{Relm, Update, Widget};
use gtk::prelude::*;
use gtk::{
    BoxExt,
    // ButtonExt,
    ContainerExt,
    // WidgetExt,
    Inhibit,
    // WindowType,
    // WindowPosition,
    // GtkWindowExt,
    // OverlayExt,
    // LabelExt,
};

mod region;
use self::region::{RegionModel, MapPoint};

const BUTTON_SIZE: f64 = 20.0;

// Messages are sent to `Widget::update` to indicate that an event happened. The model can be
// updated when an event is received.
#[derive(Msg)]
enum Msg {
    HandleClick(gdk::EventButton),
    DeletePoint((f64, f64)),
    UpdateDescription,
    WriteFile,
    ReadFile,
    Quit,
}

// This is just a container to house widgets for the parent widget, which also contains a Model
#[derive(Clone)]
struct Widgets {
    window: gtk::Window,
    window_box: gtk::Box,
    menu_box: gtk::Box,
    file_chooser_button: gtk::FileChooserButton,
    read_only_button: gtk::CheckButton,
    read_file_button: gtk::Button,
    write_file_button: gtk::Button,
    overlay: gtk::Overlay,
    map_window: gtk::ScrolledWindow,
    map_image: gtk::Image,
    location_buttons: Vec<gtk::Button>,
    location_popups: Vec<gtk::Popover>,
}

struct RegionWindow {
    model: Rc<RefCell<RegionModel>>,
    widgets: Widgets,
}

impl Update for RegionWindow {
    type Model = RegionModel;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> RegionModel {
        RegionModel {
            name: String::new(),
            image: String::new(),
            description: String::new(),
            points: vec![],
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::ReadFile => {
                println!("Read file");
            },
            Msg::WriteFile => {
                println!("Write file");
            },
            Msg::UpdateDescription => {
                println!("Update description");
            },
            Msg::DeletePoint(point) => {
                println!("Delete at {:?}", point);
            },
            Msg::HandleClick(event_button) => {
                let right_click = 3;
                if event_button.get_button() != right_click {
                    return;
                }

                let point = event_button.get_position();

                let h_scroll = self.widgets.map_window.get_hadjustment().unwrap().get_value();
                let v_scroll = self.widgets.map_window.get_vadjustment().unwrap().get_value();
                let adjusted_point = (point.0 + h_scroll, point.1 + v_scroll);

                println!("Add at {:?}", adjusted_point);
                let button = gtk::Button::new();
                self.model.borrow_mut().points.push(MapPoint::new(adjusted_point));
                self.widgets.overlay.add_overlay(&button);
                self.widgets.location_buttons.push(button);
            },
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for RegionWindow {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
	// Create our widgets
	let window = gtk::Window::new(gtk::WindowType::Toplevel);
        let window_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
        let menu_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let file_chooser_button = gtk::FileChooserButton::new("Open file", gtk::FileChooserAction::Open);
        let read_only_button = gtk::CheckButton::new_with_label("Read only");
        let read_file_button = gtk::Button::new_with_label("Read");
        let write_file_button = gtk::Button::new_with_label("Write");
        let overlay = gtk::Overlay::new();
        let map_window = gtk::ScrolledWindow::new(None, None);
        let map_image = gtk::Image::new_from_file("./resources/kellua saari.png");
        let location_buttons: Vec<gtk::Button> = vec![];
        let location_popups: Vec<gtk::Popover> = vec![];

        // Assemble the GUI
        window.add(&window_box);
        window_box.add(&menu_box);
        menu_box.add(&file_chooser_button);
        menu_box.pack_end(&read_file_button, false, true, 0);
        menu_box.pack_end(&read_only_button, false, true, 0);
        menu_box.pack_end(&write_file_button, false, true, 0);
        window_box.pack_end(&overlay, true, true, 0);
        overlay.add(&map_window);
        map_window.add(&map_image);
        for point in &location_buttons {
            overlay.add_overlay(point);
        }

        // Connect signals
        connect!(relm, read_file_button, connect_clicked(_), Msg::ReadFile);
        connect!(relm, write_file_button, connect_clicked(_), Msg::WriteFile);
        map_window.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        connect!(relm, map_window, connect_button_press_event(_, event), return (Some(Msg::HandleClick(event.clone())), Inhibit(false)));
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

        // Special connection for overlay buttons
        let ref_model = Rc::new(RefCell::new(model));
        let closure_reference_model = Rc::clone(&ref_model);
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
            let model = closure_reference_model.borrow();
            let point = model.points.get(point_index.unwrap()).expect("MapPoint for child doesn't exist");
            println!("Expected position of button: {:?}", point);

            // Get scroll info
            let kid = &parent.get_children()[0];
            let window = kid.downcast_ref::<gtk::ScrolledWindow>().expect("Overlay doesn't have ScrolledWindow as base child");
            let h_scroll = window.get_hadjustment().unwrap().get_value();
            let v_scroll = window.get_vadjustment().unwrap().get_value();

            Some(gtk::Rectangle {
                x: (point.x - h_scroll - BUTTON_SIZE / 2.0) as i32,
                y: (point.y - v_scroll - BUTTON_SIZE / 2.0) as i32,
                width: BUTTON_SIZE as i32,
                height: BUTTON_SIZE as i32
            })
        });

        // Do it
        window.show_all();

        Self {
            model: ref_model,
            widgets: Widgets {
                window,
                window_box,
                menu_box,
                file_chooser_button,
                read_only_button,
                read_file_button,
                write_file_button,
                overlay,
                map_window,
                map_image,
                location_buttons,
                location_popups,
            },
        }
    }
}

fn main() {
    RegionWindow::run(()).unwrap();
}
