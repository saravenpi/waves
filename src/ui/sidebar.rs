use eframe::egui;
use std::path::PathBuf;

use crate::app::WavesApp;
use crate::config::SidebarPosition;
use crate::types::{FileEntry, SidebarView};
use crate::ui::components::{IconButton, Select};
use crate::utils::{truncate_text, format_duration_option};

pub struct SidebarEvents {
    pub clicked_entry: Option<(usize, FileEntry)>,
    pub back_button_clicked: bool,
    pub context_menu_event: Option<(PathBuf, egui::Pos2)>,
}

pub fn render_sidebar(
    app: &mut WavesApp,
    ctx: &egui::Context,
    current_playing_file: Option<PathBuf>,
) -> egui::InnerResponse<SidebarEvents> {
    let sidebar_panel = match app.config.sidebar_position {
        SidebarPosition::Left => egui::SidePanel::left("file_browser"),
        SidebarPosition::Right => egui::SidePanel::right("file_browser"),
    };

    #[cfg(target_os = "macos")]
    let sidebar_margin = egui::Margin { left: 10.0, right: 10.0, top: 40.0, bottom: 10.0 };

    #[cfg(not(target_os = "macos"))]
    let sidebar_margin = egui::Margin::same(10.0);

    #[cfg(target_os = "macos")]
    let min_width = 100.0;

    #[cfg(not(target_os = "macos"))]
    let min_width = 250.0;

    let sidebar_panel_configured = sidebar_panel
        .resizable(true)
        .default_width(app.config.sidebar_width)
        .width_range(min_width..=800.0)
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgb(16, 16, 16))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64)))
            .inner_margin(sidebar_margin));

    sidebar_panel_configured.show(ctx, |ui| {
        ui.add_space(10.0);
        ui.heading(egui::RichText::new("Waves").size(32.0).color(egui::Color32::WHITE).strong());
        ui.add_space(10.0);

        let full_height = ui.available_height();
        let browser_height = full_height - 20.0;

        let mut events = SidebarEvents {
            clicked_entry: None,
            back_button_clicked: false,
            context_menu_event: None,
        };

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), browser_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                render_sidebar_tabs(app, ui);
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);

                let header_height = render_sidebar_header(app, ui, &mut events);
                let list_height = browser_height - header_height;

                match app.sidebar_view {
                    SidebarView::FileBrowser => {
                        render_file_browser(app, ui, list_height, &current_playing_file, &mut events);
                    }
                    SidebarView::Liked => {
                        render_liked(app, ui, list_height, &current_playing_file, &mut events, ctx);
                    }
                    SidebarView::Settings => {
                        crate::ui::settings::render_settings(app, ui, list_height);
                    }
                }
            }
        );

        events
    })
}

fn render_sidebar_tabs(app: &mut WavesApp, ui: &mut egui::Ui) {
    let mut clicked_sidebar_view = None;

    let sidebar_options = vec![
        ("📁".to_string(), "Browser".to_string()),
        ("❤".to_string(), "Liked".to_string()),
        ("⚙".to_string(), "Settings".to_string()),
    ];

    let sidebar_index = match app.sidebar_view {
        SidebarView::FileBrowser => 0,
        SidebarView::Liked => 1,
        SidebarView::Settings => 2,
    };

    let primary_color = app.primary_color();

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), 30.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add_space(2.0);

            let (_, clicked) = Select::new(sidebar_options, sidebar_index)
                .show(ui, primary_color);
            clicked_sidebar_view = clicked;
        });

    if let Some(idx) = clicked_sidebar_view {
        match idx {
            0 => app.sidebar_view = SidebarView::FileBrowser,
            1 => app.sidebar_view = SidebarView::Liked,
            2 => app.sidebar_view = SidebarView::Settings,
            _ => {}
        };
    }
}

fn render_sidebar_header(app: &mut WavesApp, ui: &mut egui::Ui, events: &mut SidebarEvents) -> f32 {
    let mut header_height = 75.0;

    match app.sidebar_view {
        SidebarView::FileBrowser => {
            render_navigation_breadcrumb(app, ui, events);
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);
            render_browsing_mode_selector(app, ui);
            ui.add_space(5.0);
            header_height = 110.0;
        }
        _ => {}
    }

    header_height
}

fn render_navigation_breadcrumb(app: &WavesApp, ui: &mut egui::Ui, events: &mut SidebarEvents) {
    use crate::types::BrowsingMode;
    use crate::types::GroupedView;

    let folder_name = match app.browsing_mode {
        BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
            match &app.grouped_view {
                GroupedView::TrackList(group_name) => {
                    group_name.trim_start_matches("🎤 ").trim_start_matches("💿 ").to_string()
                }
                GroupedView::GroupList => {
                    if app.browsing_mode == BrowsingMode::ByArtist {
                        "All Artists".to_string()
                    } else {
                        "All Albums".to_string()
                    }
                }
            }
        }
        BrowsingMode::AllSongs => "All Songs".to_string(),
        BrowsingMode::FileStructure => {
            app.current_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("/")
                .to_string()
        }
    };

    let show_back = match app.browsing_mode {
        BrowsingMode::FileStructure => app.current_dir != app.root_dir,
        BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
            matches!(app.grouped_view, GroupedView::TrackList(_))
        }
        BrowsingMode::AllSongs => false,
    };

    ui.horizontal(|ui| {
        ui.add_space(2.0);

        if show_back {
            let text_color = egui::Color32::from_rgb(150, 150, 150);
            let back_response = IconButton::new("<").size(14.0).color(text_color).show(ui);

            if back_response.clicked() {
                events.back_button_clicked = true;
            }

            if back_response.hovered() {
                ui.painter().rect_stroke(
                    back_response.rect,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64)),
                );
            }

            ui.add_space(5.0);
        }

        ui.label(
            egui::RichText::new(folder_name)
                .size(14.0)
                .color(egui::Color32::from_rgb(150, 150, 150))
        );
    });
}

fn render_browsing_mode_selector(app: &mut WavesApp, ui: &mut egui::Ui) {
    use crate::types::BrowsingMode;

    let browsing_options = vec![
        ("📂".to_string(), "Folders".to_string()),
        ("🎤".to_string(), "Artists".to_string()),
        ("💿".to_string(), "Albums".to_string()),
        ("🎵".to_string(), "All Songs".to_string()),
    ];

    let browsing_index = match app.browsing_mode {
        BrowsingMode::FileStructure => 0,
        BrowsingMode::ByArtist => 1,
        BrowsingMode::ByAlbum => 2,
        BrowsingMode::AllSongs => 3,
    };

    let (_, clicked_browsing) = Select::new(browsing_options, browsing_index)
        .show(ui, app.primary_color());

    if let Some(idx) = clicked_browsing {
        let new_mode = match idx {
            0 => BrowsingMode::FileStructure,
            1 => BrowsingMode::ByArtist,
            2 => BrowsingMode::ByAlbum,
            3 => BrowsingMode::AllSongs,
            _ => app.browsing_mode,
        };
        if new_mode != app.browsing_mode {
            app.browsing_mode = new_mode;
            app.grouped_view = crate::types::GroupedView::GroupList;
            app.current_group_tracks.clear();
            app.update_columns_with_selection(Some(0));
        }
    }
}

struct ListItemConfig<'a> {
    path: &'a PathBuf,
    name: &'a str,
    is_dir: bool,
    is_selected: bool,
    is_liked: bool,
    is_in_clipboard: bool,
    duration: &'a Option<std::time::Duration>,
}

fn render_list_item(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    config: ListItemConfig,
    current_playing_file: &Option<PathBuf>,
    primary_color: egui::Color32,
    primary_color_with_alpha: egui::Color32,
    max_chars: usize,
) -> egui::Color32 {
    let is_playing_or_parent = if let Some(playing_path) = current_playing_file {
        if playing_path == config.path {
            true
        } else if config.is_dir {
            playing_path.starts_with(config.path)
        } else {
            false
        }
    } else {
        false
    };

    let is_hovered = response.hovered();

    let name_has_emoji = config.name.starts_with("🎤 ")
        || config.name.starts_with("💿 ")
        || config.name.starts_with("🎵 ");

    let icon = if name_has_emoji {
        ""
    } else if config.is_dir {
        "📁"
    } else if config.is_liked {
        "❤"
    } else {
        "🎵"
    };

    let duration_text = if !config.is_dir {
        format_duration_option(*config.duration)
    } else {
        String::new()
    };

    let duration_width = if !duration_text.is_empty() {
        duration_text.len() + 2
    } else {
        0
    };

    let available_chars_for_name = max_chars.saturating_sub(4 + duration_width);
    let display_name = truncate_text(config.name, available_chars_for_name);

    let display_text = if icon.is_empty() {
        format!(" {}", display_name)
    } else {
        format!(" {} {}", icon, display_name)
    };

    let color = if config.is_selected {
        ui.painter().rect_filled(
            rect,
            0.0,
            primary_color_with_alpha,
        );
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, primary_color),
        );
        primary_color
    } else {
        if is_playing_or_parent {
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0, primary_color),
            );
        } else if is_hovered {
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64)),
            );
        }

        if config.is_in_clipboard {
            egui::Color32::from_rgb(100, 100, 100)
        } else {
            egui::Color32::WHITE
        }
    };

    ui.painter().text(
        rect.left_center() + egui::vec2(4.0, 0.0),
        egui::Align2::LEFT_CENTER,
        &display_text,
        egui::FontId::monospace(18.0),
        color,
    );

    if !duration_text.is_empty() {
        ui.painter().text(
            rect.right_center() - egui::vec2(10.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            &duration_text,
            egui::FontId::monospace(14.0),
            egui::Color32::from_rgb(120, 120, 120),
        );
    }

    color
}

fn render_file_browser(
    app: &mut WavesApp,
    ui: &mut egui::Ui,
    list_height: f32,
    current_playing_file: &Option<PathBuf>,
    events: &mut SidebarEvents,
) {
    if !app.columns.is_empty() {
        let mut files_to_extract = Vec::new();

        {
            let column = &app.columns[0];

            let scroll_area = egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .min_scrolled_height(list_height)
                .max_height(list_height)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .id_salt("file_browser_scroll");

            scroll_area.show(ui, |ui| {
                let available_width = ui.available_width();
                let max_chars = ((available_width - 10.0) / 10.5) as usize;

                if !column.entries.is_empty() && (column.entries[0].name.starts_with("Loading ")) {
                    let loading_text = &column.entries[0].name;
                    ui.add_space(50.0);
                    crate::ui::spinner::square_spinner_with_text(ui, loading_text, app.primary_color());
                    return;
                }

                for (idx, entry) in column.entries.iter().enumerate() {
                    let is_selected = idx == column.selected;

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(available_width, 25.0),
                        egui::Sense::click()
                    );

                    if is_selected && app.scroll_to_selection {
                        ui.scroll_to_rect(rect, Some(egui::Align::Center));
                    }

                    if response.clicked() {
                        events.clicked_entry = Some((idx, entry.clone()));
                    }

                    if response.secondary_clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            events.context_menu_event = Some((entry.path.clone(), pos));
                        }
                    }

                    let is_liked = app.liked.iter().any(|f| f.path == entry.path);
                    let is_in_clipboard = app.clipboard.as_ref()
                        .map(|(path, _)| path == &entry.path)
                        .unwrap_or(false);

                    let duration = if !entry.is_dir {
                        if let Some(cached_duration) = app.duration_cache.get(&entry.path) {
                            Some(*cached_duration)
                        } else if !app.duration_extraction_in_progress.contains(&entry.path) {
                            files_to_extract.push(entry.path.clone());
                            None
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    render_list_item(
                        ui,
                        rect,
                        &response,
                        ListItemConfig {
                            path: &entry.path,
                            name: &entry.name,
                            is_dir: entry.is_dir,
                            is_selected,
                            is_liked,
                            is_in_clipboard,
                            duration: &duration,
                        },
                        current_playing_file,
                        app.primary_color(),
                        app.primary_color_with_alpha(13),
                        max_chars,
                    );
                }
            });
        }

        for path in files_to_extract {
            app.duration_extraction_in_progress.insert(path.clone());
            let sender = app.duration_sender.clone();
            std::thread::spawn(move || {
                let duration = crate::metadata::duration::extract_duration(&path);
                let _ = sender.send((path, duration));
            });
        }
    }
}

fn render_empty_liked_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.label(
            egui::RichText::new("No liked tracks yet")
                .size(16.0)
                .color(egui::Color32::from_rgb(150, 150, 150))
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("Press 'f' to like audio files")
                .size(14.0)
                .color(egui::Color32::from_rgb(120, 120, 120))
        );
    });
}

fn render_liked(
    app: &mut WavesApp,
    ui: &mut egui::Ui,
    list_height: f32,
    current_playing_file: &Option<PathBuf>,
    events: &mut SidebarEvents,
    ctx: &egui::Context,
) {
    let selected = app.liked_selected;
    let mut clicked_liked: Option<usize> = None;
    let mut context_menu_event: Option<(PathBuf, egui::Pos2)> = None;

    if app.liked.is_empty() {
        render_empty_liked_state(ui);
    } else {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .min_scrolled_height(list_height)
            .max_height(list_height)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .id_salt("liked_scroll")
            .show(ui, |ui| {
                let available_width = ui.available_width();
                let max_chars = ((available_width - 10.0) / 10.5) as usize;

                for (idx, fav) in app.liked.iter().enumerate() {
                    let is_selected = idx == selected;

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(available_width, 25.0),
                        egui::Sense::click()
                    );

                    if is_selected && app.scroll_to_selection {
                        ui.scroll_to_rect(rect, Some(egui::Align::Center));
                    }

                    if response.clicked() {
                        clicked_liked = Some(idx);
                    }

                    if response.secondary_clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            context_menu_event = Some((fav.path.clone(), pos));
                        }
                    }

                    render_list_item(
                        ui,
                        rect,
                        &response,
                        ListItemConfig {
                            path: &fav.path,
                            name: &fav.name,
                            is_dir: fav.is_dir,
                            is_selected,
                            is_liked: true,
                            is_in_clipboard: false,
                            duration: &None,
                        },
                        current_playing_file,
                        app.primary_color(),
                        app.primary_color_with_alpha(13),
                        max_chars,
                    );
                }
            });

        if let Some((path, pos)) = context_menu_event {
            events.context_menu_event = Some((path, pos));
        }

        if let Some(idx) = clicked_liked {
            app.liked_selected = idx;
            if let Some(fav) = app.liked.get(idx) {
                let path = fav.path.clone();
                let is_dir = fav.is_dir;
                if is_dir {
                    app.current_dir = path;
                    app.update_columns_with_selection(Some(0));
                    app.sidebar_view = SidebarView::FileBrowser;
                } else {
                    app.playback_context = SidebarView::Liked;
                    app.play_file(&path, ctx);
                }
            }
        }
    }
}

pub fn handle_sidebar_events(app: &mut WavesApp, events: SidebarEvents, ctx: &egui::Context) {
    if let Some((idx, entry)) = events.clicked_entry {
        app.columns[0].selected = idx;

        if entry.is_dir {
            app.current_dir = entry.path.clone();
            app.update_columns_with_selection(Some(0));
        } else {
            app.playback_context = SidebarView::FileBrowser;
            app.play_file(&entry.path, ctx);
        }
    }

    if let Some((path, pos)) = events.context_menu_event {
        app.context_menu = Some((path, pos));
    }

    if events.back_button_clicked {
        use crate::types::{BrowsingMode, GroupedView};

        match app.browsing_mode {
            BrowsingMode::FileStructure => {
                if let Some(parent) = app.current_dir.parent() {
                    if parent >= app.root_dir.as_path() {
                        app.current_dir = parent.to_path_buf();
                        app.update_columns_with_selection(Some(0));
                    }
                }
            }
            BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                if matches!(app.grouped_view, GroupedView::TrackList(_)) {
                    app.grouped_view = GroupedView::GroupList;
                    app.current_group_tracks.clear();
                    app.update_columns_with_selection(Some(0));
                }
            }
            BrowsingMode::AllSongs => {}
        }
    }
}

pub fn update_sidebar_width(app: &mut WavesApp, ctx: &egui::Context, sidebar_response: &egui::InnerResponse<SidebarEvents>) {
    #[cfg(target_os = "macos")]
    let min_sidebar_width = 100.0;

    #[cfg(not(target_os = "macos"))]
    let min_sidebar_width = 250.0;

    if ctx.input(|i| i.pointer.any_down()) {
        let new_width = sidebar_response.response.rect.width().max(min_sidebar_width).min(800.0);
        if (new_width - app.config.sidebar_width).abs() > 1.0 {
            app.config.sidebar_width = new_width;
            let _ = app.config.save();
        }
    }
}
