use eframe::{egui, Theme};
use egui_extras::{Column, TableBuilder};
use rmule::configuration::ConfigurationEventReceiver;
use rmule::Engine;
use tracing::info;

use crate::widgets::toolbar_button::ToolbarButton;

pub fn show_main_window(engine: Engine) {
    let options = eframe::NativeOptions {
        //initial_window_size: Some(vec2(320.0, 240.0)),
        default_theme: Theme::Light,
        ..Default::default()
    };

    //engine.start().await;

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

/// TheApp represents the data structures necessary to support the UI.
/// It holds the Engine, which contains the "heart" of the program.
struct TheApp {
    engine: Engine,
    cfg_mgr_receiver: ConfigurationEventReceiver,
    current_tab: CurrentTab,
    servers: Vec<rmule::configuration::Server>,
}

impl TheApp {
    fn new(engine: Engine) -> Self {
        let cfg_mgr_receiver = engine.configuration_manager_handle().subscribe_to_events();

        Self {
            engine,
            cfg_mgr_receiver,
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
                    // TODO: This is just temporary.
                    self.engine
                        .configuration_manager_handle()
                        .send_command_blocking(rmule::configuration::ConfigurationCommand::Start)
                        .unwrap();
                    info!("Starting PRESSED");
                    // DOES NOT WORK:
                    // ctx.request_repaint_after(Duration::from_millis(100));
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

    fn receive_engine_events(&mut self) {
        if let Ok(evt) = self.cfg_mgr_receiver.try_recv() {
            use rmule::configuration::ConfigurationEvents::*;

            match evt {
                InitComplete => info!("Got InitComplete"),
                SettingsChange(_settings) => info!("Got settings"),
                AddressListChange(_addr_list) => info!("Got addr list"),
                TempDirectoryListChange(_temp_dir_list) => info!("Got temp dir list"),
                ServerListChange(server_list) => self.servers = server_list.into_iter().collect(),
            }
        }
    }
}

impl eframe::App for TheApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.receive_engine_events();

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
