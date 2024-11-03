use logos::Logos;
use thiserror::Error;

pub trait Searchable {
    fn matches_text(&self, name: &str) -> Option<u32>;
    fn matches_tag(&self, key: &str, value: &str) -> Option<u32>;
    fn matches_cmp(&self, lhs: &str, op: &str, rhs: &str) -> Option<u32>;
}

#[derive(Logos, Debug)]
#[logos(skip r"[\p{White_Space}\t\n\f]+")]
pub enum Token<'a> {
    #[token("-")]
    Not,
    #[token("or")]
    Or,
    #[token("(")]
    StartGroup,
    #[token(")")]
    EndGroup,
    #[regex("=|(>=)|(<=)|<|>", |lex| lex.slice())]
    Comparison(&'a str),
    #[regex("[a-z]+:[a-z0-9]+", |lex| lex.slice().split_once(":"))]
    Tag((&'a str, &'a str)),
    #[regex("[^-=<>()\\p{White_Space}][^=<>()\\p{White_Space}]*", |lex| lex.slice())]
    Text(&'a str),
    #[regex("\"([^\"\\\\]*(\\\\.[^\"\\\\]*)*)\"")]
    QuotedText(&'a str),
}

#[derive(Debug)]
enum Node {
    Group(Vec<Self>),
    OrGroup(Vec<Self>),
    Not(Box<Self>),
    Comparison(String, String, String),
    Tag(String, String),
    Text(String),
}

#[derive(Debug)]
enum ParseResult<'a> {
    Ok(Node),
    Unconsumed(Token<'a>),
    Eof,
    Err(SyntaxError<'a>),
}

#[derive(Debug, Error)]
pub enum SyntaxError<'a> {
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token<'a>),
}

fn parse_node(s: &str) -> Result<Node, SyntaxError<'_>> {
    let mut lexer = Token::lexer(s);
    match parse_group(&mut lexer, true) {
        ParseResult::Ok(node) => Ok(node),
        ParseResult::Unconsumed(token) => Err(SyntaxError::UnexpectedToken(token)),
        ParseResult::Eof => Err(SyntaxError::UnexpectedEof),
        ParseResult::Err(err) => Err(err),
    }
}

fn parse_next<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>) -> ParseResult<'a> {
    if let Some(token) = lexer.next() {
        match token {
            Ok(Token::Not) => {
                let next = parse_next(lexer);
                return if let ParseResult::Ok(node) = next {
                    ParseResult::Ok(Node::Not(Box::new(node)))
                } else {
                    next
                };
            }
            Ok(Token::Tag((key, value))) => {
                return ParseResult::Ok(Node::Tag(key.to_string(), value.to_string()));
            }
            Ok(Token::Text(text)) => {
                return ParseResult::Ok(Node::Text(text.to_ascii_lowercase().to_string()));
            }
            Ok(Token::QuotedText(text)) => {
                return ParseResult::Ok(Node::Text(
                    text.trim_matches('"')
                        .replace("\\\"", "\"")
                        .to_ascii_lowercase()
                        .to_string(),
                ));
            }
            Ok(Token::StartGroup) => {
                return parse_group(lexer, false);
            }
            Ok(token) => return ParseResult::Unconsumed(token),
            _ => (),
        }
    }
    ParseResult::Eof
}

fn parse_group<'a>(lexer: &mut logos::Lexer<'a, Token<'a>>, is_root: bool) -> ParseResult<'a> {
    let mut nodes = vec![];
    loop {
        match parse_next(lexer) {
            ParseResult::Ok(node) => nodes.push(node),
            ParseResult::Unconsumed(Token::EndGroup) => {
                if is_root {
                    return ParseResult::Err(SyntaxError::UnexpectedToken(Token::EndGroup));
                } else {
                    break;
                }
            }
            ParseResult::Unconsumed(Token::Comparison(op)) => {
                if let Some(Node::Text(lhs)) = nodes.pop() {
                    match lexer.next() {
                        Some(Ok(Token::Text(text))) => {
                            let rhs = text.to_string();
                            let op = op.to_string();
                            nodes.push(Node::Comparison(lhs, op, rhs));
                        }
                        Some(Ok(token)) => {
                            return ParseResult::Err(SyntaxError::UnexpectedToken(token))
                        }
                        _ => return ParseResult::Err(SyntaxError::UnexpectedEof),
                    }
                } else {
                    return ParseResult::Err(SyntaxError::UnexpectedToken(Token::Or));
                }
            }
            ParseResult::Unconsumed(Token::Or) => {
                if let Some(lhs) = nodes.pop() {
                    let mut or_nodes = vec![];
                    match lhs {
                        Node::OrGroup(nodes) => or_nodes.extend(nodes),
                        _ => or_nodes.push(lhs),
                    }

                    let mut or_arg = true;
                    let mut break_outer = false;
                    loop {
                        match parse_next(lexer) {
                            ParseResult::Ok(node) => {
                                if or_arg {
                                    match node {
                                        Node::OrGroup(nodes) => or_nodes.extend(nodes),
                                        _ => or_nodes.push(node),
                                    }
                                } else {
                                    nodes.push(Node::OrGroup(or_nodes));
                                    nodes.push(node);
                                    break;
                                }
                                or_arg = false;
                            }
                            ParseResult::Unconsumed(Token::Comparison(op)) => {
                                if let Some(Node::Text(lhs)) = or_nodes.pop() {
                                    match lexer.next() {
                                        Some(Ok(Token::Text(text))) => {
                                            let rhs = text.to_string();
                                            let op = op.to_string();
                                            or_nodes.push(Node::Comparison(lhs, op, rhs));
                                        }
                                        Some(Ok(token)) => {
                                            return ParseResult::Err(SyntaxError::UnexpectedToken(
                                                token,
                                            ))
                                        }
                                        _ => return ParseResult::Err(SyntaxError::UnexpectedEof),
                                    }
                                } else {
                                    return ParseResult::Err(SyntaxError::UnexpectedToken(
                                        Token::Or,
                                    ));
                                }
                            }
                            ParseResult::Eof => {
                                if or_arg {
                                    return ParseResult::Err(SyntaxError::UnexpectedEof);
                                } else {
                                    nodes.push(Node::OrGroup(or_nodes));
                                    break;
                                }
                            }
                            ParseResult::Unconsumed(Token::EndGroup) => {
                                if is_root {
                                    return ParseResult::Err(SyntaxError::UnexpectedToken(
                                        Token::EndGroup,
                                    ));
                                } else {
                                    nodes.push(Node::OrGroup(or_nodes));
                                    break_outer = true;
                                    break;
                                }
                            }
                            ParseResult::Unconsumed(Token::Or) => {
                                or_arg = true;
                            }
                            ParseResult::Unconsumed(token) => {
                                return ParseResult::Err(SyntaxError::UnexpectedToken(token));
                            }
                            ParseResult::Err(err) => return ParseResult::Err(err),
                        }
                    }
                    if break_outer {
                        break;
                    }
                } else {
                    return ParseResult::Err(SyntaxError::UnexpectedToken(Token::Or));
                }
            }
            ParseResult::Unconsumed(token) => return ParseResult::Unconsumed(token),
            ParseResult::Eof => {
                if is_root {
                    break;
                } else {
                    return ParseResult::Err(SyntaxError::UnexpectedEof);
                }
            }
            ParseResult::Err(err) => return ParseResult::Err(err),
        }
    }
    if nodes.len() == 1 {
        ParseResult::Ok(nodes.remove(0))
    } else {
        ParseResult::Ok(Node::Group(nodes))
    }
}

#[derive(Debug)]
pub struct Filter(Node);

impl Filter {
    pub fn new(s: &str) -> Result<Self, SyntaxError> {
        Ok(Self(parse_node(s)?))
    }

    pub fn search<T, I>(&self, iter: I) -> impl Iterator<Item = T>
    where
        I: IntoIterator<Item = T>,
        T: Searchable,
    {
        let mut results = iter
            .into_iter()
            .filter_map(|item| self.0.matches(&item).map(|score| (item, score)))
            .collect::<Vec<_>>();
        results.sort_by_key(|(_, score)| *score);
        results.into_iter().map(|(item, _)| item)
    }
}

impl Node {
    fn matches<T>(&self, item: &T) -> Option<u32>
    where
        T: Searchable,
    {
        match self {
            Node::Group(nodes) => nodes
                .iter()
                .try_fold(0, |acc, node| node.matches(item).map(|score| score + acc)),
            Node::OrGroup(nodes) => nodes.iter().filter_map(|node| node.matches(item)).max(),
            Node::Not(node) => node.matches(item).xor(Some(1)),
            Node::Text(text) => item.matches_text(text),
            Node::Tag(key, value) => item.matches_tag(key, value),
            Node::Comparison(lhs, op, rhs) => item.matches_cmp(lhs, op, rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Searchable for &str {
        fn matches_text(&self, name: &str) -> Option<u32> {
            if self.to_ascii_lowercase().contains(name) {
                Some(1)
            } else {
                None
            }
        }

        fn matches_tag(&self, key: &str, value: &str) -> Option<u32> {
            let num: u32 = value.parse().ok()?;
            if key == "len" && self.len() == num as usize {
                Some(1)
            } else {
                None
            }
        }

        fn matches_cmp(&self, lhs: &str, op: &str, rhs: &str) -> Option<u32> {
            let num: u32 = rhs.parse().ok()?;
            let len = self.len() as u32;
            if lhs != "len" {
                return None;
            }
            match op {
                "=" => (len == num).then_some(1),
                ">" => (len > num).then_some(1),
                "<" => (len < num).then_some(1),
                ">=" => (len >= num).then_some(1),
                "<=" => (len <= num).then_some(1),
                _ => None,
            }
        }
    }

    #[test]
    fn test_search() {
        let result = Filter::new("foo or bar")
            .unwrap()
            .search(vec!["foo", "bar", "baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo", "bar"]);

        let result = Filter::new("foo bar")
            .unwrap()
            .search(vec!["foo bar", "barfoo", "foo baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo bar", "barfoo"]);

        let result = Filter::new("foo -bar")
            .unwrap()
            .search(vec!["foo bar", "barfoo", "foo baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo baz"]);

        let result = Filter::new("foo or -bar")
            .unwrap()
            .search(vec!["foo bar", "barfoo", "foo baz", "bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo bar", "barfoo", "foo baz"]);

        let result = Filter::new("foo-bar")
            .unwrap()
            .search(vec!["foo-bar", "barfoo", "foo baz", "bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo-bar"]);

        let result = Filter::new("\"foo bar\"")
            .unwrap()
            .search(vec!["foo bar", "barfoo", "foo baz", "bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo bar"]);

        let result = Filter::new("")
            .unwrap()
            .search(vec!["foo bar", "barfoo", "foo baz", "bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo bar", "barfoo", "foo baz", "bar baz"]);

        let result = Filter::new("-(bar or baz or foo)")
            .unwrap()
            .search(vec!["foo bar", "barfoo", "f00 f00", "bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["f00 f00"]);

        let result = Filter::new("len:6 or baz")
            .unwrap()
            .search(vec!["foo bar", "barfoo", "foo baz", "bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["barfoo", "foo baz", "bar baz"]);

        let result = Filter::new("len>3 len <= 7")
            .unwrap()
            .search(vec!["foo", "barfoo", "foo baz", "foo bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["barfoo", "foo baz"]);

        let result = Filter::new("foo -(len = 9 or len = 7) or len = 0 baz")
            .unwrap()
            .search(vec!["foo", "barfoo", "foo baz", "foo bar", "foo bar baz"])
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["foo bar baz"]);
    }
}
