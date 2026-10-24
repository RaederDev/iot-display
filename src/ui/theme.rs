use crate::config::Config;
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily};

pub const BLUE: Color32 = Color32::from_rgb(5, 102, 141);

pub fn install(context: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Roboto".into(),
        FontData::from_static(include_bytes!("../../assets/Roboto-Regular.ttf")).into(),
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Roboto".into());
    for registration in crate::widgets::registry::REGISTERED {
        if let Some(install) = registration.fonts {
            install(&mut fonts);
        }
    }
    context.set_fonts(fonts);
    egui_extras::install_image_loaders(context);
}

pub fn apply(context: &egui::Context, config: &Config) {
    context.set_zoom_factor(config.scale);
    let mut visuals = if config.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = if config.dark {
        Color32::from_rgb(35, 35, 35)
    } else {
        Color32::from_rgb(238, 238, 238)
    };
    context.set_visuals(visuals);
    context.all_styles_mut(|style| {
        style.spacing.interact_size.y = 32.0;
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    });
}
