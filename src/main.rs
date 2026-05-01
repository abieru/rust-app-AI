use eframe::egui;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Clone)]
struct DogResponse {
    message: String,
    status: String,
}

#[derive(Clone, Default)]
struct AppState {
    image_url: String,
    loading: bool,
    error: Option<String>,
}

struct DogApp {
    state: Arc<Mutex<AppState>>,
}

impl DogApp {
    fn new(ctx: egui::Context) -> Self {
        egui_extras::install_image_loaders(&ctx);

        let app = Self {
            state: Arc::new(Mutex::new(AppState {
                loading: true,
                ..Default::default()
            })),
        };
        app.fetch_new_dog(ctx);
        app
    }

    fn fetch_new_dog(&self, ctx: egui::Context) {
        let state = self.state.clone();

        {
            let mut s = state.lock().unwrap();
            s.loading = true;
            s.error = None;
        }

        std::thread::spawn(move || {
            let result = ureq::get("https://dog.ceo/api/breeds/image/random")
                .call()
                .map(|res| res.into_string().unwrap())
                .map(|text| serde_json::from_str::<DogResponse>(&text));

            let mut s = state.lock().unwrap();
            match result {
                Ok(Ok(r)) => {
                    s.image_url = r.message;
                    s.loading = false;
                }
                Ok(Err(e)) => {
                    s.error = Some(format!("Erro ao parsear: {}", e));
                    s.loading = false;
                }
                Err(e) => {
                    s.error = Some(format!("Erro na rede: {}", e));
                    s.loading = false;
                }
            }
            ctx.request_repaint();
        });
    }
}

impl eframe::App for DogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("Dog Viewer");
                ui.add_space(10.0);

                let state = self.state.lock().unwrap().clone();

                ui.group(|ui| {
                    ui.set_min_width(420.0);
                    ui.set_min_height(320.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        if state.loading {
                            ui.spinner();
                            ui.label("Carregando doggo...");
                        } else if let Some(err) = &state.error {
                            ui.colored_label(egui::Color32::RED, err);
                        } else if !state.image_url.is_empty() {
                            ui.add(
                                egui::Image::new(&state.image_url)
                                    .max_width(400.0)
                                    .corner_radius(10.0),
                            );
                        }

                        ui.add_space(10.0);
                    });
                });

                ui.add_space(15.0);

                let button_enabled = !state.loading;
                ui.add_enabled_ui(button_enabled, |ui| {
                    if ui
                        .add_sized([200.0, 40.0], egui::Button::new("Outro Dog!"))
                        .clicked()
                    {
                        self.fetch_new_dog(ctx.clone());
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Dog Viewer",
        eframe::NativeOptions::default(),
        Box::new(|ctx| Ok(Box::new(DogApp::new(ctx.egui_ctx.clone())))),
    )
}
