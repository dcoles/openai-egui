//! Simple OpenAI GUI as a demonstration of egui.

// Hide console on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod openai;
mod ui;

use std::collections::BTreeMap;
use std::fs;

use openai::completions;
use poll_promise::Promise;
use serde_json::json;

const FONT_SIZE: f32 = 14.0;

struct OpenAIApp {
    text: String,
    ui_state: UIState,
    promise: Option<Promise<Result<completions::Response, String>>>,
    api_token: String,
}

impl OpenAIApp {
    fn new(_cc: &eframe::CreationContext<'_>, api_token: String) -> Self {
        Self {
            text: String::new(),
            ui_state: UIState::Idle,
            promise: None,
            api_token,
        }
    }
}

/// Current UI state.
#[derive(Debug, Clone, PartialEq, Eq)]
enum UIState {
    Idle,
    Busy,
    Error(String)
}

impl eframe::App for OpenAIApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut select_range = None;

        // Handle response
        if let Some(promise) = &self.promise {
            match promise.ready() {
                None => self.ui_state = UIState::Busy,
                Some(result) => {
                    match result {
                        Err(err) => self.ui_state = UIState::Error(err.clone()),
                        Ok(r) => {
                            match r {
                                completions::Response::Error(error) => {
                                    self.ui_state = UIState::Error(format!("ERROR: {}", error.error.r#type));
                                },
                                completions::Response::Success(value) => {
                                    let text = &value.choices[0].text;
                                    select_range = Some((self.text.chars().count(), text.chars().count()));

                                    self.text.push_str(text);
                                    self.ui_state = UIState::Idle;
                                },
                            }
                        },
                    }

                    self.promise = None;
                }
            }
        }

        if ctx.wants_keyboard_input() {
            let mut input = ctx.input_mut();
            if input.consume_key(egui::Modifiers::CTRL, egui::Key::Enter) {
                self.promise = Some(http_request(ctx, self.text.clone(), &self.api_token));
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Send").clicked() {
                    self.promise = Some(http_request(ctx, self.text.clone(), &self.api_token));
                }

                if let UIState::Error(error) = &self.ui_state {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }

                if matches!(self.ui_state, UIState::Busy) {
                    ui.spinner();
                }

                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().always_show_scroll(true).show(ui, |ui| {
                let mut text_edit = ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.text).font(egui::FontId::proportional(FONT_SIZE)));


                // Select range
                if let Some((start, count)) = select_range {
                    text_edit.mark_changed();

                    if let Some(mut state) = egui::TextEdit::load_state(ctx, text_edit.id) {
                        let range = egui::text_edit::CCursorRange::two(
                            egui::text::CCursor::new(start),
                            egui::text::CCursor::new(start + count)
                        );
                        state.set_ccursor_range(Some(range));
                        state.store(ctx, text_edit.id);

                        // Set focus back to text edit
                        ui.ctx().memory().request_focus(text_edit.id);
                    }

                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                }
            });
        });

    }
}

fn main() {
    // Requires a file named `openai.token` in the current working directory.
    let api_token = match fs::read_to_string("openai.token") {
        Err(_err) => {
            ui::alert("OpenAI token was not found.\nPlease add it to a file named `openai.token` in the current directory.\n\nThis app will now exit.");
            return;
        },
        Ok(s) => s.trim().to_string(),
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "OpenAI egui",
        native_options,
        Box::new(|cc| Box::new(OpenAIApp::new(cc, api_token))),
    );
}

/// Make a new HTTP request.
fn http_request(ctx: &egui::Context, prompt: String, api_token: &str) -> Promise<Result<completions::Response, String>> {
    let (sender, promise) = Promise::new();

    let headers = BTreeMap::from([
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Authorization".to_string(), format!("Bearer {}", api_token)),
    ]);

    let body = json!({
        "model": "text-davinci-003",
        "prompt": prompt,
        "temperature": 0.9,
        "max_tokens": 512,
        "top_p": 1,
        "frequency_penalty": 0.5,
        "presence_penalty": 0.25,
    });

    let body = match serde_json::to_string(&body) {
        Err(err) => {
            sender.send(Err(format!("ERROR: {}", err)));
            return promise;
        }
        Ok(json) => json,
    };
    eprintln!("Request: {}", body);

    let request = ehttp::Request {
        method: "POST".to_string(),
        url: "https://api.openai.com/v1/completions".to_string(),
        body: body.into_bytes(),
        headers,
    };

    let ctx = ctx.clone();

    ehttp::fetch(request, move |result| {
        let result = match result {
            Err(err) => {
                sender.send(Err(err));
                return;
            },
            Ok(res) => res,
        };

        eprintln!("Response: {}", result.text().unwrap_or("<unable to decode>"));

        let response: completions::Response = match serde_json::from_slice(&result.bytes) {
            Err(err) => {
                sender.send(Err(format!("ERROR: {}", err)));
                return;
            },
            Ok(r) => r,
        };

        sender.send(Ok(response));
        ctx.request_repaint();
    });

    promise
}
