#![windows_subsystem = "windows"]

use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use eframe::{
    egui::{
        Align, Button, CentralPanel, Color32, Context, Layout, RichText, TextEdit, ViewportBuilder,
        Window,
    },
    run_native, App, Frame, NativeOptions, Result,
};
use rfd::FileDialog;
use strum::IntoDiscriminant;
use strum_macros::EnumDiscriminants;
use winio_loader::{Error, WinIoLoader};

fn main() -> Result<()> {
    Command::new("cmd")
        .arg("/C")
        .arg("sc stop WinIo && sc delete WinIo && del winio.sys")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([425.0, 350.0]),
        centered: true,
        ..Default::default()
    };

    run_native(
        "WinIo Patcher",
        options,
        Box::new(|_| Ok(Box::<WinIoPatcher>::default())),
    )
}

struct WinIoPatcher {
    winio_loader: WinIoLoader,
    driver_path: String,
    driver_name: String,
    popup: Option<Popup>,
    initialization_time: Duration,
}

impl WinIoPatcher {
    fn set_dse(&mut self, enabled: bool) {
        if let Err(e) = self.winio_loader.set_dse(enabled) {
            self.popup = Some(Popup::Error(e))
        }
    }
}

impl Default for WinIoPatcher {
    fn default() -> Self {
        let start = Instant::now();

        let winio_loader = WinIoLoader::new(true).unwrap();
        let driver_path = String::new();
        let driver_name = String::new();
        let popup = None;
        let initialization_time = start.elapsed();

        Self {
            winio_loader,
            driver_name,
            driver_path,
            popup,
            initialization_time,
        }
    }
}

impl App for WinIoPatcher {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("DSE Status: ");

                    if ui
                        .add(Button::new(
                            RichText::new(if self.winio_loader.dse {
                                "Enabled"
                            } else {
                                "Disabled"
                            })
                            .heading(),
                        ))
                        .clicked()
                    {
                        self.set_dse(!self.winio_loader.dse);
                    }
                });

                ui.collapsing(RichText::new("Driver").heading(), |ui| {
                    ui.horizontal(|ui| {
                        let path_heading = ui.heading("Path: ");
                        ui.add(TextEdit::singleline(&mut self.driver_path).desired_width(250.0))
                            .labelled_by(path_heading.id);
                        if ui.button("📄").clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Driver Files", &["sys"])
                                .pick_file()
                            {
                                self.driver_name =
                                    path.file_stem().unwrap().to_string_lossy().to_string();
                                self.driver_path = path.display().to_string();
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        let name_heading = ui.heading("Name: ");
                        ui.add(TextEdit::singleline(&mut self.driver_name).desired_width(100.0))
                            .labelled_by(name_heading.id);
                    });

                    if ui
                        .add(Button::new(RichText::new("Load").heading()))
                        .clicked()
                    {
                        if self.driver_path.is_empty() || self.driver_name.is_empty() {
                            self.popup = Some(Popup::Warning(String::from(
                                "Please fill in both the driver path and name.",
                            )));
                        } else {
                            self.set_dse(false);

                            match WinIoLoader::create_driver(&self.driver_name, &self.driver_path) {
                                Ok(()) => {
                                    self.popup = Some(Popup::Success(String::from(
                                        "Successfully loaded driver!",
                                    )));
                                }
                                Err(e) => self.popup = Some(Popup::Error(e)),
                            }

                            self.set_dse(true);
                        }
                    }
                });

                if let Some(popup) = self.popup.clone() {
                    Window::new(format!("{:?}", popup.discriminant()))
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.label(match popup {
                                Popup::Success(s) => RichText::new(s).color(Color32::GREEN),
                                Popup::Warning(s) => RichText::new(s).color(Color32::YELLOW),
                                Popup::Error(e) => RichText::new(e.to_string()).color(Color32::RED),
                            });
                            if ui.button("OK").clicked() {
                                self.popup = None;
                            }
                        });
                }

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.label(format!(
                        "Initialization Time: {:?}",
                        self.initialization_time
                    ));

                    ui.label(format!(
                        "SeValidateImageHeader: {:X}",
                        self.winio_loader.se_validate_image_header
                    ));
                    ui.label(format!(
                        "SeValidateImageData: {:X}",
                        self.winio_loader.se_validate_image_data
                    ));
                });
            });
        });
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
enum Popup {
    Success(String),
    Warning(String),
    Error(Error),
}
