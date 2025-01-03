use eframe::egui::{self, Sense, UiBuilder};
#[derive(Default)]
pub struct App {
    pub file_open: bool,
    pub settings_open: bool,
    pub convert_open: bool
}

impl App {
    pub fn new() -> Self {
        App {
            file_open: false,
            settings_open: false,
            convert_open: false
        }
    }
}

impl eframe::App for App {
    fn update (&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {

        let mut file_open = self.file_open;
        egui::Window::new("file_menu")
        .open(&mut file_open)
        .anchor(egui::Align2::LEFT_TOP,[0.0,0.0])
        .title_bar(false)
        .auto_sized()
        .show(ctx, |ui| {
            if !ctx.is_pointer_over_area() { self.file_open = false }
            if ui.add_enabled(false, egui::Button::new("Open")).clicked() {
                //Open New File Somehow...
            }
        });

        egui::TopBottomPanel::top("top_settings_bar")
        .show(ctx, |ui| {
            ui.columns(3, |columns| {
                columns[0]
                .vertical_centered(|ui| {
                    if ui.add( egui::Button::new("File")).clicked() {
                        self.file_open = true;
                    }
                });
                columns[1].vertical_centered(|ui| {
                    if ui.add( egui::Button::new("Settings")).clicked() {
                        self.settings_open = true;
                    }
                });
                columns[2].vertical_centered(|ui| {
                    if ui.add( egui::Button::new("Convert")).clicked() {
                        self.convert_open = true;
                    }
                });
            }); 
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test Title That is now really really long...");
        });

        

        
    }
}