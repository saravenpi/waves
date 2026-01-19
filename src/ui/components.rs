use eframe::egui;

/// Reusable UI components with consistent black and white, minimalistic, squared styling

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
}

pub struct Button {
    text: String,
    style: ButtonStyle,
    selected: bool,
    min_size: egui::Vec2,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: ButtonStyle::Primary,
            selected: false,
            min_size: egui::vec2(100.0, 35.0),
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, primary_color: egui::Color32) -> egui::Response {
        let (fill_color, text_color, stroke_color) = if self.selected {
            match self.style {
                ButtonStyle::Primary => (primary_color, egui::Color32::WHITE, primary_color),
                ButtonStyle::Secondary => (egui::Color32::WHITE, egui::Color32::from_rgb(16, 16, 16), egui::Color32::WHITE),
            }
        } else {
            match self.style {
                ButtonStyle::Primary => (egui::Color32::TRANSPARENT, primary_color, primary_color),
                ButtonStyle::Secondary => (egui::Color32::TRANSPARENT, egui::Color32::from_rgb(150, 150, 150), egui::Color32::WHITE),
            }
        };

        let button = egui::Button::new(egui::RichText::new(&self.text).size(14.0).color(text_color))
            .fill(fill_color)
            .stroke(egui::Stroke::new(1.0, stroke_color))
            .rounding(0.0)
            .min_size(self.min_size);

        ui.add(button)
    }
}

pub struct Modal {
    title: String,
    width: f32,
    height: f32,
}

impl Modal {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 400.0,
            height: 150.0,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn show<R>(
        self,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> (Option<R>, bool) {
        let mut clicked_outside = false;
        let mut result = None;

        let response = egui::Window::new("")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .fixed_size([self.width, self.height])
            .frame(egui::Frame {
                fill: egui::Color32::TRANSPARENT,
                stroke: egui::Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                egui::Frame {
                    fill: egui::Color32::from_rgb(16, 16, 16),
                    stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    inner_margin: egui::Margin::same(20.0),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        if !self.title.is_empty() {
                            ui.label(egui::RichText::new(&self.title).size(18.0).color(egui::Color32::WHITE).strong());
                            ui.add_space(15.0);
                        }
                        add_contents(ui)
                    })
                    .inner
                })
                .inner
            });

        if let Some(resp) = response {
            if resp.response.clicked_elsewhere() {
                clicked_outside = true;
            }
            if let Some(inner) = resp.inner {
                result = Some(inner);
            }
        }

        (result, clicked_outside)
    }
}

pub struct IconButton {
    icon: String,
    size: f32,
    color: Option<egui::Color32>,
}

impl IconButton {
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            size: 28.0,
            color: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let color = self.color.unwrap_or(egui::Color32::WHITE);
        let button = egui::Button::new(egui::RichText::new(&self.icon).size(self.size).color(color))
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE);
        ui.add(button)
    }
}

pub struct Select {
    options: Vec<(String, String)>,
    selected_index: usize,
}

impl Select {
    pub fn new(options: Vec<(String, String)>, selected_index: usize) -> Self {
        Self {
            options,
            selected_index,
        }
    }

    pub fn show(self, ui: &mut egui::Ui, primary_color: egui::Color32) -> (egui::Response, Option<usize>) {
        let mut clicked_index = None;

        let total_width = ui.available_width();
        let button_width = total_width / self.options.len() as f32;

        let response = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            for (idx, (icon, label)) in self.options.iter().enumerate() {
                let is_selected = idx == self.selected_index;

                let (bg_color, text_color, stroke) = if is_selected {
                    (primary_color, egui::Color32::WHITE, egui::Stroke::new(1.0, primary_color))
                } else {
                    (egui::Color32::TRANSPARENT, egui::Color32::from_rgb(150, 150, 150), egui::Stroke::new(1.0, egui::Color32::WHITE))
                };

                let button_text = if icon.is_empty() {
                    label.clone()
                } else {
                    format!("{} {}", icon, label)
                };

                let button = egui::Button::new(egui::RichText::new(&button_text).size(12.0).color(text_color))
                    .fill(bg_color)
                    .stroke(stroke)
                    .rounding(0.0)
                    .min_size(egui::vec2(button_width, 28.0));

                if ui.add(button).clicked() {
                    clicked_index = Some(idx);
                }
            }
        });

        (response.response, clicked_index)
    }
}

pub struct ConfirmDialog {
    title: String,
    message: String,
    confirm_text: String,
    cancel_text: String,
    selected: usize,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "Confirm".to_string(),
            cancel_text: "Cancel".to_string(),
            selected: 1,
        }
    }

    pub fn confirm_text(mut self, text: impl Into<String>) -> Self {
        self.confirm_text = text.into();
        self
    }

    pub fn cancel_text(mut self, text: impl Into<String>) -> Self {
        self.cancel_text = text.into();
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn show(
        mut self,
        ctx: &egui::Context,
        primary_color: egui::Color32,
    ) -> (bool, bool, usize) {
        let mut confirmed = false;
        let mut cancelled = false;

        let (_response, clicked_outside) = Modal::new(&self.title)
            .size(400.0, 150.0)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new(&self.message).size(14.0).color(egui::Color32::from_rgb(200, 200, 200)));
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 220.0) / 2.0);

                    if Button::new(&self.cancel_text)
                        .style(ButtonStyle::Secondary)
                        .selected(self.selected == 0)
                        .show(ui, primary_color)
                        .clicked()
                    {
                        cancelled = true;
                    }

                    ui.add_space(20.0);

                    if Button::new(&self.confirm_text)
                        .style(ButtonStyle::Primary)
                        .selected(self.selected == 1)
                        .show(ui, primary_color)
                        .clicked()
                    {
                        confirmed = true;
                    }
                });
            });

        if clicked_outside {
            cancelled = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            cancelled = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft)) {
            self.selected = 0;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight)) {
            self.selected = 1;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if self.selected == 0 {
                cancelled = true;
            } else {
                confirmed = true;
            }
        }

        (confirmed, cancelled, self.selected)
    }
}
