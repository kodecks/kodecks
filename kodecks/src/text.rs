use regex_lite::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy)]
pub enum Section<'a> {
    Text(&'a str),
    Card(&'a str),
    Keyword(&'a str),
    Number(i32),
}

pub fn parse_text(text: &str) -> Vec<Section> {
    static REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?:<<(.+?)>>)|(?:\[\[(.+?)\]\])|(?:([+-]?[0-9]+))").unwrap());

    let mut sections = vec![];
    let mut cursor = 0;
    for capture in REGEX.captures_iter(text) {
        let prefix = &text[cursor..capture.get(0).unwrap().start()];
        if !prefix.is_empty() {
            sections.push(Section::Text(prefix));
        }
        if let Some(card) = capture.get(1) {
            sections.push(Section::Card(card.as_str()));
        } else if let Some(ability) = capture.get(2) {
            sections.push(Section::Keyword(ability.as_str()));
        } else if let Some(ability) = capture.get(3) {
            sections.push(Section::Number(ability.as_str().parse().unwrap()));
        }

        cursor = capture.get(0).unwrap().end();
    }
    let suffix = &text[cursor..];
    if !suffix.is_empty() {
        sections.push(Section::Text(suffix));
    }
    sections
}
