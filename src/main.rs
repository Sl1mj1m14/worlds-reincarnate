use eframe::egui;

use mc_classic as classic;
use mc_classic_js as js;

mod ui;
mod convert;

const APP_NAME: &str = "Worlds Reincarnate";
const BASE_WIDTH: f32 = 400.0;
const BASE_HEIGHT: f32 = 400.0;

fn main () -> eframe::Result {

    let app = ui::App {};

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([BASE_WIDTH, BASE_HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
         options, 
         Box::new(|_|Ok(Box::new(app)))
    )
}



