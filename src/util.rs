use std::collections::{HashMap, HashSet};

use eframe::egui::{self, Color32, FontId, TextFormat};

use crate::{
    db::Track,
    models::{AlbumSummary, ArtistSummary},
};

pub fn format_duration(duration_seconds: i64) -> String {
    let minutes = duration_seconds / 60;
    let seconds = duration_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

pub fn highlight_match_job(
    text: &str,
    query: &str,
    base_color: Color32,
    strong: bool,
) -> egui::text::LayoutJob {
    use crate::theme::ACCENT_GREEN;

    let mut job = egui::text::LayoutJob::default();
    let base_format = TextFormat {
        color: base_color,
        font_id: if strong {
            FontId::proportional(16.0)
        } else {
            FontId::proportional(14.0)
        },
        ..Default::default()
    };

    let trimmed_query = query.trim();
    if trimmed_query.is_empty() {
        job.append(text, 0.0, base_format);
        return job;
    }

    let highlight_format = TextFormat {
        color: Color32::WHITE,
        background: ACCENT_GREEN,
        font_id: if strong {
            FontId::proportional(16.0)
        } else {
            FontId::proportional(14.0)
        },
        ..Default::default()
    };

    let lower_text = text.to_ascii_lowercase();
    let lower_query = trimmed_query.to_ascii_lowercase();

    let mut cursor = 0;
    while let Some(found_at) = lower_text[cursor..].find(&lower_query) {
        let start = cursor + found_at;
        let end = start + lower_query.len();
        if start > cursor {
            job.append(&text[cursor..start], 0.0, base_format.clone());
        }
        job.append(&text[start..end], 0.0, highlight_format.clone());
        cursor = end;
    }
    if cursor < text.len() {
        job.append(&text[cursor..], 0.0, base_format);
    }
    job
}

pub fn summarize_albums<'a>(tracks: impl IntoIterator<Item = &'a Track>) -> Vec<AlbumSummary> {
    let mut grouped: HashMap<(String, String), Vec<&Track>> = HashMap::new();
    for track in tracks {
        grouped
            .entry((track.artist.clone(), track.album.clone()))
            .or_default()
            .push(track);
    }

    let mut albums: Vec<AlbumSummary> = grouped
        .into_iter()
        .map(|((artist, title), tracks)| AlbumSummary {
            title,
            artist,
            track_count: tracks.len(),
            track_ids: tracks.into_iter().map(|t| t.id).collect(),
        })
        .collect();

    albums.sort_by(|a, b| a.artist.cmp(&b.artist).then_with(|| a.title.cmp(&b.title)));
    albums
}

pub fn summarize_artists<'a>(tracks: impl IntoIterator<Item = &'a Track>) -> Vec<ArtistSummary> {
    let mut grouped: HashMap<String, Vec<&Track>> = HashMap::new();
    for track in tracks {
        grouped.entry(track.artist.clone()).or_default().push(track);
    }

    let mut artists: Vec<ArtistSummary> = grouped
        .into_iter()
        .map(|(name, tracks)| {
            let album_count = tracks
                .iter()
                .map(|t| t.album.to_ascii_lowercase())
                .collect::<HashSet<_>>()
                .len();
            ArtistSummary {
                name,
                track_count: tracks.len(),
                album_count,
                track_ids: tracks.into_iter().map(|t| t.id).collect(),
            }
        })
        .collect();

    artists.sort_by(|a, b| a.name.cmp(&b.name));
    artists
}
