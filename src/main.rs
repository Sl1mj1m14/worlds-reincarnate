use eframe::egui;
use rfd::FileDialog;

use thiserror::Error;

use mc_classic;
use mc_classic_js;

mod log;
mod ui;
mod convert;
mod read;
mod write;

pub const DEBUG_FLAG: bool = true;
pub const QUALIFIER: &str = "org";
pub const ORGANIZATION: &str = "Sl1mJ1m Inc";
pub const APPLICATION: &str = "Worlds Reincarnate";

const BASE_WIDTH: f32 = 600.0;
const BASE_HEIGHT: f32 = 400.0;

fn main () -> eframe::Result {

    //env_logger::init();

    let app: ui::App = ui::App::new();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([BASE_WIDTH, BASE_HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        APPLICATION,
         options, 
         Box::new(|_|Ok(Box::new(app)))
    )
}



