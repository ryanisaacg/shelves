use std::fmt::Write;

pub fn lex(html: &str) -> Result<String, std::fmt::Error> {
    let mut result_buffer = String::new();

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
                        "lt" => write!(result_buffer, "<"),
                        "gt" => write!(result_buffer, ">"),
                        not_recognized => write!(result_buffer, "&{};", not_recognized),
                    }?;
                    escape_sequence.clear();
                    state = ParseState::Text;
                }
                _ if state == ParseState::EscapeSequence => {
                    escape_sequence.push(ch);
                }
                _ => {
                    write!(result_buffer, "{ch}")?;
                }
            }
        }
    }

    Ok(result_buffer)
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ParseState {
    InTag,
    EscapeSequence,
    Text,
}
