use eframe::egui;
use std::path::PathBuf;

pub enum ContextMenuAction {
    Rename,
    Delete,
    Copy,
    Cut,
    ToggleFavorite,
    EditMetadata,
}

#[allow(dead_code)]
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

pub fn show_text_prompt(
    ctx: &egui::Context,
    hint: &str,
    text: &mut String,
) -> (bool, bool) {
    let mut confirmed = false;
    let mut cancelled = false;

    let window_response = egui::Window::new("")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_size([400.0, 60.0])
        .frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT,
            stroke: egui::Stroke::NONE,
            ..Default::default()
        })
        .show(ctx, |ui| {
            egui::Frame {
                fill: egui::Color32::BLACK,
                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
                inner_margin: egui::Margin::same(12.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                let response = ui.add_sized(
                    [ui.available_width(), 20.0],
                    egui::TextEdit::singleline(text)
                        .font(egui::TextStyle::Monospace)
                        .hint_text(hint)
                        .frame(false)
                );

                response.request_focus();

                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !text.is_empty() {
                    confirmed = true;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancelled = true;
                }
            });
        });

    if let Some(response) = window_response {
        if ctx.input(|i| i.pointer.primary_released()) {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                if !response.response.rect.contains(pos) {
                    cancelled = true;
                }
            }
        }
    }

    (confirmed, cancelled)
}

pub fn show_context_menu(
    ctx: &egui::Context,
    _path: &PathBuf,
    pos: egui::Pos2,
    _is_dir: bool,
) -> Option<ContextMenuAction> {
    let mut action = None;

    egui::Area::new(egui::Id::new("context_menu"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame {
                fill: egui::Color32::BLACK,
                stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                inner_margin: egui::Margin::same(4.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_min_width(150.0);

                let item_height = 32.0;

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), item_height),
                    egui::Sense::click()
                );

                let (bg_color, text_color, stroke) = if response.clicked() {
                    (egui::Color32::WHITE, egui::Color32::BLACK, egui::Stroke::NONE)
                } else if response.hovered() {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::new(1.0, egui::Color32::WHITE))
                } else {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::NONE)
                };

                ui.painter().rect_filled(rect, 0.0, bg_color);
                if stroke != egui::Stroke::NONE {
                    ui.painter().rect_stroke(rect, 0.0, stroke);
                }

                if response.clicked() {
                    action = Some(ContextMenuAction::Rename);
                }

                ui.painter().text(
                    rect.left_center() + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "Rename",
                    egui::FontId::monospace(14.0),
                    text_color,
                );

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), item_height),
                    egui::Sense::click()
                );

                let (bg_color, text_color, stroke) = if response.clicked() {
                    (egui::Color32::WHITE, egui::Color32::BLACK, egui::Stroke::NONE)
                } else if response.hovered() {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::new(1.0, egui::Color32::WHITE))
                } else {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::NONE)
                };

                ui.painter().rect_filled(rect, 0.0, bg_color);
                if stroke != egui::Stroke::NONE {
                    ui.painter().rect_stroke(rect, 0.0, stroke);
                }

                if response.clicked() {
                    action = Some(ContextMenuAction::Delete);
                }

                ui.painter().text(
                    rect.left_center() + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "Delete",
                    egui::FontId::monospace(14.0),
                    text_color,
                );

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), item_height),
                    egui::Sense::click()
                );

                let (bg_color, text_color, stroke) = if response.clicked() {
                    (egui::Color32::WHITE, egui::Color32::BLACK, egui::Stroke::NONE)
                } else if response.hovered() {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::new(1.0, egui::Color32::WHITE))
                } else {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::NONE)
                };

                ui.painter().rect_filled(rect, 0.0, bg_color);
                if stroke != egui::Stroke::NONE {
                    ui.painter().rect_stroke(rect, 0.0, stroke);
                }

                if response.clicked() {
                    action = Some(ContextMenuAction::Copy);
                }

                ui.painter().text(
                    rect.left_center() + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "Copy",
                    egui::FontId::monospace(14.0),
                    text_color,
                );

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), item_height),
                    egui::Sense::click()
                );

                let (bg_color, text_color, stroke) = if response.clicked() {
                    (egui::Color32::WHITE, egui::Color32::BLACK, egui::Stroke::NONE)
                } else if response.hovered() {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::new(1.0, egui::Color32::WHITE))
                } else {
                    (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::NONE)
                };

                ui.painter().rect_filled(rect, 0.0, bg_color);
                if stroke != egui::Stroke::NONE {
                    ui.painter().rect_stroke(rect, 0.0, stroke);
                }

                if response.clicked() {
                    action = Some(ContextMenuAction::Cut);
                }

                ui.painter().text(
                    rect.left_center() + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "Cut",
                    egui::FontId::monospace(14.0),
                    text_color,
                );

                if !_is_dir {
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), item_height),
                        egui::Sense::click()
                    );

                    let (bg_color, text_color, stroke) = if response.clicked() {
                        (egui::Color32::WHITE, egui::Color32::BLACK, egui::Stroke::NONE)
                    } else if response.hovered() {
                        (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::new(1.0, egui::Color32::WHITE))
                    } else {
                        (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::NONE)
                    };

                    ui.painter().rect_filled(rect, 0.0, bg_color);
                    if stroke != egui::Stroke::NONE {
                        ui.painter().rect_stroke(rect, 0.0, stroke);
                    }

                    if response.clicked() {
                        action = Some(ContextMenuAction::ToggleFavorite);
                    }

                    ui.painter().text(
                        rect.left_center() + egui::vec2(10.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        "Toggle Favorite",
                        egui::FontId::monospace(14.0),
                        text_color,
                    );

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), item_height),
                        egui::Sense::click()
                    );

                    let (bg_color, text_color, stroke) = if response.clicked() {
                        (egui::Color32::WHITE, egui::Color32::BLACK, egui::Stroke::NONE)
                    } else if response.hovered() {
                        (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::new(1.0, egui::Color32::WHITE))
                    } else {
                        (egui::Color32::BLACK, egui::Color32::WHITE, egui::Stroke::NONE)
                    };

                    ui.painter().rect_filled(rect, 0.0, bg_color);
                    if stroke != egui::Stroke::NONE {
                        ui.painter().rect_stroke(rect, 0.0, stroke);
                    }

                    if response.clicked() {
                        action = Some(ContextMenuAction::EditMetadata);
                    }

                    ui.painter().text(
                        rect.left_center() + egui::vec2(10.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        "Edit Metadata",
                        egui::FontId::monospace(14.0),
                        text_color,
                    );
                }
            });
        });

    action
}
