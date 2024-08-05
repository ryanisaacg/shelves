use client::Client;

use crate::url::Url;

mod client;
mod url;

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

    println!("{:?}", resp.headers);
    show(resp.body.as_str()?);

    Ok(())
}

fn show(html: &str) {
    let mut state = ParseState::InTag;
    let mut escape_sequence = String::new();

    for ch in html.chars() {
        if state == ParseState::InTag {
            if ch == '>' {
                state = ParseState::Text;
            }
        } else {
            match ch {
                '<' => {
                    state = ParseState::InTag;
                }
                '&' => {
                    state = ParseState::EscapeSequence;
                }
                ';' if state == ParseState::EscapeSequence => {
                    match escape_sequence.as_str() {
                        "lt" => print!("<"),
                        "gt" => print!(">"),
                        not_recognized => print!("&{};", not_recognized),
                    }
                    escape_sequence.clear();
                    state = ParseState::Text;
                }
                _ if state == ParseState::EscapeSequence => {
                    escape_sequence.push(ch);
                }
                _ => print!("{ch}"),
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ParseState {
    InTag,
    EscapeSequence,
    Text,
}
