use eframe::egui::Vec2;
use super::buttons::{Button, Open, Theme, JS};

#[derive(Default)]
pub struct Menu {
    pub title: String,
    pub enabled: bool,
    //pub align: Align2, 
    pub offset: Vec2,
    pub width: f32,
    pub buttons: Vec<Box<dyn Button>>
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            title: "default".to_string(),
            enabled: false,
            //align: Align2::LEFT_TOP,
            offset: Vec2::new(0.0, 0.0),
            width: 100.0,
            buttons: Vec::new()
        }
    }

    pub fn set_file_menu(&mut self) {
        self.title = "file".to_string();

        self.buttons = Vec::new();
        self.buttons.push(Box::new(Open::new()));

        self.offset = [0.0, 0.0].into();
        self.enabled = true;
    }

    pub fn set_settings_menu(&mut self) {
        self.title = "settings".to_string();

        self.buttons = Vec::new();
        self.buttons.push(Box::new(Theme::new()));

        self.offset = [100.0, 0.0].into();
        self.enabled = true;
    }

    pub fn set_convert_menu(&mut self) {
        self.title = "convert".to_string();

        self.buttons = Vec::new();
        self.buttons.push(Box::new(JS::new()));

        self.offset = [200.0, 0.0].into();
        self.enabled = true;
    }
}