use std::fs::File;
use std::io::prelude::*;

// use gtk::prelude::*;

// const TEXT_BORDER_SIZE: i32 = 3;
// const TEXT_PADDING_SIZE: i32 = 3;
// const BUTTON_SIZE: i32 = 20;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapPoint {
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
pub struct RegionModel {
    pub name: String,
    pub image: String,
    pub description: String,
    pub points: Vec<MapPoint>,
}

impl RegionModel {
    pub fn from_path(path: &str) -> Self {
        let mut region_file = File::open(path).expect(&format!("Error opening {}", path));
        let mut ron_data = String::new();
        region_file.read_to_string(&mut ron_data).unwrap();
        ron::de::from_bytes(ron_data.as_bytes()).expect(&format!("{} doesn't match expected RegionData format", path))
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

