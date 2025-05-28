#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use eframe::egui;
use egui::{
    Align, Align2, CentralPanel, Color32, ColorImage, Context, CursorIcon, DragValue, FontId,
    Frame, Layout, Pos2, Rect, ScrollArea, Sense, Shape, SidePanel, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Ui, UiBuilder, Vec2,
};
use egui_file_dialog::FileDialog;
use gamedata::v2_0_0::CONSTRUCT_CARD_BY_NAME;
use image::{GenericImageView, ImageFormat};
use models::v2_0_0::{PlayerTarget, Tier};
use simulator::{
    CardModification, CardTemplate, DispatchableEvent, PlayerTemplate, Simulation,
    SimulationResult, SimulationSummary, SimulationTemplate,
};
use tracing_subscriber::EnvFilter;

lazy_static::lazy_static! {
    pub static ref OPTIMAL_THREAD_COUNT: usize = num_cpus::get().max(1);
}

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    tracing::info!("launch");

    let blur_paths: Vec<PathBuf> = CONSTRUCT_CARD_BY_NAME
        .iter()
        .map(|(_, construct)| {
            let card = construct();
            PathBuf::from(format!("gamedata/src/v2_0_0/cards/images/{}.avif", card.id))
        })
        .collect();
    spawn_backdrop_thread(blur_paths);

    let options = eframe::NativeOptions::default();
    let app = App::new();
    eframe::run_native(
        &format!("Emporium v{}", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|_| Ok(Box::new(app))),
    )
}

fn spawn_backdrop_thread(paths: Vec<PathBuf>) {
    thread::spawn(move || {
        for original in paths {
            if let (Some(ext), Some(stem)) = (
                original.extension().and_then(|e| e.to_str()),
                original.file_stem().and_then(|s| s.to_str()),
            ) {
                let mut backdrop = original.clone();
                backdrop.set_file_name(format!("{stem}.backdrop.{ext}"));

                if backdrop.exists() {
                    continue;
                }
                if let Ok(img) = image::open(&original) {
                    let blurred = img.blur(30.0);
                    let _ = blurred.save(&backdrop);
                }
            }
        }
    });
}

#[derive(Clone)]
struct TexturedCard {
    pub card: models::v2_0_0::Card,
    pub texture: Option<TextureHandle>,
    pub blurred_texture: Option<TextureHandle>,
    pub texture_path: PathBuf,
}

impl From<models::v2_0_0::Card> for TexturedCard {
    fn from(value: models::v2_0_0::Card) -> Self {
        let texture_path = PathBuf::from(format!(
            "gamedata/src/v2_0_0/cards/images/{}.avif",
            &value.id
        ));
        Self {
            card: value,
            texture: None,
            blurred_texture: None,
            texture_path,
        }
    }
}

#[derive(Clone)]
struct CardOnBoard {
    template: TexturedCard,
    tier: Tier,
    position: u8,
    modifications: Vec<CardModification>,
}

impl Into<CardTemplate> for CardOnBoard {
    fn into(self) -> CardTemplate {
        CardTemplate {
            name: self.template.card.name.to_string(),
            tier: self.tier,
            modifications: self.modifications,
        }
    }
}

struct App {
    texture_rx: Receiver<(String, ColorImage, u8)>,
    texture_tx: Sender<(String, ColorImage, u8)>,
    cards_with_texture: Vec<TexturedCard>,
    player_board: Vec<CardOnBoard>,
    opponent_board: Vec<CardOnBoard>,
    sim_event_rx: Option<Receiver<DispatchableEvent>>,
    sim_result_rx: Option<Receiver<SimulationResult>>,
    sim_errors: Vec<String>,
    sim_warnings: Vec<String>,
    sim_logs: Vec<String>,
    sim_running: bool,
    sim_results: Vec<SimulationResult>,
    sim_iterations: usize,
    sim_completed: usize,
    sim_start: Option<Instant>,
    sim_elapsed: Duration,
    sim_load_error: Option<String>,
    player_health: u64,
    opponent_health: u64,
    toml_file_dialog: FileDialog,
    loading_ids: HashSet<String>,
}

impl App {
    fn new() -> Self {
        let (texture_tx, texture_rx) = std::sync::mpsc::channel();
        let mut cards_with_texture = Vec::with_capacity(CONSTRUCT_CARD_BY_NAME.len());

        for (_, construct) in CONSTRUCT_CARD_BY_NAME.iter() {
            cards_with_texture.push(construct().into());
        }

        cards_with_texture.sort_by_key(|c: &TexturedCard| c.card.name);
        Self {
            texture_rx,
            texture_tx,
            cards_with_texture,
            player_board: Vec::with_capacity(10),
            opponent_board: Vec::with_capacity(10),
            sim_event_rx: None,
            sim_result_rx: None,
            sim_running: false,
            sim_logs: Vec::new(),
            sim_warnings: Vec::new(),
            sim_errors: Vec::new(),
            sim_results: Vec::new(),
            sim_iterations: 1000,
            sim_completed: 0,
            sim_start: None,
            sim_elapsed: Duration::ZERO,
            sim_load_error: None,
            player_health: 300,
            opponent_health: 300,
            toml_file_dialog: FileDialog::new()
                .add_file_filter(
                    "TOML files",
                    Arc::new(|path| path.extension().unwrap_or_default() == "toml"),
                )
                .title("Select Simulation Template"),
            loading_ids: HashSet::with_capacity(CONSTRUCT_CARD_BY_NAME.len()),
        }
    }

    fn show_board(&mut self, ui: &mut Ui, board_owner: PlayerTarget) {
        let board = match board_owner {
            PlayerTarget::Player => &mut self.player_board,
            PlayerTarget::Opponent => &mut self.opponent_board,
        };

        ui.horizontal_wrapped(|ui| {
            let slot_w = 64.0;
            let slot_h = 96.0;
            let gap = ui.spacing().item_spacing.x;
            let mut pos = 0u8;

            while pos < 10 {
                if let Some(card_on_board) = board.iter_mut().find(|c| c.position == pos) {
                    let spaces = card_on_board.template.card.size.board_spaces() as f32;
                    let extra = (spaces - 1.0) * gap + (spaces - 1.0) * 2.0 + 2.0; //Border
                    let width = slot_w * spaces + extra;
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(width, slot_h), Sense::click());

                    let id = card_on_board.template.card.id;
                    if let Some(shared) =
                        self.cards_with_texture.iter_mut().find(|t| t.card.id == id)
                    {
                        if shared.texture.is_none()
                            && !self.loading_ids.contains(id)
                            && ui.is_rect_visible(rect)
                        {
                            self.loading_ids.insert(id.to_string());
                            let tx = self.texture_tx.clone();
                            let path = shared.texture_path.clone();
                            let size = shared.card.size.board_spaces() as u8;
                            thread::spawn(move || {
                                let bytes = std::fs::read(&path).unwrap();
                                let img =
                                    image::load_from_memory_with_format(&bytes, ImageFormat::Avif)
                                        .unwrap()
                                        .to_rgba8();
                                let (w, h) = img.dimensions();
                                let ci = ColorImage::from_rgba_unmultiplied(
                                    [w as usize, h as usize],
                                    &img.into_raw(),
                                );
                                tx.send((id.to_string(), ci, size)).unwrap();
                            });
                        }

                        if let Some(tex) = &shared.texture {
                            ui.painter().add(Shape::image(
                                tex.id(),
                                rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            ));
                            let border = match card_on_board.tier {
                                Tier::Bronze => Color32::from_rgb(205, 127, 50),
                                Tier::Silver => Color32::from_rgb(192, 192, 192),
                                Tier::Gold => Color32::from_rgb(255, 215, 0),
                                Tier::Diamond => Color32::from_rgb(0, 215, 215),
                                Tier::Legendary => Color32::from_rgb(148, 0, 211),
                            };
                            ui.painter().rect_stroke(
                                rect,
                                0.0,
                                Stroke::new(2.0, border),
                                StrokeKind::Middle,
                            );
                        }
                    }

                    if resp.clicked() {
                        let tiers = card_on_board.template.card.available_tiers();
                        if let Some(i) = tiers.iter().position(|t| *t == card_on_board.tier) {
                            card_on_board.tier = tiers[(i + 1) % tiers.len()];
                        }
                    }
                    let position_inc = card_on_board.template.card.size.board_spaces() as u8;

                    if resp.secondary_clicked() {
                        board.retain(|c| c.position != pos);
                    }

                    pos += position_inc;
                } else {
                    ui.push_id(pos, |ui| {
                        let frame = Frame::new()
                            .stroke(Stroke::new(1.0, ui.style().visuals.selection.stroke.color));
                        if let (_, Some(idx)) = ui.dnd_drop_zone::<usize, _>(frame, |ui| {
                            let (r, resp) = ui.allocate_exact_size(
                                Vec2::new(slot_w, slot_h),
                                Sense::click_and_drag(),
                            );
                            ui.painter().rect_stroke(
                                r,
                                0.0,
                                Stroke::new(1.0, Color32::LIGHT_GRAY),
                                StrokeKind::Middle,
                            );
                            resp
                        }) {
                            let template = self.cards_with_texture[*idx].clone();
                            let size = template.card.size.board_spaces() as u8;
                            let overlap = board.iter().any(|c| {
                                let start = c.position;
                                let end = start + c.template.card.size.board_spaces() as u8;
                                (pos < end) && (start < pos + size)
                            });
                            if !overlap && pos + size <= 10 {
                                board.push(CardOnBoard {
                                    tier: template.card.min_tier(),
                                    template,
                                    position: pos,
                                    modifications: vec![],
                                });
                            }
                        }
                    });
                    pos += 1;
                }
            }
        });
    }

    fn show_side_panel(&mut self, ui: &mut Ui) {
        ui.heading("Add card");
        ui.separator();

        let entry_height = 72.0;
        let spacing = 4.0;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (idx, temp) in self.cards_with_texture.iter().enumerate() {
                    let img_w = entry_height * (temp.card.size.board_spaces() as f32) / 2.0;
                    let row_id = ui.make_persistent_id(format!("card-row-{}", idx));

                    ui.dnd_drag_source(ui.id().with(idx), idx, |ui| {
                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), entry_height),
                            Sense::click_and_drag().union(Sense::hover()),
                        );

                        if temp.texture.is_none()
                            && !self.loading_ids.contains(&temp.card.id.to_string())
                            && ui.is_rect_visible(row_rect)
                        {
                            self.loading_ids.insert(temp.card.id.to_string());
                            let tx = self.texture_tx.clone();
                            let path = temp.texture_path.clone();
                            let id = temp.card.id;
                            let size = temp.card.size.board_spaces() as u8;
                            tracing::debug!(?path, "spawn load texture thread");
                            thread::spawn(move || {
                                match std::fs::read(&path) {
                                    Ok(bytes) => match image::load_from_memory_with_format(
                                        &bytes,
                                        ImageFormat::Avif,
                                    )
                                    .map(|i| i.to_rgba8())
                                    {
                                        Ok(img) => {
                                            let (w, h) = img.dimensions();
                                            let ci = ColorImage::from_rgba_unmultiplied(
                                                [w as usize, h as usize],
                                                &img.into_raw(),
                                            );
                                            tx.send((id.to_string(), ci, size)).unwrap();
                                        }
                                        Err(error) => {
                                            tracing::warn!(?error, ?path, "unable to load texture")
                                        }
                                    },
                                    Err(error) => {
                                        tracing::warn!(?error, ?path, "unable to read texture")
                                    }
                                };
                            });
                        }

                        if row_resp.hovered() {
                            egui::show_tooltip(ui.ctx(), ui.layer_id(), row_id, |ui| {
                                Frame::popup(&ui.style()).show(ui, |ui| {
                                    ui.label(temp.card.name);
                                    ui.separator();
                                    ui.horizontal_wrapped(|ui| {
                                        for tag in temp.card.tags.iter() {
                                            ui.label(format!("({tag:?})"));
                                        }
                                    });
                                    ui.separator();
                                    ui.label(format!(
                                        "Size: {} spaces",
                                        temp.card.size.board_spaces()
                                    ));
                                });
                            });
                        }

                        row_resp.on_hover_cursor(CursorIcon::PointingHand);

                        let mut row_ui = ui.new_child(
                            UiBuilder::new()
                                .max_rect(row_rect)
                                .layout(Layout::left_to_right(Align::Center))
                                .sense(Sense::click_and_drag().union(Sense::hover())),
                        );

                        let text_w = (row_rect.width() - img_w - spacing).max(0.0);
                        row_ui.allocate_ui_with_layout(
                            Vec2::new(text_w, entry_height),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label(temp.card.name);
                                ui.horizontal_wrapped(|ui| {
                                    for tag in temp.card.tags.iter() {
                                        ui.label(format!("({tag:?})"));
                                    }
                                });
                            },
                        );

                        row_ui.add_space(spacing);

                        let image_rect = Rect::from_min_max(
                            Pos2::new(row_rect.right() - img_w, row_rect.top()),
                            Pos2::new(row_rect.right(), row_rect.bottom()),
                        );
                        if let Some(tex) = &temp.texture {
                            row_ui.painter().add(Shape::image(
                                tex.id(),
                                image_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            ));
                        } else {
                            row_ui
                                .painter()
                                .rect_filled(image_rect, 0.0, Color32::from_gray(30));
                            row_ui.painter().text(
                                image_rect.center(),
                                Align2::CENTER_CENTER,
                                "(Loading…)",
                                FontId::proportional(16.0),
                                Color32::LIGHT_GRAY,
                            );
                        }

                        ui.add_space(spacing);
                    });
                }
            });
    }

    fn show_central_ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let boards_height = 72.0;
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), boards_height),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        self.show_board(ui, PlayerTarget::Opponent);
                        ui.separator();
                        self.show_board(ui, PlayerTarget::Player);
                        ui.add_space(4.0);
                    });
                },
            );

            ui.separator();
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| {
                    if !self.sim_running && self.sim_results.is_empty() {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Iterations:");
                            ui.add(
                                DragValue::new(&mut self.sim_iterations)
                                    .range(1..=10_000)
                                    .speed(1),
                            );
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Player Health:");
                            ui.add(
                                DragValue::new(&mut self.player_health)
                                    .range(1..=9999)
                                    .speed(1),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Opponent Health:");
                            ui.add(
                                DragValue::new(&mut self.opponent_health)
                                    .range(1..=9999)
                                    .speed(1),
                            );
                        });
                        ui.add_space(8.0);
                        if ui.button("Load Simulation…").clicked() {
                            self.toml_file_dialog.pick_file();
                        }
                        if let Some(template_path) = self.toml_file_dialog.take_picked() {
                            tracing::debug!(?template_path, "template selected");
                            match std::fs::read_to_string(&template_path) {
                                Ok(template_str) => {
                                    match toml::from_str::<SimulationTemplate>(&template_str) {
                                        Ok(template) => {
                                            self.player_health = template.player.health;
                                            self.opponent_health = template.opponent.health;
                                            self.sim_load_error = None;

                                            self.player_board.clear();
                                            self.opponent_board.clear();

                                            let mut pos = 0;
                                            for ct in &template.player.card_templates {
                                                if let Some(tc) = self
                                                    .cards_with_texture
                                                    .iter()
                                                    .find(|t| t.card.name == ct.name)
                                                {
                                                    self.player_board.push(CardOnBoard {
                                                        tier: ct.tier.clone(),
                                                        template: tc.clone(),
                                                        position: pos,
                                                        modifications: vec![],
                                                    });
                                                    pos += tc.card.size.board_spaces() as u8;
                                                } else {
                                                    tracing::error!(name = ?ct.name, "unknown card in template");
                                                    self.sim_load_error = Some(format!(
                                                        "Unknown card in template: “{}”",
                                                        ct.name
                                                    ));
                                                    break;
                                                }
                                            }
                                            let mut pos = 0;
                                            for ct in &template.opponent.card_templates {
                                                if let Some(tc) = self
                                                    .cards_with_texture
                                                    .iter()
                                                    .find(|t| t.card.name == ct.name)
                                                {
                                                    self.opponent_board.push(CardOnBoard {
                                                        tier: ct.tier.clone(),
                                                        template: tc.clone(),
                                                        position: pos,
                                                        modifications: vec![],
                                                    });
                                                    pos += tc.card.size.board_spaces() as u8;
                                                } else {
                                                    tracing::error!(name = ?ct.name, "unknown card in template");
                                                    self.sim_load_error = Some(format!(
                                                        "Unknown card in template: “{}”",
                                                        ct.name
                                                    ));
                                                    break;
                                                }
                                            }
                                        }
                                        Err(error) => {
                                            tracing::error!(
                                                ?error,
                                                ?template_path,
                                                "parse template"
                                            );
                                            self.sim_load_error = Some(format!(
                                                "Parsing template file failed:\n{}",
                                                error
                                            ));
                                        }
                                    }
                                }
                                Err(error) => {
                                    tracing::error!(?error, ?template_path, "read template");
                                    self.sim_load_error =
                                        Some(format!("Reading template file failed:\n{}", error));
                                }
                            }
                        }
                        if let Some(err) = &self.sim_load_error {
                            ui.colored_label(Color32::RED, err);
                        }
                        ui.add_space(4.0);
                        ui.add_space(8.0);
                        if ui
                            .add_sized([150.0, 30.0], egui::Button::new("Run Simulation"))
                            .clicked()
                        {
                            let (evt_tx, evt_rx) = std::sync::mpsc::channel();
                            let (res_tx, res_rx) = std::sync::mpsc::channel();

                            let iterations = self.sim_iterations;
                            let base_chunk = iterations / *OPTIMAL_THREAD_COUNT;
                            let remainder = iterations % *OPTIMAL_THREAD_COUNT;

                            let player_cards: Vec<CardTemplate> = self
                                .player_board
                                .clone()
                                .into_iter()
                                .map(Into::into)
                                .collect();
                            let opponent_cards: Vec<CardTemplate> = self
                                .opponent_board
                                .clone()
                                .into_iter()
                                .map(Into::into)
                                .collect();
                            let sim_template: SimulationTemplate = SimulationTemplate {
                                player: PlayerTemplate {
                                    health: self.player_health,
                                    card_templates: player_cards,
                                    skill_templates: vec![],
                                }, // TODO: SKILLS!
                                opponent: PlayerTemplate {
                                    health: self.opponent_health,
                                    card_templates: opponent_cards,
                                    skill_templates: vec![],
                                }, // TODO: SKILLS!
                                seed: None,
                            };
                            let sim_template = Arc::new(sim_template);

                            for i in 0..*OPTIMAL_THREAD_COUNT {
                                let chunk = base_chunk + if i < remainder { 1 } else { 0 };
                                let worker_template = sim_template.clone();
                                let thread_evt_tx = evt_tx.clone();
                                let thread_res_tx = res_tx.clone();
                                thread::spawn(move || {
                                    let base_rng = Simulation::create_rng();
                                    let rng_clone = base_rng.clone();
                                    for _ in 0..chunk {
                                        let mut sim: Simulation =
                                            worker_template.as_ref().clone().try_into().unwrap();
                                        sim = sim.with_channel(thread_evt_tx.clone());
                                        let result = sim.run_once_with_rng(rng_clone.clone());
                                        if let Err(_) = thread_res_tx.send(result) {
                                            break;
                                        }
                                    }
                                });
                            }

                            self.sim_event_rx = Some(evt_rx);
                            self.sim_result_rx = Some(res_rx);
                            self.sim_logs.clear();
                            self.sim_warnings.clear();
                            self.sim_errors.clear();
                            self.sim_running = true;
                            self.sim_completed = 0;
                            self.sim_start = Some(Instant::now());
                        }
                    }

                    if let Some(event_rx) = &self.sim_event_rx {
                        for event in event_rx.try_iter() {
                            match event {
                                DispatchableEvent::Error(msg) => self.sim_errors.push(msg),
                                DispatchableEvent::Warning(msg) => self.sim_warnings.push(msg),
                                DispatchableEvent::Log(msg) => self.sim_logs.push(msg),
                                DispatchableEvent::CardFrozen(id, duration) => self.sim_logs.push(format!("Froze item {id} for {duration}")),
                                DispatchableEvent::Tick => {},
                            }
                        }

                        if let Some(res_rx) = &self.sim_result_rx {
                            for res in res_rx.try_iter() {
                                self.sim_results.push(res);
                            }
                            if self.sim_results.len() >= self.sim_iterations {
                                self.sim_running = false;
                                if let Some(start) = self.sim_start {
                                    self.sim_elapsed = Instant::now() - start;
                                    self.sim_start = None
                                }
                            }
                        }
                    }

                    if !self.sim_running && !self.sim_results.is_empty() {
                        ui.heading("Results Summary");

                        let summary: SimulationSummary = Into::into(&self.sim_results);
                        let winrate = 100.0 * summary.victories as f32 / self.sim_iterations as f32;
                        let loserate = 100.0 * summary.defeats as f32 / self.sim_iterations as f32;
                        let drawrate = 100.0
                            * (summary.draw_timeout as f32 + summary.draw_simultaneous as f32)
                            / self.sim_iterations as f32;
                        let time_taken_per_sim = self.sim_elapsed / self.sim_iterations as u32;

                        ui.separator();

                        egui::Grid::new("results_grid")
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .min_col_width(80.0)
                            .show(ui, |ui| {
                                ui.label("Iterations:");
                                ui.label(self.sim_iterations.to_string());
                                ui.end_row();

                                ui.label("Win rate:");
                                ui.label(format!(
                                    "{winrate:.2}% ({}/{})",
                                    summary.victories, self.sim_iterations,
                                ));
                                ui.end_row();

                                ui.label("Draw rate:");
                                ui.label(format!(
                                    "{drawrate:.2}% ({}/{})",
                                    summary.draw_timeout + summary.draw_simultaneous,
                                    self.sim_iterations
                                ));
                                ui.end_row();

                                ui.label("Lose rate:");
                                ui.label(format!(
                                    "{loserate:.2}% ({}/{})",
                                    summary.defeats, self.sim_iterations
                                ));
                                ui.end_row();

                                ui.label("Defeats:");
                                ui.label(summary.defeats.to_string());
                                ui.end_row();

                                ui.label("Draws:");
                                ui.label(
                                    (summary.draw_timeout + summary.draw_simultaneous).to_string(),
                                );
                                ui.end_row();

                                ui.separator();
                                ui.end_row();

                                ui.label("CPU threads used:");
                                ui.label(format!("{}", *OPTIMAL_THREAD_COUNT));
                                ui.end_row();

                                ui.label("Total duration:");
                                ui.label(format!("{:?}", self.sim_elapsed));
                                ui.end_row();

                                ui.label("Time/sim:");
                                ui.label(format!("{:?}", time_taken_per_sim));
                                ui.end_row();
                            });

                        ui.separator();

                        ui.collapsing("Errors", |ui| {
                            for err in &self.sim_errors {
                                ui.colored_label(Color32::RED, err);
                            }
                        });

                        ui.collapsing("Warnings", |ui| {
                            for warn in &self.sim_warnings {
                                ui.colored_label(Color32::YELLOW, warn);
                            }
                        });

                        ui.collapsing("Logs", |ui| {
                            for log in &self.sim_logs {
                                ui.label(log);
                            }
                        });

                        if ui
                            .add_sized([120.0, 24.0], egui::Button::new("Clear Results"))
                            .clicked()
                        {
                            self.sim_logs.clear();
                            self.sim_results.clear();
                        }
                    }

                    Ok::<(), ()>(())
                },
            );
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        for (texture_id, ci, _size) in self.texture_rx.try_iter() {
            let handle = ctx.load_texture(texture_id.clone(), ci, TextureOptions::default());
            if let Some(textured_card) = self
                .cards_with_texture
                .iter_mut()
                .find(|t| t.card.id == texture_id)
            {
                textured_card.texture = Some(handle);
            }
            ctx.request_repaint();
        }

        for tpl in &mut self.cards_with_texture {
            if tpl.blurred_texture.is_none() {
                let mut backdrop = tpl.texture_path.clone();
                if let (Some(ext), Some(stem)) = (
                    backdrop.extension().and_then(|e| e.to_str()),
                    backdrop.file_stem().and_then(|s| s.to_str()),
                ) {
                    backdrop.set_file_name(format!("{stem}.backdrop.{ext}"));
                    if backdrop.exists() {
                        if let Ok(img) = image::open(&backdrop) {
                            let (w, h) = img.dimensions();
                            let ci = ColorImage::from_rgba_unmultiplied(
                                [w as usize, h as usize],
                                &img.to_rgba8().into_raw(),
                            );
                            let handle = ctx.load_texture(
                                format!("blurred-{}", tpl.card.id),
                                ci,
                                TextureOptions::default(),
                            );
                            tpl.blurred_texture = Some(handle);
                            ctx.request_repaint();
                        }
                    }
                }
            }
        }

        self.toml_file_dialog.update(ctx);

        SidePanel::left("card_select")
            .resizable(false)
            .min_width(350.0)
            .max_width(350.0)
            .default_width(250.0)
            .show(ctx, |ui| {
                self.show_side_panel(ui);
            });

        CentralPanel::default().show(ctx, |ui| {
            self.show_central_ui(ui);
        });

        if self.sim_running {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}
