mod config;
mod types;
mod favorites;
mod metadata;
mod audio;
mod file_operations;
mod ui;
mod utils;
mod album_cover;
mod app;
mod macos;
mod cursor_sound;
mod startup_sound;
mod delete_sound;
mod update;

use app::WavesApp;
use config::Config;
use eframe::egui;
use std::sync::mpsc;

/// Entry point for the WAVES music player application.
///
/// Initializes the GUI window with custom fonts, styling, and configuration.
/// Loads application icon and sets up the eframe viewport with user preferences.
fn main() -> eframe::Result {
    cursor_sound::init_sound_system();

    let config = Config::load();

    let (file_open_sender, file_open_receiver) = mpsc::channel();

    #[cfg(target_os = "macos")]
    let (menu_action_sender, menu_action_receiver) = mpsc::channel();
    #[cfg(not(target_os = "macos"))]
    let _menu_action_sender: mpsc::Sender<()> = mpsc::channel().0;

    #[cfg(target_os = "macos")]
    {
        macos::setup_file_open_handler(file_open_sender.clone());
        macos::setup_menu_bar(menu_action_sender);
    }

    let icon_data = include_bytes!("../waves_logo.png");
    let icon_image = image::load_from_memory(icon_data).ok();

    let icon = icon_image.and_then(|img| {
        let rgba = img.to_rgba8();
        Some(egui::IconData {
            rgba: rgba.to_vec(),
            width: img.width(),
            height: img.height(),
        })
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 700.0])
        .with_title("Waves")
        .with_decorations(config.decorations)
        .with_transparent(true);

    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    if config.window_corner_radius > 0.0 {
        viewport = viewport.with_window_level(egui::viewport::WindowLevel::Normal);
    }

    let file_open_sender_for_builder = file_open_sender.clone();

    let options = eframe::NativeOptions {
        viewport,
        event_loop_builder: Some(Box::new(move |builder| {
            #[cfg(target_os = "macos")]
            {
                use winit::platform::macos::EventLoopBuilderExtMacOS;
                let _sender = file_open_sender_for_builder.clone();
                builder.with_default_menu(false);
                builder.with_activate_ignoring_other_apps(true);
            }
        })),
        ..Default::default()
    };

    eframe::run_native(
        "Waves",
        options,
        Box::new(move |cc| {
            let mut fonts = egui::FontDefinitions::default();

            if let Some(custom_font_path) = config.custom_font.as_ref() {
                let expanded = shellexpand::tilde(custom_font_path).to_string();
                if let Ok(font_data) = std::fs::read(&expanded) {
                    fonts.font_data.insert(
                        "custom".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                    );

                    fonts.families
                        .get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .insert(0, "custom".to_owned());

                    fonts.families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "custom".to_owned());
                }
            }

            let system_font_paths: Vec<&str> = vec![
                #[cfg(target_os = "macos")]
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
                #[cfg(target_os = "macos")]
                "/Library/Fonts/Arial Unicode.ttf",
                #[cfg(target_os = "macos")]
                "/System/Library/Fonts/Helvetica.ttc",
                #[cfg(target_os = "linux")]
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                #[cfg(target_os = "linux")]
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
                #[cfg(target_os = "windows")]
                "C:\\Windows\\Fonts\\arial.ttf",
            ];

            for font_path in &system_font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "unicode_fallback".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                    );

                    fonts.families
                        .get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .push("unicode_fallback".to_owned());

                    fonts.families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .push("unicode_fallback".to_owned());

                    break;
                }
            }

            cc.egui_ctx.set_fonts(fonts);

            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.override_text_color = Some(egui::Color32::WHITE);
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(18.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::monospace(18.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(18.0),
            );
            cc.egui_ctx.set_style(style);

            cc.egui_ctx.options_mut(|o| o.warn_on_id_clash = false);

            #[cfg(target_os = "macos")]
            {
                Ok(Box::new(WavesApp::new_with_receiver(
                    file_open_receiver,
                    menu_action_receiver,
                )))
            }
            #[cfg(not(target_os = "macos"))]
            {
                Ok(Box::new(WavesApp::new_with_receiver(
                    file_open_receiver,
                )))
            }
        }),
    )
}
