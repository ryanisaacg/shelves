#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod client;
mod parser;
mod url;

use eframe::{
    egui,
    epaint::{Color32, Pos2},
};

use client::Client;
use url::Url;

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
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::new(Browser { text: contents }))),
    )
    .unwrap();

    Ok(())
}

struct Browser {
    text: String,
}

const WIDTH: f32 = 800.;
const HEIGHT: f32 = 600.;
const HSTEP: f32 = 13.;
const VSTEP: f32 = 8.;

impl eframe::App for Browser {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut pos = Pos2::ZERO;
            for ch in self.text.chars() {
                let rendered_char =
                    ui.painter()
                        .layout(ch.to_string(), Default::default(), Color32::WHITE, 800.0);
                ui.painter().galley(pos, rendered_char, Default::default());
                pos.x += HSTEP;
                if pos.x >= WIDTH {
                    pos.x = 0.;
                    pos.y += VSTEP;
                }
            }
        });
    }
}
