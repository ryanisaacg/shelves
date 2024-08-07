use std::sync::Arc;

use eframe::{
    egui::{text::LayoutJob, Color32, FontFamily, FontId, Galley, Stroke, TextFormat, Ui},
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
                        font_id: FontId::new(12.0, FontFamily::Proportional),
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

    let mut pos = Pos2::new(0., 0.);
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
                // TODO: padding
                if pos.x + word_width > ui.min_rect().width() {
                    pos.x = 0.;
                    pos.y += galley.rect.height();
                    display_list.push(DisplayListItem { pos, galley });
                } else {
                    display_list.push(DisplayListItem { pos, galley });
                    pos.x += word_width + space.rect.width();
                }
            }
            FormatToken::Linebreak => {
                // TODO: wrong height
                pos.x = 0.;
                pos.y += VSTEP;
            }
        }
    }

    display_list
}
