use crate::url::Url;

mod url;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    args.next().unwrap(); // discard binary name
    let url = args.next().unwrap();

    let example = Url::new(url);
    show(example.request()?.as_str());

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
