use eframe::egui::{self, Color32, CornerRadius, RichText, Vec2};

/// Карточка с фиксированным скруглением и гарантированным растяжением по ширине
pub fn card_frame(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    egui::Frame::group(ui.style())
        .corner_radius(CornerRadius::same(8))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        })
        .response
}

/// Кнопка вкладки для панели навигации (Segmented Control)
pub fn tab_button(ui: &mut egui::Ui, is_selected: bool, title: &str) -> egui::Response {
    let (bg_color, text_color) = if is_selected {
        (Color32::from_rgb(32, 100, 210), Color32::WHITE)
    } else {
        (Color32::from_rgb(34, 38, 46), Color32::from_rgb(180, 190, 205))
    };

    let btn = egui::Button::new(
        RichText::new(title)
            .size(13.0)
            .color(text_color)
            .strong(),
    )
    .fill(bg_color)
    .corner_radius(CornerRadius::same(6))
    .min_size(Vec2::new(0.0, 28.0));

    ui.add(btn)
}
