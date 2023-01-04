use eframe::{egui, Theme};
use egui_extras::{Column, TableBuilder};
use rmule::Engine;
use tracing::info;

use crate::widgets::toolbar_button::ToolbarButton;

pub fn start_ui(engine: Engine) {
    let options = eframe::NativeOptions {
        //initial_window_size: Some(vec2(320.0, 240.0)),
        default_theme: Theme::Light,
        ..Default::default()
    };

    let h = engine.configuration_manager_handle();
    let receiver = h.subscribe_to_events();
    //while receiver.recv().await {}

    eframe::run_native(
        "rMule",
        options,
        Box::new(|_cc| Box::new(TheApp::new(engine))),
    )
}

enum CurrentTab {
    Networks,
    // Searches,
    // Downloads,
    // Log,
}

struct TheApp {
    engine: Engine,
    current_tab: CurrentTab,
    servers: Vec<rmule::configuration::Server>,
}

impl TheApp {
    fn new(engine: Engine) -> Self {
        Self {
            engine,
            current_tab: CurrentTab::Networks,
            servers: Vec::new(),
        }
    }

    /// Construct the toolbar at the top of the window.
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

    /// Construct the status bar at the bottom of the window.
    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            ui.horizontal(|ui| ui.label("STATUS BAR GOES HERE"))
        });
    }

    /// Construct the Networks tab.
    fn networks_tab(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            TableBuilder::new(ui)
                .column(Column::initial(125.0).resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Server Name");
                    });
                    header.col(|ui| {
                        ui.heading("Address");
                    });
                    header.col(|ui| {
                        ui.heading("Port");
                    });
                })
                .body(|mut body| {
                    for server in &self.servers {
                        body.row(30.0, |mut row| {
                            let name = match &server.name {
                                Some(s) => s,
                                None => "",
                            };

                            row.col(|ui| {
                                ui.label(name);
                            });

                            row.col(|ui| {
                                ui.label(server.ip_addr.to_string());
                            });

                            row.col(|ui| {
                                ui.label(server.port.to_string());
                            });
                        });
                    }
                });
        });
    }
}

impl eframe::App for TheApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.toolbar(ctx);

        match &self.current_tab {
            CurrentTab::Networks => self.networks_tab(ctx),
        }

        self.status_bar(ctx);

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
