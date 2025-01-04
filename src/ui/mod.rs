use eframe::egui::{self, Sense, UiBuilder};

mod buttons;
mod sub_menus;
#[derive(Default)]
pub struct App {
    pub sub_menu: sub_menus::Menu,
}

impl App {
    pub fn new() -> Self {
        App {
            sub_menu: sub_menus::Menu::new()
        }
    }
}

impl eframe::App for App {
    fn update (&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {

        let mut in_sub_menu: bool = false;

        egui::Window::new(self.sub_menu.title.clone())
        .open(&mut self.sub_menu.enabled.clone())
        .anchor(egui::Align2::LEFT_TOP,self.sub_menu.offset.clone())
        .title_bar(false)
        .min_width(self.sub_menu.width.clone())
        .max_width(self.sub_menu.width.clone())
        .resizable([false, false])
        .show(ctx, |ui| {
            if ctx.is_pointer_over_area() { in_sub_menu = true; }
            for button in &self.sub_menu.buttons {
                if ui.add_enabled(button.settings().enabled, egui::Button::new(button.settings().name)).clicked() {
                    match button.action() {
                        Ok(_) => self.sub_menu.enabled = false,
                        Err(e) => eprintln!("{e}")
                    }
                }
            }
        });

        egui::TopBottomPanel::top("top_settings_bar")
        .show(ctx, |ui| {
            ui.columns(3, |columns| {
                columns[0]
                .vertical_centered(|ui| {
                    if ui.add( egui::Button::new("File")).clicked() {
                        self.sub_menu.set_file_menu();
                    }
                });
                columns[1].vertical_centered(|ui| {
                    if ui.add( egui::Button::new("Settings")).clicked() {
                        self.sub_menu.set_settings_menu();
                    }
                });
                columns[2].vertical_centered(|ui| {
                    if ui.add( egui::Button::new("Convert")).clicked() {
                        self.sub_menu.set_convert_menu();
                    }
                });
            }); 
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test Title That is now really really long...");
        });

        

        
    }
}