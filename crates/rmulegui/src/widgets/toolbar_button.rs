use eframe::egui::{
    widgets, Response, Sense, TextStyle, Widget, WidgetInfo, WidgetText, WidgetType,
};
use eframe::emath::NumExt;
use eframe::epaint::{pos2, Rect, Vec2};

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct ToolbarButton {
    text: WidgetText,
    image: Option<widgets::Image>,
}

impl ToolbarButton {
    pub fn new<S: Into<WidgetText>>(text: S) -> Self {
        Self {
            text: text.into(),
            image: None,
        }
    }
}

// enabled
// mouse pressed
// focus rectange
// image
impl Widget for ToolbarButton {
    fn ui(self, ui: &mut eframe::egui::Ui) -> Response {
        let button_padding = ui.spacing().button_padding;
        let text_wrap_width = ui.available_width() - 2.0 * button_padding.x;
        let text = self
            .text
            .into_galley(ui, Some(false), text_wrap_width, TextStyle::Button);
        let mut desired_size = text.size();
        desired_size += 2.0 * button_padding;
        desired_size = desired_size.at_least(Vec2::new(80.0, 40.0));

        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, text.text()));

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            let text_pos = if let Some(image) = self.image {
                let icon_spacing = ui.spacing().icon_spacing;
                pos2(
                    rect.min.x + button_padding.x + image.size().x + icon_spacing,
                    rect.center().y - 0.5 * text.size().y,
                )
            } else {
                ui.layout()
                    .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                    .min
            };

            text.paint_with_visuals(ui.painter(), text_pos, visuals);

            if let Some(image) = self.image {
                let image_rect = Rect::from_min_size(
                    pos2(rect.min.x, rect.center().y - 0.5 - (image.size().y / 2.0)),
                    image.size(),
                );
                image.paint_at(ui, image_rect);
            }
        }

        response
    }
}
