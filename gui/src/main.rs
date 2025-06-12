#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use eframe::egui;
use egui::{
    Align, Align2, Button, CentralPanel, Color32, ColorImage, Context, CursorIcon, DragValue,
    FontId, Frame, Grid, Layout, Pos2, Rect, ScrollArea, Sense, Shape, SidePanel, Stroke,
    StrokeKind, TextureHandle, TextureOptions, Ui, UiBuilder, Vec2,
};
use egui_file_dialog::FileDialog;
use gamedata::v2_0_0::CONSTRUCT_CARD_BY_NAME;
use gui::interop;
use image::GenericImageView;
use models::v2_0_0::{PlayerTarget, Tier};
use simulator::{
    CardTemplate, DispatchableEvent, GlobalCardId, PlayerHealth, PlayerTemplate, Simulation,
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

struct App {
    texture_rx: Receiver<(String, ColorImage, u8)>,
    texture_tx: Sender<(String, ColorImage, u8)>,
    cards_with_texture: Vec<TexturedCard>,
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
    toml_file_dialog: FileDialog,
    loading_ids: HashSet<String>,
    simulation: Simulation,
}

impl App {
    fn new() -> Self {
        let (texture_tx, texture_rx) = std::sync::mpsc::channel();
        let mut cards_with_texture = Vec::with_capacity(CONSTRUCT_CARD_BY_NAME.len());

        for (_, construct) in CONSTRUCT_CARD_BY_NAME.iter() {
            cards_with_texture.push(construct().into());
        }

        let simulation: Simulation = SimulationTemplate {
            player: PlayerTemplate {
                health: 300,
                regen: 0,
                card_templates: vec![],
                skill_templates: vec![],
            },
            opponent: PlayerTemplate {
                health: 300,
                regen: 0,
                card_templates: vec![],
                skill_templates: vec![],
            },
            seed: None,
        }
        .try_into()
        .unwrap();

        cards_with_texture.sort_by_key(|c: &TexturedCard| c.card.name);
        Self {
            simulation,
            texture_rx,
            texture_tx,
            cards_with_texture,
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
        struct BoardCardView {
            id: GlobalCardId,
            position: u8,
            tier: Tier,
            size: u8,
            inner: models::v2_0_0::Card,
            removed: bool,
        }

        let mut board_views: Vec<BoardCardView> = self
            .simulation
            .get_cards_by_owner(board_owner)
            .iter()
            .filter_map(|gid| {
                self.simulation.cards.get(gid).map(|c| BoardCardView {
                    id: *gid,
                    position: c.position,
                    tier: c.tier,
                    size: c.inner.size.board_spaces() as u8,
                    inner: c.inner.clone(),
                    removed: false,
                })
            })
            .collect();

        let mut slot_to_idx: [Option<usize>; 10] = [None; 10];
        for (i, view) in board_views.iter().enumerate() {
            if (view.position as usize) < 10 {
                slot_to_idx[view.position as usize] = Some(i);
            }
        }

        ui.horizontal_wrapped(|ui| {
            let slot_w = 64.0;
            let slot_h = 96.0;
            let gap = ui.spacing().item_spacing.x;

            for position in 0u8..10 {
                if let Some(i) = slot_to_idx[position as usize] {
                    let view = &mut board_views[i];
                    let extra = (view.size as f32 - 1.0) * (gap + 2.0) + 2.0;
                    let width = slot_w * (view.size as f32) + extra;
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(width, slot_h), Sense::click());

                    if let Some(shared) = self
                        .cards_with_texture
                        .iter_mut()
                        .find(|t| t.card.id == view.inner.id)
                    {
                        if shared.texture.is_none()
                            && !self.loading_ids.contains(&view.id.to_string())
                            && ui.is_rect_visible(rect)
                        {
                            let id_str = view.id.to_string();
                            self.loading_ids.insert(id_str.clone());
                            tracing::debug!(id = ?id_str, path = ?shared.texture_path, "spawn load texture thread");
                            interop::spawn_load_texture_thread(
                                shared.texture_path.clone(),
                                id_str,
                                shared.card.size.board_spaces() as u8,
                                self.texture_tx.clone(),
                            );
                        }

                        if let Some(tex) = &shared.texture {
                            ui.painter().add(Shape::image(
                                tex.id(),
                                rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            ));
                            let border = match view.tier {
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

                            let badge_radius = 8.0;
                            let badge_center =
                                Pos2::new(rect.center().x, rect.top() - badge_radius);
                            let badge_rect = Rect::from_center_size(
                                badge_center,
                                Vec2::splat(badge_radius * 2.0),
                            );
                            ui.painter()
                                .rect_filled(badge_rect, badge_radius, Color32::RED);
                            ui.painter().text(
                                badge_center,
                                Align2::CENTER_CENTER,
                                "124",
                                FontId::proportional(12.0),
                                Color32::WHITE,
                            );
                        }

                        if resp.clicked() {
                            let tiers = view.inner.available_tiers();
                            if let Some(idx) = tiers.iter().position(|t| *t == view.tier) {
                                view.tier = tiers[(idx + 1) % tiers.len()];
                            }
                        }
                        if resp.secondary_clicked() {
                            view.removed = true;
                        }
                    }
                } else {
                    ui.push_id(position, |ui| {
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
                            let overlap = board_views.iter().any(|c| {
                                let start = c.position;
                                let end = start + c.size;
                                (position < end) && (start < position + size)
                            });
                            if !overlap && position + size <= 10 {
                                let global_id = GlobalCardId::default();
                                let ct = CardTemplate {
                                    name: template.card.name.to_string(),
                                    tier: template.card.min_tier(),
                                    modifications: vec![],
                                };
                                if let Ok(new_card) =
                                    ct.create_card_on_board(position, board_owner, global_id)
                                {
                                    self.simulation.cards.insert(global_id, new_card);
                                    match board_owner {
                                        PlayerTarget::Player => {
                                            self.simulation.player.card_ids.push(global_id)
                                        }
                                        PlayerTarget::Opponent => {
                                            self.simulation.opponent.card_ids.push(global_id)
                                        }
                                    }
                                } else {
                                    let todo = true; //TODO: notify user of failure
                                }
                            }
                        }
                    });
                }
            }
        });

        // 4) Synchronize back to the simulation (mutable borrow)
        for view in board_views {
            if view.removed {
                // Remove from simulation
                self.simulation.cards.remove(&view.id);
                match board_owner {
                    PlayerTarget::Player => {
                        self.simulation.player.card_ids.retain(|&g| g != view.id)
                    }
                    PlayerTarget::Opponent => {
                        self.simulation.opponent.card_ids.retain(|&g| g != view.id)
                    }
                }
            } else {
                // Update tier on the real card
                if let Some(card) = self.simulation.cards.get_mut(&view.id) {
                    card.tier = view.tier;
                }
            }
        }
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
                            tracing::debug!(id = ?temp.card.id, path = ?temp.texture_path, "spawn load texture thread");
                            interop::spawn_load_texture_thread(
                                temp.texture_path.clone(),
                                temp.card.id.to_string(),
                                temp.card.size.board_spaces() as u8,
                                self.texture_tx.clone(),
                            );
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
                            Layout::top_down(Align::Min),
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
            let mut player_health_val = self.simulation.player.health.max() as f64;
            let mut opponent_health_val = self.simulation.opponent.health.max() as f64;
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
                                DragValue::new(&mut player_health_val)
                                .range(1..=50000)
                                .speed(1),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Opponent Health:");
                            ui.add(
                                DragValue::new(&mut opponent_health_val)
                                .range(1..=50000)
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
                                        Ok(template) => match template.try_into() {
                                            Ok(sim) => self.simulation = sim,
                                            Err(error) => {
                                                self.sim_load_error = Some(format!(
                                                    "Loading simulation from template failed:\n{}",
                                                    error
                                                ))
                                            }
                                        },
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
                            .add_sized([150.0, 30.0], Button::new("Run Simulation"))
                            .clicked()
                        {
                            let (evt_tx, evt_rx) = std::sync::mpsc::channel();
                            let (res_tx, res_rx) = std::sync::mpsc::channel();

                            let iterations = self.sim_iterations;
                            let base_chunk = iterations / *OPTIMAL_THREAD_COUNT;
                            let remainder = iterations % *OPTIMAL_THREAD_COUNT;

                            tracing::debug!(thread_count = ?(*OPTIMAL_THREAD_COUNT), "executing simulation");
                            for i in 0..*OPTIMAL_THREAD_COUNT {
                                let chunk = base_chunk + if i < remainder { 1 } else { 0 };
                                let thread_evt_tx = evt_tx.clone();
                                let thread_res_tx = res_tx.clone();
                                let simulation = self.simulation.clone();
                                interop::spawn_run_simulation_thread(
                                    chunk,
                                    simulation,
                                    thread_res_tx,
                                    thread_evt_tx,
                                );
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
                            let todo = true; //TODO fix this
                            match event {
                                DispatchableEvent::Error(msg) => self.sim_errors.push(msg),
                                DispatchableEvent::Warning(msg) => self.sim_warnings.push(msg),
                                DispatchableEvent::Log(msg) => self.sim_logs.push(msg),
                                DispatchableEvent::CardFrozen(id, duration) => self
                                    .sim_logs
                                    .push(format!("Froze item {id} for {duration}")),
                                DispatchableEvent::Tick => {}
                                _ => {}
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

                        Grid::new("results_grid")
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
                            .add_sized([120.0, 24.0], Button::new("Clear Results"))
                            .clicked()
                        {
                            self.sim_logs.clear();
                            self.sim_results.clear();
                        }
                    }

                    Ok::<(), ()>(())
                },
            );
            self.simulation.player.health =
                PlayerHealth(player_health_val as i64, player_health_val as u64);
            self.simulation.opponent.health =
                PlayerHealth(opponent_health_val as i64, opponent_health_val as u64);
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
