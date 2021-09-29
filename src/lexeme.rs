#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Lexeme<'a> {
    Cell(&'a str),
    NewLine,
}
