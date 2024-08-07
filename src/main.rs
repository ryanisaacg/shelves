#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod client;
mod layout;
mod parser;
mod url;

use eframe::{
    egui::{self, Event, MouseWheelUnit, Vec2},
    epaint::Color32,
};

use client::Client;
use layout::{DisplayListItem, VSTEP};
use url::Url;

use crate::layout::calculate_draw_list;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    args.next().unwrap(); // discard binary name
    let url = match args.next() {
        Some(url) => url,
        None => {
            let dir = std::env::current_dir()?;
            format!("file://{}/test.html", dir.to_str().unwrap())
        }
    };

    let input = Url::new(url)?;
    let mut client = Client::new();
    let resp = client.request(&input)?;

    eprintln!("{:?}", resp.headers);
    let contents = parser::lex(resp.body.as_str()?)?.trim().to_string();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WIDTH, HEIGHT]),
        ..Default::default()
    };

    let draw_list = calculate_draw_list(contents.as_str(), WIDTH);

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Browser {
                draw_list,
                scroll: Vec2::ZERO,
            }))
        }),
    )
    .unwrap();

    Ok(())
}

struct Browser {
    draw_list: Vec<DisplayListItem>,
    scroll: Vec2,
}

const WIDTH: f32 = 800.;
const HEIGHT: f32 = 600.;

impl eframe::App for Browser {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for display in self.draw_list.iter() {
                let rendered_char = ui.painter().layout(
                    display.ch.to_string(),
                    Default::default(),
                    Color32::WHITE,
                    800.0,
                );
                ui.painter()
                    .galley(display.pos + self.scroll, rendered_char, Default::default());
            }
        });
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        for event in raw_input.events.drain(..) {
            match event {
                Event::MouseWheel {
                    unit,
                    delta,
                    modifiers: _,
                } => {
                    let px = match unit {
                        MouseWheelUnit::Point => delta,
                        MouseWheelUnit::Line => delta * VSTEP,
                        MouseWheelUnit::Page => todo!(),
                    };
                    self.scroll += px;
                }
                Event::Key { key, pressed, .. } => {
                    if !pressed {
                        continue;
                    }
                    match key {
                        egui::Key::ArrowDown => self.scroll.y -= VSTEP,
                        egui::Key::ArrowUp => self.scroll.y += VSTEP,
                        _ => {}
                    }
                }
                Event::Copy
                | Event::Cut
                | Event::Paste(_)
                | Event::Text(_)
                | Event::PointerMoved(_)
                | Event::MouseMoved(_)
                | Event::PointerButton { .. }
                | Event::PointerGone
                | Event::Zoom(_)
                | Event::Ime(_)
                | Event::Touch { .. }
                | Event::WindowFocused(_)
                | Event::AccessKitActionRequest(_)
                | Event::Screenshot { .. } => {}
            }
        }
    }
}
