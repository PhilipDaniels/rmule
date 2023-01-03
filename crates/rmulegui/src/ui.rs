use eframe::{egui, Theme};
use rmule::Engine;
use tracing::info;

use crate::widgets::toolbar_button::ToolbarButton;

pub fn start_ui(engine: Engine) {
    let options = eframe::NativeOptions {
        //initial_window_size: Some(vec2(320.0, 240.0)),
        default_theme: Theme::Light,
        ..Default::default()
    };

    eframe::run_native(
        "rMule",
        options,
        Box::new(|_cc| Box::new(TheApp::new(engine))),
    )
}

struct TheApp {
    engine: Engine,
}

impl TheApp {
    fn new(engine: Engine) -> Self {
        Self { engine }
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(ToolbarButton::new("Networks")).clicked() {
                    info!("Networks");
                }
                if ui.add(ToolbarButton::new("Searches")).clicked() {
                    info!("Searches");
                }
                if ui.add(ToolbarButton::new("Downloads")).clicked() {
                    info!("Downloads");
                }
                if ui.add(ToolbarButton::new("Log")).clicked() {
                    info!("Log");
                }
            });
        });
    }
}

impl eframe::App for TheApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.toolbar(ctx);

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.heading("My egui Application");
        //     ui.horizontal(|ui| {
        //         let name_label = ui.label("Your name: ");
        //         ui.text_edit_singleline(&mut self.name)
        //             .labelled_by(name_label.id);
        //     });
        //     ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
        //     if ui.button("Click each year").clicked() {
        //         self.age += 1;
        //     }
        //     ui.label(format!("Hello '{}', age {}", self.name, self.age));
        // });
    }
}
