#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::PathBuf, sync::mpsc::Receiver};

use eframe::egui;
use egui::{
    Align, Align2, CentralPanel, Color32, ColorImage, Context, CursorIcon, FontId, Layout, Pos2,
    Rect, ScrollArea, Sense, Shape, SidePanel, Stroke, StrokeKind, TextureHandle, TextureOptions,
    Ui, UiBuilder, Vec2,
};
use gamedata::v2_0_0::CONSTRUCT_CARD_BY_NAME;
use image::ImageFormat;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();
    tracing::info!("launch");
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        &format!("Emporium v{}", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|_| Ok(Box::new(Content::new()))),
    )
}

#[derive(Clone)]
struct TexturedCard {
    texture: Option<TextureHandle>,
    card: models::v2_0_0::Card,
}

impl From<models::v2_0_0::Card> for TexturedCard {
    fn from(value: models::v2_0_0::Card) -> Self {
        Self {
            card: value,
            texture: None,
        }
    }
}

struct CardInstance {
    template: TexturedCard,
    position: u8,
}

struct Content {
    selected_card: Option<usize>,
    texture_rx: Receiver<(String, ColorImage, u8)>,
    cards_with_texture: Vec<TexturedCard>,
    boards: [Vec<CardInstance>; 2],
}

impl Content {
    // TODO: use egui log view?
    fn new() -> Self {
        let (texture_tx, texture_rx) = std::sync::mpsc::channel();
        let mut cards_with_texture = Vec::new();

        for (_, construct) in CONSTRUCT_CARD_BY_NAME.iter() {
            let card = construct();
            let id = card.id;
            let size = card.size.board_spaces();
            let textured_card: TexturedCard = card.into();
            cards_with_texture.push(textured_card);

            let tx_clone = texture_tx.clone();
            let image_path =
                PathBuf::from(format!("gamedata/src/v2_0_0/cards/images/{}.avif", &id));
            std::thread::spawn(
                move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                    tracing::debug!(texture = ?id, "load texture");
                    let bytes = std::fs::read(&image_path).inspect_err(|error| {
                        tracing::error!(?error, ?image_path, "read texture file")
                    })?;
                    let dyn_img =
                        image::load_from_memory_with_format(&bytes, ImageFormat::Avif)?.to_rgba8();
                    let (w, h) = dyn_img.dimensions();
                    let ci = ColorImage::from_rgba_unmultiplied(
                        [w as usize, h as usize],
                        &dyn_img.into_raw(),
                    );
                    tx_clone.send((id.to_string(), ci, size))?;
                    Ok(())
                },
            );
        }
        drop(texture_tx);
        cards_with_texture.sort_by_key(|c| c.card.name);

        Self {
            selected_card: None,
            texture_rx,
            cards_with_texture,
            boards: [Vec::new(), Vec::new()],
        }
    }

    fn show_board(&mut self, ui: &mut Ui, player: usize) {
        let board = &mut self.boards[player];
        ui.horizontal_wrapped(|ui| {
            let slot_w = 64.0;
            let slot_h = 96.0;
            let mut i: u8 = 0;
            while i < 10 {
                if let Some(inst) = board.iter().find(|c| c.position == i) {
                    let size = inst.template.card.size.board_spaces();
                    let desired = Vec2::new(slot_w * size as f32, slot_h);
                    let (rect, resp) = ui.allocate_exact_size(desired, Sense::click());
                    ui.painter().rect_filled(rect, 4.0, Color32::WHITE);

                    if let Some(tex) = &inst.template.texture {
                        ui.painter().add(Shape::image(
                            tex.id(),
                            rect,
                            Rect::from_min_size(Pos2::ZERO, Vec2::ZERO),
                            Color32::WHITE,
                        ));
                    } else {
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            "(Loading…)",
                            Default::default(),
                            Color32::LIGHT_GRAY,
                        );
                    }

                    if resp.clicked() {
                        board.retain(|c| c.position != i);
                    }
                    i += size;
                } else {
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(slot_w, slot_h), Sense::click());
                    ui.painter().rect_stroke(
                        rect,
                        4.0,
                        Stroke::new(1.0, Color32::LIGHT_GRAY),
                        StrokeKind::Middle,
                    );

                    if resp.clicked() {
                        tracing::info!("click");
                    }
                    i += 1;
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

                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), entry_height),
                        Sense::hover(),
                    );

                    if row_resp.clicked() {
                        self.selected_card = Some(idx);
                    }

                    if row_resp.on_hover_cursor(CursorIcon::PointingHand).hovered() {
                        let fill = ui.style().visuals.selection.bg_fill;
                        ui.painter().rect_filled(row_rect, 4.0, fill);
                    }

                    let mut row_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(row_rect)
                            .layout(Layout::left_to_right(Align::Center))
                            .sense(Sense::hover()),
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
                }
            });
    }

    fn show_central_ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let boards_height = 72.0;
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), boards_height),
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.vertical_centered(|ui| {
                        self.show_board(ui, 0);
                        ui.separator();
                        self.show_board(ui, 1);
                    });
                },
            );

            ui.separator(); // horizontal line between the two rows

            // 2) Bottom row: use remaining height for item details
            let avail = ui.available_size();
            ui.allocate_ui_with_layout(
                Vec2::new(avail.x, avail.y),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    if let Some(idx) = self.selected_card {
                        let textured_card = &self.cards_with_texture[idx];
                        let card = &textured_card.card;
                        ui.heading(card.name);
                        ui.horizontal_wrapped(|ui| {
                            for tag in &card.tags {
                                ui.label(format!("({tag:?})"));
                            }
                        });
                        if let Some(tex) = &textured_card.texture {
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

impl eframe::App for Content {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        for (texture_id, ci, _size) in self.texture_rx.try_iter() {
            let tex = ctx.load_texture(texture_id.clone(), ci, TextureOptions::default());
            if let Some(tmp) = self
                .cards_with_texture
                .iter_mut()
                .find(|t| t.card.id == texture_id)
            {
                tracing::debug!(texture = ?texture_id, "texture load complete");
                tmp.texture = Some(tex);
            }
            ctx.request_repaint();
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
