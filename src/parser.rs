use crate::Lexeme;

/// The helper struct for tracking a position in the file
struct Tracker<'a> {
    rest: &'a str,
    pos: usize,
}

impl<'a> Iterator for Tracker<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.rest.chars().next()?;
        let len = ch.len_utf8();
        self.rest = &self.rest[len..];
        self.pos += len;
        Some(ch)
    }
}

pub struct Parser<'a> {
    input: &'a str,
    tracker: Tracker<'a>,
    lex_start: usize,
    running: bool,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            tracker: Tracker {
                rest: input,
                pos: 0,
            },
            lex_start: 0,
            running: true,
        }
    }

    fn next(&mut self) -> Option<Result<Lexeme<'a>, usize>> {
        const QUOTE: char = '"';

        let tracker = &mut self.tracker;

        loop {
            self.lex_start = tracker.pos;
            match tracker.next()? {
                '\n' => return Some(Ok(Lexeme::NewLine)),
                ch if ch.is_whitespace() => continue,
                QUOTE => break,
                _ => return Some(Err(self.lex_start)),
            }
        }

        Some(match tracker.position(|ch| ch == QUOTE) {
            None => Err(self.lex_start),
            Some(len) => {
                let start = self.lex_start + QUOTE.len_utf8();
                Ok(Lexeme::Cell(&self.input[start..start + len]))
            }
        })
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Lexeme<'a>, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.running {
            let lex = Self::next(self)?;
            self.running = lex.is_ok();
            Some(lex)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let mut parser = Parser::new("");
        assert!(parser.next().is_none());
    }

    #[test]
    fn parse_empty_cell() {
        let parser = Parser::new(r#" """" "#);
        let parsed: Vec<_> = parser.map(Result::unwrap).collect();
        assert_eq!(parsed, [Lexeme::Cell(""), Lexeme::Cell("")]);
    }

    #[test]
    fn parse_one() {
        let parser = Parser::new(r#" "hi" "#);
        let parsed: Vec<_> = parser.map(Result::unwrap).collect();
        assert_eq!(parsed, [Lexeme::Cell("hi")]);
    }

    #[test]
    fn parse_two() {
        let parser = Parser::new(r#" "hi" "fi" "#);
        let parsed: Vec<_> = parser.map(Result::unwrap).collect();
        assert_eq!(parsed, [Lexeme::Cell("hi"), Lexeme::Cell("fi")]);
    }

    #[test]
    fn parse_nl() {
        let parser = Parser::new("\n \n\n");
        let parsed: Vec<_> = parser.map(Result::unwrap).collect();
        assert_eq!(parsed, [Lexeme::NewLine, Lexeme::NewLine, Lexeme::NewLine,]);
    }

    #[test]
    fn parse_two_nl() {
        let parser = Parser::new(
            r#" "hi"
                "fi"
            "#,
        );

        let parsed: Vec<_> = parser.map(Result::unwrap).collect();
        assert_eq!(
            parsed,
            [
                Lexeme::Cell("hi"),
                Lexeme::NewLine,
                Lexeme::Cell("fi"),
                Lexeme::NewLine,
            ]
        );
    }

    #[test]
    fn parse_error_start() {
        let parser = Parser::new("...");
        let parsed: Vec<_> = parser.collect();
        assert_eq!(parsed, [Err(0)]);
    }

    #[test]
    fn parse_error_end() {
        let parser = Parser::new("\"...");
        let parsed: Vec<_> = parser.collect();
        assert_eq!(parsed, [Err(0)]);
    }
}
