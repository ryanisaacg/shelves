use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub enum Token {
    Tag(String),
    Word(String),
}

pub fn lex(html: &str) -> Vec<Token> {
    let mut results = Vec::new();

    let mut state = ParseState::Text;
    let mut buffer = String::new();

    for grapheme in UnicodeSegmentation::graphemes(html, true) {
        if state == ParseState::InTag {
            if grapheme == ">" {
                results.push(Token::Tag(buffer.clone()));
                buffer.clear();
                state = ParseState::Text;
            } else {
                buffer.push_str(grapheme);
            }
        } else {
            match grapheme {
                "<" => {
                    results.push(Token::Word(buffer.clone()));
                    buffer.clear();
                    state = ParseState::InTag;
                }
                "&" => {
                    results.push(Token::Word(buffer.clone()));
                    buffer.clear();
                    state = ParseState::EscapeSequence;
                }
                ";" if state == ParseState::EscapeSequence => {
                    results.push(Token::Word(match buffer.as_str() {
                        "lt" => "<".to_string(),
                        "gt" => ">".to_string(),
                        not_recognized => format!("&{buffer};"),
                    }));
                    buffer.clear();
                    state = ParseState::Text;
                }
                _ if state == ParseState::EscapeSequence => {
                    if grapheme.trim().is_empty() {
                        results.push(Token::Word(format!("&{buffer}")));
                        buffer.clear();
                        state = ParseState::Text;
                    } else {
                        buffer.push_str(grapheme);
                    }
                }
                _ => {
                    if grapheme.trim().is_empty() {
                        results.push(Token::Word(buffer.clone()));
                        buffer.clear();
                    } else {
                        buffer.push_str(grapheme);
                    }
                }
            }
        }
    }

    results
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ParseState {
    InTag,
    EscapeSequence,
    Text,
}
