use eframe::egui;

use thiserror::Error;

use mc_classic;
use mc_classic_js;

mod ui;
mod convert;
mod read;
mod write;

const APP_NAME: &str = "Worlds Reincarnate";
const BASE_WIDTH: f32 = 600.0;
const BASE_HEIGHT: f32 = 400.0;

fn main () -> eframe::Result {

    let app: ui::App = ui::App::new();

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



