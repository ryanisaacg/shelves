use std::sync::Arc;

use eframe::{
    egui::{text::LayoutJob, Color32, FontFamily, FontId, Galley, TextFormat, Ui},
    epaint::Pos2,
};

use crate::parser::Token;

pub const HSTEP: f32 = 13.;
pub const VSTEP: f32 = 15.;

pub enum FormatToken {
    Text { layout: LayoutJob },
    Linebreak,
}

pub fn format_tokens(tokens: &[Token]) -> Vec<FormatToken> {
    let mut italics = false;
    let mut bold = false;
    let mut size = 16.0;

    let mut format_tokens = Vec::new();

    for token in tokens.iter() {
        match token {
            Token::Tag(tag) => match tag.as_str() {
                "i" => italics = true,
                "/i" => italics = false,
                "b" => bold = true,
                "/b" => bold = false,
                "br" | "/br" | "/p" => {
                    format_tokens.push(FormatToken::Linebreak);
                }
                "small" => size -= 2.0,
                "/small" => size += 2.0,
                "big" => size += 4.0,
                "/big" => size -= 4.0,
                _ => {}
            },
            Token::Word(word) => {
                if word.is_empty() {
                    continue;
                }
                let mut job = LayoutJob::default();
                // TODO: bold isn't
                job.append(
                    word,
                    0.,
                    TextFormat {
                        font_id: FontId::new(size, FontFamily::Proportional),
                        // TODO: this is no good, don't hardcode colors
                        color: if bold {
                            Color32::WHITE
                        } else {
                            Color32::LIGHT_GRAY
                        },
                        italics,
                        ..Default::default()
                    },
                );
                format_tokens.push(FormatToken::Text { layout: job });
            }
        }
    }

    format_tokens
}

pub struct DisplayListItem {
    pub pos: Pos2,
    pub galley: Arc<Galley>,
}

pub fn layout(ui: &Ui, tokens: &[FormatToken]) -> Vec<DisplayListItem> {
    let mut display_list = Vec::new();
    let mut line_buffer = Vec::new();

    let mut cursor = Pos2::new(0., 0.);
    for token in tokens.iter() {
        match token {
            FormatToken::Text { layout } => {
                let font = layout
                    .sections
                    .first()
                    .map(|section| section.format.font_id.clone())
                    .unwrap_or_default();
                let space = ui
                    .painter()
                    .layout_no_wrap(" ".to_string(), font, Color32::BLACK);
                let galley = ui.painter().layout_job(layout.clone());
                let word_width = galley.rect.width();
                if cursor.x + word_width > ui.min_rect().width() {
                    flush(&mut line_buffer, &mut display_list, &mut cursor);
                }
                line_buffer.push(DisplayListItem {
                    pos: cursor,
                    galley,
                });
                cursor.x += word_width + space.rect.width();
            }
            FormatToken::Linebreak => {
                flush(&mut line_buffer, &mut display_list, &mut cursor);
            }
        }
    }

    flush(&mut line_buffer, &mut display_list, &mut cursor);

    display_list
}

fn flush(
    line_buffer: &mut Vec<DisplayListItem>,
    display_list: &mut Vec<DisplayListItem>,
    cursor: &mut Pos2,
) {
    cursor.x = 0.0;
    let max_ascent = line_buffer
        .iter()
        .filter_map(|item| galley_max_ascent(&item.galley))
        .reduce(f32::max)
        .unwrap_or(0.0);
    let baseline = cursor.y + 1.25 * max_ascent;
    let max_descent = line_buffer
        .iter()
        .map(|item| item.galley.mesh_bounds.bottom() - item.galley.mesh_bounds.center().y)
        .reduce(f32::max)
        .unwrap_or(0.0);
    for mut word in line_buffer.drain(..) {
        word.pos.y = baseline - galley_max_ascent(&word.galley).unwrap_or(0.0);
        display_list.push(word);
    }
    cursor.y = baseline + 1.25 * max_descent;
    cursor.y += VSTEP;
}

fn galley_max_ascent(galley: &Galley) -> Option<f32> {
    galley
        .rows
        .iter()
        .flat_map(|row| row.glyphs.iter())
        .map(|glyph| glyph.ascent)
        .reduce(f32::max)
}
