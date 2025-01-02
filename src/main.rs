use eframe::egui;

fn main () -> eframe::Result {

    let app = App {};

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([400.0,400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "temp",
         options, 
         Box::new(|_|Ok(Box::new(app)))
    )

}

#[derive(Default)]
struct App {}

impl eframe::App for App {
    fn update (&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {

    }
}

