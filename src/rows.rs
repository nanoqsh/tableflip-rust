use crate::Lexeme;
use std::{cell::RefCell, iter::Peekable, rc::Rc};

struct Inner<I>
where
    I: Iterator,
{
    iter: Peekable<I>,
    n_cols: usize,
}

impl<I> Inner<I>
where
    I: Iterator,
{
    fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            n_cols: 0,
        }
    }
}

/// The iterator wrapper for iterating over rows
pub struct Rows<I>
where
    I: Iterator,
{
    inner: Option<Inner<I>>,
}

impl<I> Rows<I>
where
    I: Iterator,
{
    pub fn new<T>(iter: T) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: Some(Inner::new(iter.into_iter())),
        }
    }

    pub fn split(mut self) -> (Head<I>, Tail<I>) {
        let inner = self.inner.take();
        let rows = Rc::new(RefCell::new(self));
        let head = Head {
            rows: Rc::clone(&rows),
            inner,
        };
        let tail = Tail { rows, inner: None };
        (head, tail)
    }
}

pub struct Head<I>
where
    I: Iterator,
{
    rows: Rc<RefCell<Rows<I>>>,
    inner: Option<Inner<I>>,
}

impl<'a, I> Iterator for Head<I>
where
    I: Iterator<Item = Lexeme<'a>>,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.inner.as_mut().unwrap();
        match inner.iter.next() {
            Some(Lexeme::Cell(cell)) => {
                inner.n_cols += 1;
                Some(cell)
            }
            Some(Lexeme::NewLine) => None,
            None => None,
        }
    }
}

impl<I> Drop for Head<I>
where
    I: Iterator,
{
    fn drop(&mut self) {
        let mut rows = self.rows.borrow_mut();
        rows.inner = self.inner.take();
    }
}

pub struct Tail<I>
where
    I: Iterator,
{
    rows: Rc<RefCell<Rows<I>>>,
    inner: Option<Inner<I>>,
}

impl<I> Tail<I>
where
    I: Iterator,
{
    pub fn row(&mut self) -> Option<TailRow<I>> {
        let inner = match &mut self.inner {
            None => {
                let mut rows = self.rows.try_borrow_mut().expect("Use head iterator first");
                self.inner = rows.inner.take();
                self.inner.as_mut().unwrap()
            }
            Some(inner) => inner,
        };

        let cols_left = inner.n_cols;
        if inner.iter.peek().is_none() {
            None
        } else {
            Some(TailRow::new(self, cols_left))
        }
    }
}

enum TailRowState {
    Iterate,
    Default,
    Done,
}

pub struct TailRow<'t, I>
where
    I: Iterator,
{
    tail: &'t mut Tail<I>,
    cols_left: usize,
    state: TailRowState,
}

impl<'a, 't, I> TailRow<'t, I>
where
    I: Iterator,
{
    fn new(tail: &'t mut Tail<I>, cols_left: usize) -> Self {
        Self {
            tail,
            cols_left,
            state: TailRowState::Iterate,
        }
    }

    fn next(&mut self) -> Option<&'a str>
    where
        I: Iterator<Item = Lexeme<'a>>,
    {
        let inner = self.tail.inner.as_mut().unwrap();
        match self.state {
            TailRowState::Iterate => match inner.iter.next() {
                Some(Lexeme::Cell(cell)) => {
                    if self.cols_left == 1 {
                        // If iterating is not ended,
                        // iterate and ignore the rest part.
                        loop {
                            match inner.iter.next() {
                                Some(Lexeme::Cell(_)) => (),
                                Some(Lexeme::NewLine) => break,
                                None => break,
                            }
                        }
                    }

                    Some(cell)
                }
                Some(Lexeme::NewLine) => {
                    if self.cols_left == 0 {
                        self.state = TailRowState::Done;
                        None
                    } else {
                        self.state = TailRowState::Default;
                        Some("")
                    }
                }
                None => None,
            },
            TailRowState::Default => match self.cols_left {
                0 => None,
                _ => Some(""),
            },
            TailRowState::Done => None,
        }
    }
}

impl<'a, I> Iterator for TailRow<'_, I>
where
    I: Iterator<Item = Lexeme<'a>>,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cols_left == 0 {
            return None;
        }

        let res = Self::next(self);
        self.cols_left -= 1;
        res
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.cols_left;
        (left, Some(left))
    }
}

impl<'a, I> ExactSizeIterator for TailRow<'_, I> where I: Iterator<Item = Lexeme<'a>> {}

/// Drop implementation checks an iterator was fully used.
/// This is useful to prevent infinite loop and incomplete row usage.
impl<I> Drop for TailRow<'_, I>
where
    I: Iterator,
{
    fn drop(&mut self) {
        if self.cols_left != 0 && !std::thread::panicking() {
            panic!("The iterator must be fully used")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterator() {
        let table = [
            Lexeme::Cell("a"),
            Lexeme::Cell("b"),
            Lexeme::Cell("c"),
            Lexeme::NewLine,
            Lexeme::Cell("0"),
            Lexeme::Cell("1"),
            Lexeme::NewLine,
            Lexeme::Cell("2"),
            Lexeme::Cell("3"),
            Lexeme::Cell("4"),
            Lexeme::Cell("5"),
            Lexeme::NewLine,
            Lexeme::NewLine,
        ];

        let rows = Rows::new(table);
        let (head, mut tail) = rows.split();

        let head: Vec<_> = head.collect();
        assert_eq!(head, ["a", "b", "c"]);

        if let Some(row) = tail.row() {
            let tail: Vec<_> = row.collect();
            assert_eq!(tail, ["0", "1", ""]);
        };

        if let Some(row) = tail.row() {
            let tail: Vec<_> = row.collect();
            assert_eq!(tail, ["2", "3", "4"]);
        };

        if let Some(row) = tail.row() {
            let tail: Vec<_> = row.collect();
            assert_eq!(tail, ["", "", ""]);
        };

        assert!(tail.row().is_none());
    }
}
