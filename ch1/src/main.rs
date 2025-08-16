use eframe::egui;
use std::error::Error;


fn main() -> Result<(), Box<dyn Error>> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "My egui app", 
        native_options, 
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))
    )?;

    Ok(())
}

#[derive(Default)]
struct MyEguiApp {}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}