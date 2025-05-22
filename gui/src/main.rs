#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, sync::mpsc::Receiver, thread};

use eframe::egui;
use egui::{
    Align, Align2, CentralPanel, Color32, ColorImage, Context, CursorIcon, FontId, Frame, LayerId,
    Layout, Order, Pos2, ProgressBar, Rect, ScrollArea, Sense, Shape, SidePanel, Stroke,
    StrokeKind, TextureHandle, TextureOptions, Ui, UiBuilder, Vec2,
};
use gamedata::v2_0_0::CONSTRUCT_CARD_BY_NAME;
use image::{GenericImageView, ImageFormat};
use models::v2_0_0::PlayerTarget;
use simulator::simulation::Simulation;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
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
    pub backdrop_loaded: bool,
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
            backdrop_loaded: false,
        }
    }
}

struct CardOnBoard {
    template: TexturedCard,
    position: u8,
}

struct App {
    selected_card: Option<usize>,
    texture_rx: Receiver<(String, ColorImage, u8)>,
    cards_with_texture: Vec<TexturedCard>,
    player_board: Vec<CardOnBoard>,
    opponent_board: Vec<CardOnBoard>,
    active_simulation: Option<Simulation>,
}

impl App {
    fn new() -> Self {
        let (texture_tx, texture_rx) = std::sync::mpsc::channel();
        let mut cards_with_texture = Vec::new();

        for (_, construct) in CONSTRUCT_CARD_BY_NAME.iter() {
            let card = construct();
            let id = card.id;
            let size = card.size.board_spaces();
            let textured_card: TexturedCard = card.into();
            cards_with_texture.push(textured_card.clone());

            let tx_clone = texture_tx.clone();
            let image_path = textured_card.texture_path.clone();
            thread::spawn(move || {
                tracing::debug!(texture = ?id, "load texture");
                let bytes = std::fs::read(&image_path).unwrap();
                let dyn_img = image::load_from_memory_with_format(&bytes, ImageFormat::Avif)
                    .unwrap()
                    .to_rgba8();
                let (w, h) = dyn_img.dimensions();
                let ci = ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    &dyn_img.into_raw(),
                );
                tx_clone.send((id.to_string(), ci, size)).unwrap();
            });
        }
        drop(texture_tx);
        cards_with_texture.sort_by_key(|c| c.card.name);

        Self {
            selected_card: None,
            texture_rx,
            cards_with_texture,
            player_board: Vec::new(),
            opponent_board: Vec::new(),
            active_simulation: None,
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
            let h_spacing = ui.spacing().item_spacing.x;
            let mut board_position = 0u8;

            while board_position < 10 {
                if let Some(inst) = board.iter().find(|c| c.position == board_position) {
                    let card_width_board_spaces = inst.template.card.size.board_spaces() as f32;
                    let extra_width = (card_width_board_spaces - 1.0) * h_spacing;
                    let full_width = slot_w * card_width_board_spaces + extra_width;
                    let full_position = ui.next_widget_position();
                    let full_rect =
                        Rect::from_min_size(full_position, Vec2::new(full_width, slot_h));
                    let (_r, resp) = ui.allocate_exact_size(full_rect.size(), Sense::click());

                    if let Some(backdrop) = &inst.template.blurred_texture {
                        ui.painter().add(Shape::image(
                            backdrop.id(),
                            full_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        ));
                    }

                    if let Some(tex) = &inst.template.texture {
                        let orig_w = slot_w * card_width_board_spaces;
                        let draw_rect = if card_width_board_spaces > 1.0 {
                            Rect::from_center_size(full_rect.center(), Vec2::new(orig_w, slot_h))
                        } else {
                            full_rect
                        };

                        ui.painter().add(Shape::image(
                            tex.id(),
                            draw_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        ));
                    }

                    let position_increment = inst.template.card.size.board_spaces() as u8;
                    if resp.clicked() {
                        board.retain(|c| c.position != board_position);
                    }

                    board_position += position_increment;
                } else {
                    // empty slot → drop zone (unchanged)
                    ui.push_id(board_position, |ui| {
                        let frame = Frame::new()
                            .stroke(Stroke::new(1.0, ui.style().visuals.selection.stroke.color));
                        if let (_, Some(card_idx)) = ui.dnd_drop_zone::<usize, _>(frame, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(
                                Vec2::new(slot_w, slot_h),
                                Sense::click_and_drag(),
                            );
                            ui.painter().rect_stroke(
                                rect,
                                4.0,
                                Stroke::new(1.0, Color32::LIGHT_GRAY),
                                StrokeKind::Middle,
                            );
                            resp
                        }) {
                            let template = self.cards_with_texture[*card_idx].clone();
                            let card_size = template.card.size.board_spaces() as u8;
                            let overlap = board.iter().any(|c| {
                                let start = c.position;
                                let end = start + c.template.card.size.board_spaces() as u8;
                                (board_position < end) && (start < board_position + card_size)
                            });
                            if !overlap && board_position + card_size <= 10 {
                                board.push(CardOnBoard {
                                    template,
                                    position: board_position,
                                });
                            }
                        }
                    });
                    board_position += 1;
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

                    ui.dnd_drag_source(ui.id().with(idx), idx, |ui| {
                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), entry_height),
                            Sense::drag(),
                        );

                        if row_resp.hovered() {
                            let hovered = ui.style().visuals.widgets.hovered;
                            ui.painter().rect_filled(
                                row_rect,
                                hovered.corner_radius,
                                hovered.bg_fill,
                            );
                            self.selected_card = Some(idx);
                        }

                        // drag preview (just the image)
                        if ui.memory(|m| m.is_being_dragged(ui.id().with(idx))) {
                            if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()) {
                                if let Some(tex) = &temp.texture {
                                    let size = Vec2::new(img_w, entry_height);
                                    let rect = Rect::from_min_size(pointer - size * 0.5, size);
                                    let layer = LayerId::new(Order::Tooltip, ui.id().with(idx));
                                    ui.ctx().layer_painter(layer).add(Shape::image(
                                        tex.id(),
                                        rect,
                                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                        Color32::WHITE,
                                    ));
                                }
                            }
                            return;
                        }

                        row_resp.on_hover_cursor(CursorIcon::PointingHand);

                        let mut row_ui = ui.new_child(
                            UiBuilder::new()
                                .max_rect(row_rect)
                                .layout(Layout::left_to_right(Align::Center)),
                        );

                        let text_w = (row_rect.width() - img_w - spacing).max(0.0);
                        row_ui.allocate_ui_with_layout(
                            Vec2::new(text_w, entry_height),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label(temp.card.name);
                                ui.horizontal_wrapped(|ui| {
                                    for tag in &temp.card.tags {
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
                                .rect_filled(image_rect, 4.0, Color32::from_gray(30));
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
                        ui.add(
                            ProgressBar::new(
                                self.active_simulation
                                    .as_ref()
                                    .map(|s| s.opponent.health.fraction())
                                    .unwrap_or(1.0),
                            )
                            .fill(Color32::from_rgb(0xb8, 0xbb, 0x26))
                            .desired_width(ui.available_width()),
                        );
                        ui.add_space(4.0);
                        self.show_board(ui, PlayerTarget::Opponent);
                        ui.separator();
                        self.show_board(ui, PlayerTarget::Player);
                        ui.add_space(4.0);
                        ui.add(
                            ProgressBar::new(
                                self.active_simulation
                                    .as_ref()
                                    .map(|s| s.player.health.fraction())
                                    .unwrap_or(1.0),
                            )
                            .fill(Color32::from_rgb(0xb8, 0xbb, 0x26))
                            .desired_width(ui.available_width()),
                        );
                    });
                },
            );

            ui.separator();

            let avail = ui.available_size();
            ui.allocate_ui_with_layout(
                Vec2::new(avail.x, avail.y),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    if let Some(idx) = self.selected_card {
                        let textured = &self.cards_with_texture[idx];
                        ui.heading(textured.card.name);
                        ui.horizontal_wrapped(|ui| {
                            for tag in &textured.card.tags {
                                ui.label(format!("({tag:?})"));
                            }
                        });
                        if let Some(tex) = &textured.texture {
                            ui.image(tex);
                        }
                    } else {
                        ui.label("Select a card to see details");
                    }
                },
            );
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 1) Load originals
        for (texture_id, ci, _size) in self.texture_rx.try_iter() {
            let handle = ctx.load_texture(texture_id.clone(), ci, TextureOptions::default());
            if let Some(tpl) = self
                .cards_with_texture
                .iter_mut()
                .find(|t| t.card.id == texture_id)
            {
                tpl.texture = Some(handle);
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
    }
}
