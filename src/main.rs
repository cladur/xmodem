mod common;
mod receiver;
mod transmitter;

use eframe::egui::{self, CentralPanel, Layout};
use eframe::epi::App;
use eframe::run_native;
use receiver::receive;
use std::thread;
use transmitter::transmit;

use std::path::PathBuf;

#[derive(PartialEq, Clone, Copy)]
pub enum ReceiverMode {
    Normal,
    CRC,
}

#[derive(PartialEq)]
enum Mode {
    None,
    Transmitter,
    Receiver,
}

struct AppState {
    mode: Mode,
    input_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    in_port: String,
    out_port: String,
    receiver_mode: ReceiverMode,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            mode: Mode::None,
            input_file: None,
            output_file: None,
            in_port: String::from("COM1"),
            out_port: String::from("COM2"),
            receiver_mode: ReceiverMode::Normal,
        }
    }
}

impl App for AppState {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &eframe::epi::Frame) {
        ctx.set_pixels_per_point(2.);
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("XMODEM");
                ui.add_space(20.);
                match self.mode {
                    Mode::None => {
                        ui.heading("Mode");
                        ui.add_space(5.0);
                        if ui.button("Send").clicked() {
                            self.mode = Mode::Transmitter;
                        }
                        ui.add_space(5.);
                        if ui.button("Receive").clicked() {
                            self.mode = Mode::Receiver;
                        }
                    }
                    Mode::Transmitter => {
                        // ### ################ ###
                        // ### TRANSMITTER MODE ###
                        // ### ################ ###

                        ui.heading("Transmitter Mode");
                        ui.add_space(10.);

                        ui.label("Input File");

                        ui.add_space(5.);

                        // FILE PICKER
                        if ui.button("Pick File").clicked() {
                            self.input_file = rfd::FileDialog::new().pick_file();
                        }
                        if let Some(selected_file) = &self.input_file {
                            ui.label(selected_file.to_string_lossy().to_string());
                        } else {
                            ui.add_space(20.0);
                        }

                        // PORT NUMBER
                        ui.add_space(5.);
                        ui.label("Port");
                        ui.text_edit_singleline(&mut self.in_port);
                        // START TRANSMISSION
                        if ui.button("Send").clicked() {
                            println!("Transmitting data...");
                            println!("Opening port {}...", self.in_port);
                            let mut port = serialport::new(&self.in_port, 115_200).open().unwrap();
                            if let Some(path) = &self.input_file {
                                let data = common::file_to_u8(path.to_str().unwrap());
                                println!("Transmitting {} bytes...", data.len());
                                thread::spawn(move || {
                                    transmit(&mut port, &data);
                                });
                            }
                        }

                        ui.add_space(30.);
                        if ui.button("Back").clicked() {
                            self.mode = Mode::None;
                        }
                    }
                    Mode::Receiver => {
                        // ### ############# ###
                        // ### RECEIVER MODE ###
                        // ### ############# ###

                        ui.heading("Receiver Mode");
                        ui.add_space(10.);

                        ui.label("Output File");

                        ui.add_space(5.);

                        // FILE PICKER
                        if ui.button("Pick File").clicked() {
                            self.output_file = rfd::FileDialog::new().save_file();
                        }
                        if let Some(selected_file) = &self.output_file {
                            ui.label(selected_file.to_string_lossy().to_string());
                        } else {
                            ui.add_space(20.0);
                        }

                        // PORT NUMBER
                        ui.add_space(5.);
                        ui.label("Port");
                        ui.text_edit_singleline(&mut self.out_port);

                        ui.label("Checksum Algorithm");
                        ui.radio_value(&mut self.receiver_mode, ReceiverMode::Normal, "Normal");
                        ui.radio_value(&mut self.receiver_mode, ReceiverMode::CRC, "CRC");

                        if ui.button("Receive").clicked() {
                            let mut port = serialport::new(&self.in_port, 115_200).open().unwrap();
                            let data = receive(&mut port, self.receiver_mode);
                            if let Some(path) = &self.output_file {
                                common::u8_to_file(path.to_str().unwrap(), &data);
                            }
                        }

                        ui.add_space(30.);
                        if ui.button("Back").clicked() {
                            self.mode = Mode::None;
                        }
                    }
                }
            });
        });
    }

    fn name(&self) -> &str {
        "Xmodem"
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(350.0, 675.0)),
        ..eframe::NativeOptions::default()
    };
    run_native(Box::new(AppState::new()), native_options);
}
