extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate serde;

use relm::{Relm, Update, Widget};
use gtk::{
    Orientation::Vertical,
    Button,
    ButtonExt,
    ContainerExt,
    Label,
    LabelExt,
    WidgetExt,
    Window,
    Inhibit,
    WindowType
};

mod region;
use self::region::{MapPoint, RegionData, RegionModel};

// Messages are sent to `Widget::update` to indicate that an event happened. The model can be
// updated when an event is received.
#[derive(Msg)]
enum Msg {
    AddNewPoint,
    DeletePoint((f32, f32)),
    UpdateDescription,
    WriteFile,
    ReadFile,
    Quit,
}

// This is just a container to house widgets for the parent widget, which also contains a Model
#[derive(Clone)]
struct Widgets {
    counter_label: Label,
    minus_button: Button,
    plus_button: Button,
    window: Window,
}

// This is that parent widget. It implements Update and Widget
struct RegionWindow {
    model: RegionModel,
    widgets: Widgets,
}

impl Update for RegionWindow {
    // Specify the model used for this widget.
    type Model = RegionModel;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    // Return the initial model.
    fn model(_: &Relm<Self>, _: ()) -> RegionModel {
        RegionModel {
            counter: 0,
        }
    }

    // The model may be updated when a message is received.
    // Widgets may also be updated in this function.
    fn update(&mut self, event: Msg) {
        let label = &self.widgets.counter_label;
        match event {
            Msg::Decrement => {
                self.model.counter -= 1;
                label.set_text(&self.model.counter.to_string());
            },
            Msg::Increment => {
                self.model.counter += 1;
                label.set_text(&self.model.counter.to_string());
            },
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for RegionWindow {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    // Create the widgets.
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        // GTK+ widgets are used normally within a `Widget`.
        let vbox = gtk::Box::new(Vertical, 0);
        let plus_button = Button::new_with_label("+");
        let counter_label = Label::new("0");
        let minus_button = Button::new_with_label("-");
        vbox.add(&plus_button);
        vbox.add(&counter_label);
        vbox.add(&minus_button);

        let window = Window::new(WindowType::Toplevel);
        window.add(&vbox);
        window.show_all();

        connect!(relm, plus_button, connect_clicked(_), Msg::Increment);
        connect!(relm, minus_button, connect_clicked(_), Msg::Decrement);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

        RegionWindow {
            model,
            widgets: Widgets {
                counter_label,
                minus_button,
                plus_button,
                window,
            }
        }
    }
}

fn main() {
    RegionWindow::run(()).unwrap();
}
