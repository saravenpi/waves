use eframe::egui;

/// Reusable UI components with consistent black and white, minimalistic, squared styling

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
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

    pub fn min_size(mut self, size: egui::Vec2) -> Self {
        self.min_size = size;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, primary_color: egui::Color32) -> egui::Response {
        let (fill_color, text_color, stroke_color) = if self.selected {
            match self.style {
                ButtonStyle::Primary => (primary_color, egui::Color32::WHITE, primary_color),
                ButtonStyle::Secondary => (egui::Color32::WHITE, egui::Color32::BLACK, egui::Color32::WHITE),
                ButtonStyle::Danger => {
                    let danger_color = egui::Color32::from_rgb(220, 50, 50);
                    (danger_color, egui::Color32::WHITE, danger_color)
                }
            }
        } else {
            match self.style {
                ButtonStyle::Primary => (egui::Color32::TRANSPARENT, primary_color, primary_color),
                ButtonStyle::Secondary => (egui::Color32::TRANSPARENT, egui::Color32::from_rgb(150, 150, 150), egui::Color32::WHITE),
                ButtonStyle::Danger => {
                    let danger_color = egui::Color32::from_rgb(220, 50, 50);
                    (egui::Color32::TRANSPARENT, danger_color, danger_color)
                }
            }
        };

        let button = egui::Button::new(egui::RichText::new(&self.text).size(14.0).color(text_color))
            .fill(fill_color)
            .stroke(egui::Stroke::new(1.0, stroke_color))
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
                    fill: egui::Color32::BLACK,
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

pub struct TextInput {
    hint: String,
    width: Option<f32>,
}

impl TextInput {
    pub fn new(hint: impl Into<String>) -> Self {
        Self {
            hint: hint.into(),
            width: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut egui::Ui, text: &mut String) -> egui::Response {
        let text_edit = egui::TextEdit::singleline(text)
            .hint_text(&self.hint)
            .frame(true)
            .desired_width(self.width.unwrap_or(ui.available_width()));

        let response = ui.add(text_edit);

        ui.style_mut().visuals.extreme_bg_color = egui::Color32::BLACK;
        ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::BLACK;
        ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(20, 20, 20);
        ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::from_rgb(30, 30, 30);
        ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        ui.style_mut().visuals.selection.bg_fill = egui::Color32::from_rgb(150, 100, 255);

        response
    }
}

pub struct Toggle {
    text: String,
}

impl Toggle {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
        }
    }

    pub fn show(self, ui: &mut egui::Ui, value: &mut bool, primary_color: egui::Color32) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&self.text).size(14.0).color(egui::Color32::WHITE));
            ui.add_space(10.0);

            let rect_size = egui::vec2(45.0, 22.0);
            let (rect, response) = ui.allocate_exact_size(rect_size, egui::Sense::click());

            if response.clicked() {
                *value = !*value;
            }

            let _visuals = ui.style().interact(&response);
            let bg_color = if *value { primary_color } else { egui::Color32::from_rgb(40, 40, 40) };
            let border_color = if response.hovered() { egui::Color32::from_rgb(200, 200, 200) } else { egui::Color32::WHITE };

            ui.painter().rect(
                rect,
                0.0,
                bg_color,
                egui::Stroke::new(1.0, border_color),
            );

            let circle_radius = 8.0;
            let circle_offset = if *value { rect.width() - circle_radius - 5.0 } else { circle_radius + 5.0 };
            let circle_pos = egui::pos2(rect.left() + circle_offset, rect.center().y);

            ui.painter().circle(
                circle_pos,
                circle_radius,
                egui::Color32::WHITE,
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );

            response
        })
        .inner
    }
}

pub struct Slider {
    label: String,
    min: f32,
    max: f32,
    suffix: String,
}

impl Slider {
    pub fn new(label: impl Into<String>, min: f32, max: f32) -> Self {
        Self {
            label: label.into(),
            min,
            max,
            suffix: String::new(),
        }
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    pub fn show(self, ui: &mut egui::Ui, value: &mut f32, primary_color: egui::Color32) -> egui::Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.label).size(14.0).color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let display_value = if !self.suffix.is_empty() {
                        format!("{:.0}{}", value, self.suffix)
                    } else {
                        format!("{:.2}", value)
                    };
                    ui.label(egui::RichText::new(display_value).size(12.0).color(egui::Color32::from_rgb(150, 150, 150)));
                });
            });

            ui.add_space(5.0);

            let slider = egui::Slider::new(value, self.min..=self.max)
                .show_value(false);

            let response = ui.add(slider);

            ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 40, 40);
            ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 50, 50);
            ui.style_mut().visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, primary_color);
            ui.style_mut().visuals.widgets.active.bg_fill = primary_color;
            ui.style_mut().visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            ui.style_mut().visuals.selection.bg_fill = primary_color;

            response
        })
        .inner
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
