use eframe::egui;

pub fn square_spinner(ui: &mut egui::Ui, size: f32, color: egui::Color32) {
    let time = ui.input(|i| i.time) as f32;
    let rect = ui.allocate_space(egui::vec2(size, size)).1;
    let painter = ui.painter();

    let center = rect.center();
    let square_size = size * 0.6;
    let half_size = square_size / 2.0;

    let rotation = time * 2.0;

    let corners = [
        egui::vec2(-half_size, -half_size),
        egui::vec2(half_size, -half_size),
        egui::vec2(half_size, half_size),
        egui::vec2(-half_size, half_size),
    ];

    let rotated_corners: Vec<egui::Pos2> = corners
        .iter()
        .map(|corner| {
            let cos = rotation.cos();
            let sin = rotation.sin();
            let rotated_x = corner.x * cos - corner.y * sin;
            let rotated_y = corner.x * sin + corner.y * cos;
            egui::pos2(center.x + rotated_x, center.y + rotated_y)
        })
        .collect();

    painter.add(egui::Shape::closed_line(
        rotated_corners.clone(),
        egui::Stroke::new(3.0, color),
    ));

    let inner_size = square_size * 0.5;
    let inner_half = inner_size / 2.0;
    let inner_rotation = -time * 3.0;

    let inner_corners = [
        egui::vec2(-inner_half, -inner_half),
        egui::vec2(inner_half, -inner_half),
        egui::vec2(inner_half, inner_half),
        egui::vec2(-inner_half, inner_half),
    ];

    let inner_rotated: Vec<egui::Pos2> = inner_corners
        .iter()
        .map(|corner| {
            let cos = inner_rotation.cos();
            let sin = inner_rotation.sin();
            let rotated_x = corner.x * cos - corner.y * sin;
            let rotated_y = corner.x * sin + corner.y * cos;
            egui::pos2(center.x + rotated_x, center.y + rotated_y)
        })
        .collect();

    painter.add(egui::Shape::closed_line(
        inner_rotated,
        egui::Stroke::new(2.0, color),
    ));

    ui.ctx().request_repaint();
}

pub fn square_spinner_with_text(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        square_spinner(ui, 40.0, color);
        ui.add_space(10.0);
        ui.label(egui::RichText::new(text).size(14.0).color(color));
    });
}
