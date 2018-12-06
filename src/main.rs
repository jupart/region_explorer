#[macro_use]
extern crate serde;

use gio::prelude::*;

mod region;
use self::region::RegionData;

fn main() {
    let region_data = RegionData::from_path("./resources/kellua_saari.ron");
    let application = gtk::Application::new("com.github.basic",
                                            gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    application.connect_startup(move |app| {
        region_data.build_ui(app);
    });
    application.connect_activate(|_| {});
    application.run(&vec![]);
}
