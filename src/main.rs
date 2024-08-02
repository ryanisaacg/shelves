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
    show(client.request(&input)?.as_str());

    Ok(())
}

fn show(html: &str) {
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => print!("{ch}"),
            _ => {}
        }
    }
}
