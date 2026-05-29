use eframe::egui::{self, CornerRadius, Margin, RichText, Sense, Vec2};

use crate::{
    app::{PlaymuApp, TrackAction},
    db::Track,
    icons::{icon_widget, Icon},
    models::{BrowseFocus, LibraryDensity, LibrarySortKey, LibraryView},
    theme::{
        ACCENT_GREEN, ACCENT_GREEN_SOFT, PANEL_SOFT, SURFACE, SURFACE_HOVER, TEXT_BRIGHT,
        TEXT_FAINT, TEXT_MUTED,
    },
    ui::{
        card_frame, paint_art_play,
        widgets::{
            add_to_playlist_button, draw_collection_card, draw_library_sort_cell,
            hover_highlight_row, nav_button,
        },
    },
    util::format_duration,
};

impl PlaymuApp {
    pub fn draw_library(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Collection").size(22.0).strong());
            ui.label(RichText::new(self.browse_focus.label()).color(TEXT_MUTED));
            if self.browse_focus != BrowseFocus::All && ui.button("Clear Focus").clicked() {
                self.clear_browse_focus();
            }
        });
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            for view in [
                LibraryView::Songs,
                LibraryView::Albums,
                LibraryView::Artists,
                LibraryView::Playlists,
            ] {
                if nav_button(ui, self.library_view == view, view.label()).clicked() {
                    self.library_view = view;
                }
                ui.add_space(6.0);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let dense_selected = self.library_density == LibraryDensity::Dense;
                if ui
                    .add(
                        egui::Button::new("Dense")
                            .fill(if dense_selected { ACCENT_GREEN_SOFT } else { SURFACE }),
                    )
                    .clicked()
                {
                    self.library_density = LibraryDensity::Dense;
                }
                let compact_selected = self.library_density == LibraryDensity::Compact;
                if ui
                    .add(
                        egui::Button::new("Compact")
                            .fill(if compact_selected { ACCENT_GREEN_SOFT } else { SURFACE }),
                    )
                    .clicked()
                {
                    self.library_density = LibraryDensity::Compact;
                }
            });
        });
        ui.add_space(12.0);

        if self.library_view == LibraryView::Playlists {
            self.draw_playlists(ui);
            return;
        }

        if self.library_view == LibraryView::Songs {
            self.draw_library_song_header(ui);
            ui.add_space(8.0);
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            match self.library_view {
                LibraryView::Songs => {
                    let visible_tracks = self.sorted_library_tracks();
                    for track in visible_tracks {
                        let highlighted = self.selected_track_id == Some(track.id)
                            || self.now_playing_track_id == Some(track.id);
                        self.draw_library_song_row(ui, &track, highlighted);
                    }
                }
                LibraryView::Albums => {
                    for album in self.library_albums() {
                        let response = draw_collection_card(
                            ui,
                            &album.title,
                            &album.artist,
                            &format!("{} tracks", album.track_count),
                            SURFACE,
                        );
                        if response.clicked() {
                            self.open_album(&album.artist, &album.title);
                        }
                        if response.double_clicked() {
                            self.play_track_list(album.track_ids.clone(), 0);
                        }
                        ui.add_space(6.0);
                    }
                }
                LibraryView::Artists => {
                    for artist in self.library_artists() {
                        let response = draw_collection_card(
                            ui,
                            &artist.name,
                            "Open artist focus or double click to queue all songs",
                            &format!(
                                "{} tracks across {} albums",
                                artist.track_count, artist.album_count
                            ),
                            SURFACE,
                        );
                        if response.clicked() {
                            self.open_artist(&artist.name);
                        }
                        if response.double_clicked() {
                            self.play_track_list(artist.track_ids.clone(), 0);
                        }
                        ui.add_space(6.0);
                    }
                }
                LibraryView::Playlists => {} // handled via early return above
            }

            if self.tracks.is_empty() || self.filtered_tracks().is_empty() {
                egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
                    ui.set_min_height(240.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(48.0);
                        let message = if self.tracks.is_empty() {
                            "No music indexed yet"
                        } else {
                            "Nothing matches the current browse focus"
                        };
                        ui.label(RichText::new(message).size(28.0).strong());
                        ui.label(
                            RichText::new(
                                "Import a local music folder from the left sidebar to get started, \
                                 or clear the active filter.",
                            )
                            .color(TEXT_MUTED),
                        );
                    });
                });
            }
        });
    }

    // -------------------------------------------------------------------------
    // Playlists view
    // -------------------------------------------------------------------------

    pub fn draw_playlists(&mut self, ui: &mut egui::Ui) {
        // Sidebar: list of playlists + create button.
        ui.columns(2, |cols| {
            // Left column — playlist list + create
            let col = &mut cols[0];
            col.horizontal(|ui| {
                ui.label(RichText::new("Your Playlists").size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new("+ New").color(ACCENT_GREEN).strong())
                        .clicked()
                    {
                        self.show_create_playlist = !self.show_create_playlist;
                    }
                });
            });
            col.add_space(8.0);

            if self.show_create_playlist {
                card_frame(SURFACE_HOVER).show(col, |ui| {
                    ui.label(RichText::new("New playlist name").color(TEXT_MUTED).small());
                    ui.add_space(4.0);
                    let te = ui.add(
                        egui::TextEdit::singleline(&mut self.new_playlist_name)
                            .hint_text("My Favourites")
                            .desired_width(f32::INFINITY),
                    );
                    if te.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.create_playlist();
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Create").color(egui::Color32::BLACK).strong(),
                                )
                                .fill(ACCENT_GREEN),
                            )
                            .clicked()
                        {
                            self.create_playlist();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_create_playlist = false;
                            self.new_playlist_name.clear();
                        }
                    });
                });
                col.add_space(8.0);
            }

            egui::ScrollArea::vertical().show(col, |ui| {
                let playlists = self.playlists.clone();
                if playlists.is_empty() {
                    ui.label(
                        RichText::new("No playlists yet.\nClick \"+ New\" to create one.")
                            .color(TEXT_FAINT),
                    );
                }
                for pl in &playlists {
                    let selected = self.active_playlist_id == Some(pl.id);
                    let fill = if selected { ACCENT_GREEN_SOFT } else { SURFACE };
                    let response = card_frame(fill)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new(&pl.name)
                                    .strong()
                                    .color(if selected { ACCENT_GREEN } else { TEXT_BRIGHT }),
                            );
                            let count = self
                                .playlists
                                .iter()
                                .position(|p| p.id == pl.id)
                                .map(|_| {
                                    crate::db::list_playlist_track_ids(&self.db_path, pl.id)
                                        .map(|v| v.len())
                                        .unwrap_or(0)
                                })
                                .unwrap_or(0);
                            ui.label(
                                RichText::new(format!("{count} tracks"))
                                    .color(TEXT_FAINT)
                                    .small(),
                            );
                        })
                        .response
                        .interact(egui::Sense::click());

                    if response.clicked() {
                        self.active_playlist_id = Some(pl.id);
                    }
                    if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    ui.add_space(6.0);
                }
            });

            // Right column — active playlist detail
            let col = &mut cols[1];
            let pv = self.active_playlist_view();
            match pv {
                None => {
                    col.add_space(32.0);
                    col.label(
                        RichText::new("Select a playlist to see its tracks.")
                            .color(TEXT_FAINT),
                    );
                }
                Some(pv) => {
                    col.horizontal(|ui| {
                        ui.label(
                            RichText::new(&pv.playlist.name)
                                .size(20.0)
                                .strong()
                                .color(TEXT_BRIGHT),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Delete Playlist").color(TEXT_MUTED),
                                    )
                                    .fill(egui::Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                self.delete_active_playlist();
                            }
                            ui.add_space(8.0);
                            if !pv.track_ids.is_empty()
                                && ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Play All")
                                                .color(egui::Color32::BLACK)
                                                .strong(),
                                        )
                                        .fill(ACCENT_GREEN),
                                    )
                                    .clicked()
                            {
                                self.play_track_list(pv.track_ids.clone(), 0);
                            }
                        });
                    });
                    col.label(
                        RichText::new(format!("{} tracks", pv.track_ids.len()))
                            .color(TEXT_FAINT)
                            .small(),
                    );
                    col.add_space(10.0);

                    // Collect tracks in playlist order
                    let pl_tracks: Vec<crate::db::Track> = pv
                        .track_ids
                        .iter()
                        .filter_map(|id| self.tracks.iter().find(|t| t.id == *id).cloned())
                        .collect();

                    if pl_tracks.is_empty() {
                        col.label(
                            RichText::new("This playlist is empty.\nAdd songs using the + button on any track.")
                                .color(TEXT_FAINT),
                        );
                    } else {
                        egui::ScrollArea::vertical().show(col, |ui| {
                            let active_pl_id = self.active_playlist_id;
                            for track in pl_tracks {
                                let highlighted = self.selected_track_id == Some(track.id)
                                    || self.now_playing_track_id == Some(track.id);
                                let playlists_snap = self.playlists.clone();
                                let mut play_art = false;
                                let mut remove_from_pl = false;
                                let mut playlist_add: Option<i64> = None;

                                let response = egui::Frame::new()
                                    .fill(if highlighted { ACCENT_GREEN_SOFT } else { SURFACE })
                                    .corner_radius(egui::CornerRadius::same(10))
                                    .inner_margin(egui::Margin::symmetric(12, 8))
                                    .show(ui, |ui| {
                                        ui.set_min_height(52.0);
                                        ui.horizontal(|ui| {
                                            let (_, clicked) = paint_art_play(ui, &track.album, None, 42.0, 8);
                                            play_art = clicked;
                                            ui.add_space(10.0);
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(&track.title)
                                                        .strong()
                                                        .color(TEXT_BRIGHT),
                                                );
                                                ui.label(
                                                    RichText::new(format!(
                                                        "{} - {}",
                                                        track.artist, track.album
                                                    ))
                                                    .color(TEXT_MUTED)
                                                    .small(),
                                                );
                                            });
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new("✕")
                                                                    .color(TEXT_MUTED),
                                                            )
                                                            .fill(egui::Color32::TRANSPARENT)
                                                            .min_size(Vec2::splat(22.0)),
                                                        )
                                                        .on_hover_text("Remove from playlist")
                                                        .clicked()
                                                    {
                                                        remove_from_pl = true;
                                                    }
                                                    if let Some(id) = add_to_playlist_button(
                                                        ui,
                                                        track.id,
                                                        &playlists_snap,
                                                        false,
                                                    ) {
                                                        playlist_add = Some(id);
                                                    }
                                                },
                                            );
                                        });
                                    })
                                    .response
                                    .interact(egui::Sense::click());

                                if response.hovered() {
                                    hover_highlight_row(ui, response.rect, 10);
                                }
                                if play_art || response.double_clicked() {
                                    let ids = pv.track_ids.clone();
                                    let start = ids
                                        .iter()
                                        .position(|id| *id == track.id)
                                        .unwrap_or(0);
                                    self.play_track_list(ids, start);
                                } else if response.clicked() {
                                    self.selected_track_id = Some(track.id);
                                }
                                if remove_from_pl {
                                    if let Some(pl_id) = active_pl_id {
                                        let _ = crate::db::remove_track_from_playlist(
                                            &self.db_path,
                                            pl_id,
                                            track.id,
                                        );
                                    }
                                }
                                if let Some(id) = playlist_add {
                                    self.add_track_to_playlist(id, track.id);
                                }
                                ui.add_space(4.0);
                            }
                        });
                    }
                }
            }
        });
    }

    pub fn draw_library_song_header(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_width(ui.available_width());

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.35, 30.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if draw_library_sort_cell(
                            ui,
                            self.library_sort_key,
                            self.library_sort_ascending,
                            LibrarySortKey::Title,
                        )
                        .clicked()
                        {
                            self.toggle_library_sort(LibrarySortKey::Title);
                        }
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.28, 30.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if draw_library_sort_cell(
                            ui,
                            self.library_sort_key,
                            self.library_sort_ascending,
                            LibrarySortKey::Artist,
                        )
                        .clicked()
                        {
                            self.toggle_library_sort(LibrarySortKey::Artist);
                        }
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.24, 30.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if draw_library_sort_cell(
                            ui,
                            self.library_sort_key,
                            self.library_sort_ascending,
                            LibrarySortKey::Album,
                        )
                        .clicked()
                        {
                            self.toggle_library_sort(LibrarySortKey::Album);
                        }
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 30.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if draw_library_sort_cell(
                            ui,
                            self.library_sort_key,
                            self.library_sort_ascending,
                            LibrarySortKey::Duration,
                        )
                        .clicked()
                        {
                            self.toggle_library_sort(LibrarySortKey::Duration);
                        }
                    },
                );
            });
        });
    }

    pub fn toggle_library_sort(&mut self, key: LibrarySortKey) {
        if self.library_sort_key == key {
            self.library_sort_ascending = !self.library_sort_ascending;
        } else {
            self.library_sort_key = key;
            self.library_sort_ascending = true;
        }
    }

    pub fn draw_library_song_row(&mut self, ui: &mut egui::Ui, track: &Track, highlighted: bool) {
        let duration_label = if track.duration_seconds > 0 {
            format_duration(track.duration_seconds)
        } else {
            "--:--".to_string()
        };

        let row_height = self.library_density.row_height();
        let gap = self.library_density.vertical_gap();
        let is_now_playing = self.now_playing_track_id == Some(track.id);
        let fill = if highlighted { ACCENT_GREEN_SOFT } else { SURFACE };
        let title_color = if is_now_playing { ACCENT_GREEN } else { TEXT_BRIGHT };
        let show_art = self.library_density == LibraryDensity::Dense;
        let playlists_snap = self.playlists.clone();
        let art_tex = self.get_art_texture(ui.ctx(), &track.artist, &track.album);

        let mut play_art_clicked = false;
        let mut playlist_chosen: Option<i64> = None;
        let mut ctx_action: Option<TrackAction> = None;

        let response = egui::Frame::new()
            .fill(fill)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(12, 4))
            .show(ui, |ui| {
                ui.set_min_height(row_height);
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width() * 0.35, row_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if show_art {
                                let art_size = row_height - 16.0;
                                let (_, clicked) = paint_art_play(
                                    ui,
                                    &track.album,
                                    art_tex.as_ref(),
                                    art_size,
                                    6,
                                );
                                play_art_clicked = clicked;
                                ui.add_space(10.0);
                            }
                            if is_now_playing {
                                icon_widget(ui, 14.0, Icon::Equalizer, ACCENT_GREEN);
                                ui.add_space(6.0);
                            }
                            ui.label(RichText::new(&track.title).strong().color(title_color));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width() * 0.28, row_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(&track.artist).color(TEXT_MUTED));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width() * 0.24, row_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(&track.album).color(TEXT_MUTED));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), row_height),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(&duration_label).color(TEXT_MUTED).small());
                            if let Some(id) =
                                add_to_playlist_button(ui, track.id, &playlists_snap, false)
                            {
                                playlist_chosen = Some(id);
                            }
                        },
                    );
                });
            })
            .response
            .interact(Sense::click());

        response.context_menu(|ui| {
            if ui.button("Play Now").clicked() {
                ctx_action = Some(TrackAction::PlayNow);
                ui.close_menu();
            }
            if ui.button("Add to Queue End").clicked() {
                ctx_action = Some(TrackAction::AddToQueueEnd);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Go to Album").clicked() {
                ctx_action = Some(TrackAction::GoToAlbum);
                ui.close_menu();
            }
            if ui.button("Go to Artist").clicked() {
                ctx_action = Some(TrackAction::GoToArtist);
                ui.close_menu();
            }
        });

        if response.hovered() {
            hover_highlight_row(ui, response.rect, 8);
        }
        let track_id = track.id;
        if play_art_clicked {
            self.selected_track_id = Some(track_id);
            self.play_selected_from_visible();
        } else if response.double_clicked() {
            self.selected_track_id = Some(track_id);
            self.play_selected_from_visible();
        } else if response.clicked() {
            self.selected_track_id = Some(track_id);
        }
        if let Some(pl_id) = playlist_chosen {
            self.add_track_to_playlist(pl_id, track_id);
        }
        if let Some(action) = ctx_action {
            self.handle_track_action(action, track_id);
        }
        ui.add_space(gap);
    }
}
