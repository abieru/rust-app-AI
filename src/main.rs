use eframe::egui;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Clone)]
struct JokeResponse {
    id: String,
    joke: String,
    status: i32,
}

#[derive(Deserialize, Clone)]
struct JokeSearchResult {
    id: String,
    joke: String,
}

#[derive(Deserialize, Clone)]
struct SearchResponse {
    results: Vec<JokeSearchResult>,
    status: i32,
}

#[derive(Clone)]
enum FetchMode {
    Random,
    Search(String),
}

#[derive(Clone, Default)]
struct AppState {
    joke: String,
    loading: bool,
    error: Option<String>,
    search_term: String,
}

struct DadJokeApp {
    state: Arc<Mutex<AppState>>,
}

impl DadJokeApp {
    fn new(ctx: egui::Context) -> Self {
        let app = Self {
            state: Arc::new(Mutex::new(AppState {
                loading: true,
                ..Default::default()
            })),
        };
        app.fetch_joke(ctx, FetchMode::Random);
        app
    }

    fn fetch_joke(&self, ctx: egui::Context, mode: FetchMode) {
        let state = self.state.clone();

        {
            let mut s = state.lock().unwrap();
            s.loading = true;
            s.error = None;
        }

        std::thread::spawn(move || {
            let result: Result<String, String> = match &mode {
                FetchMode::Random => {
                    let res = ureq::get("https://icanhazdadjoke.com/")
                        .set("Accept", "application/json")
                        .set("User-Agent", "DadJokeRustApp (github.com/abieru)")
                        .call();

                    match res {
                        Ok(resp) => {
                            let text = resp.into_string().unwrap();
                            match serde_json::from_str::<JokeResponse>(&text) {
                                Ok(joke) => Ok(joke.joke),
                                Err(e) => Err(format!("Erro ao parsear: {}", e)),
                            }
                        }
                        Err(e) => Err(format!("Erro na rede: {}", e)),
                    }
                }
                FetchMode::Search(term) => {
                    let encoded = urlencoding::encode(term);
                    let url = format!("https://icanhazdadjoke.com/search?term={}", encoded);
                    let res = ureq::get(&url)
                        .set("Accept", "application/json")
                        .set("User-Agent", "DadJokeRustApp (github.com/abieru)")
                        .call();

                    match res {
                        Ok(resp) => {
                            let text = resp.into_string().unwrap();
                            match serde_json::from_str::<SearchResponse>(&text) {
                                Ok(search) => {
                                    if search.results.is_empty() {
                                        Err("Nenhuma piada encontrada".to_string())
                                    } else {
                                        Ok(search.results[0].joke.clone())
                                    }
                                }
                                Err(e) => Err(format!("Erro ao parsear: {}", e)),
                            }
                        }
                        Err(e) => Err(format!("Erro na rede: {}", e)),
                    }
                }
            };

            let mut s = state.lock().unwrap();
            match result {
                Ok(joke) => {
                    s.joke = joke;
                    s.loading = false;
                }
                Err(e) => {
                    s.error = Some(e);
                    s.loading = false;
                }
            }
            ctx.request_repaint();
        });
    }
}

impl eframe::App for DadJokeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .fill(egui::Color32::from_rgb(30, 30, 46)),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);

                    ui.heading(
                        egui::RichText::new("Dad Joke Generator")
                            .size(28.0)
                            .color(egui::Color32::from_rgb(137, 180, 250)),
                    );
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("Piadas ruins, direto da internet")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(127, 127, 160)),
                    );
                    ui.add_space(20.0);

                    let state = self.state.lock().unwrap().clone();

                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(49, 50, 68))
                        .corner_radius(12.0)
                        .inner_margin(24.0)
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    if state.loading {
                                        ui.spinner();
                                        ui.add_space(8.0);
                                        ui.label(
                                            egui::RichText::new("Buscando piada...")
                                                .color(egui::Color32::from_rgb(127, 127, 160)),
                                        );
                                    } else if let Some(err) = &state.error {
                                        ui.colored_label(egui::Color32::RED, err);
                                    } else if !state.joke.is_empty() {
                                        ui.add_space(10.0);
                                        ui.label(
                                            egui::RichText::new(&state.joke)
                                                .size(18.0)
                                                .color(egui::Color32::WHITE),
                                        );
                                    }
                                    ui.add_space(10.0);
                                },
                            );
                        });

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        let resp = ui.add(
                            egui::TextEdit::singleline(
                                &mut self.state.lock().unwrap().search_term,
                            )
                            .desired_width(250.0)
                            .hint_text("Buscar por tema (ex: dog, food, work)..."),
                        );

                        let enter_pressed =
                            resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                        let button_enabled = !state.loading;
                        ui.add_enabled_ui(button_enabled, |ui| {
                            if ui.button("Buscar").clicked() || enter_pressed {
                                let term = self.state.lock().unwrap().search_term.clone();
                                if !term.is_empty() {
                                    self.fetch_joke(ctx.clone(), FetchMode::Search(term));
                                }
                            }
                        });
                    });

                    ui.add_space(15.0);

                    let button_enabled = !state.loading;
                    ui.add_enabled_ui(button_enabled, |ui| {
                        if ui
                            .add_sized(
                                [220.0, 45.0],
                                egui::Button::new(egui::RichText::new("Nova Piada!").size(16.0)),
                            )
                            .clicked()
                        {
                            self.state.lock().unwrap().search_term.clear();
                            self.fetch_joke(ctx.clone(), FetchMode::Random);
                        }
                    });
                });
            });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Dad Joke Generator",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([600.0, 500.0])
                .with_resizable(false),
            ..Default::default()
        },
        Box::new(|ctx| Ok(Box::new(DadJokeApp::new(ctx.egui_ctx.clone())))),
    )
}
