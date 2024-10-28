use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section<'a> {
    Text(&'a str),
    Card(&'a str),
    Keyword(&'a str),
    Number(i32),
}

pub fn parse_text(text: &str) -> Vec<Section> {
    let mut sections = vec![];
    let mut lexer = Token::lexer(text);
    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::StartCardName) => {
                if let Some(Ok(Token::Text(card))) = lexer.next() {
                    sections.push(Section::Card(card));
                }
                lexer.next();
            }
            Ok(Token::StartKeyword) => {
                if let Some(Ok(Token::Text(keyword))) = lexer.next() {
                    sections.push(Section::Keyword(keyword));
                }
                lexer.next();
            }
            Ok(Token::Number(number)) => sections.push(Section::Number(number)),
            Ok(Token::Text(text)) => sections.push(Section::Text(text)),
            _ => {}
        }
    }
    sections
}

#[derive(Logos, Debug, PartialEq)]
#[logos()]
enum Token<'source> {
    #[token("<<")]
    StartCardName,
    #[token(">>")]
    EndCardName,
    #[token("[[")]
    StartKeyword,
    #[token("]]")]
    EndKeyword,
    #[regex("[+-]?[0-9]+", |lex| lex.slice().parse::<i32>().unwrap())]
    Number(i32),
    #[regex("([^+\\-<>\\[\\]0-9][^<>\\[\\]][ ]?)+", |lex| lex.slice())]
    Text(&'source str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let text = "This is a <<card>> with [[ability]] and +500 power.";
        let sections = parse_text(text);
        assert_eq!(
            sections,
            vec![
                Section::Text("This is a "),
                Section::Card("card"),
                Section::Text(" with "),
                Section::Keyword("ability"),
                Section::Text(" and "),
                Section::Number(500),
                Section::Text(" power.")
            ]
        );
    }
}
