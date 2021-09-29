use std::fmt;

pub struct Table<'a> {
    cols_width: Vec<usize>,
    rows: Vec<&'a str>,
    rows_len: usize,
}

impl<'a> Table<'a> {
    pub fn new() -> Self {
        Self {
            cols_width: vec![],
            rows: vec![],
            rows_len: 0,
        }
    }

    pub fn head<H>(mut self, header: H) -> Self
    where
        H: IntoIterator<Item = &'a str>,
    {
        assert!(self.rows.is_empty());
        self.rows = header.into_iter().collect();
        self.cols_width = self.rows.iter().map(|row| row.chars().count()).collect();
        self
    }

    pub fn tail<R>(mut self, row: R) -> Self
    where
        R: IntoIterator<Item = &'a str>,
        R::IntoIter: ExactSizeIterator,
    {
        let row = row.into_iter();
        assert_eq!(row.len(), self.cols_len());
        self.rows_len += 1;

        for (idx, cell) in row.enumerate() {
            let width = &mut self.cols_width[idx];
            *width = cell.chars().count().max(*width);
            self.rows.push(cell);
        }

        self
    }

    pub fn cols_len(&self) -> usize {
        self.cols_width.len()
    }

    pub fn rows_len(&self) -> usize {
        self.rows_len
    }
}

impl fmt::Display for Table<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.rows.is_empty() {
            return Ok(());
        }

        let mut rows = self.rows.chunks(self.cols_len());
        let header = rows.next().unwrap();
        for (cell, width) in header.iter().zip(&self.cols_width) {
            write!(f, "| {:width$} ", cell, width = width)?;
        }
        writeln!(f, "|")?;

        if self.rows_len() == 1 {
            return Ok(());
        }

        for &width in &self.cols_width {
            write!(f, "|")?;
            for _ in 0..width + 2 {
                write!(f, "-")?;
            }
        }
        writeln!(f, "|")?;

        for row in rows {
            for (cell, width) in row.iter().zip(&self.cols_width) {
                write!(f, "| {:width$} ", cell, width = width)?;
            }
            writeln!(f, "|")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let table = Table::new()
            .head(["one", "two", "three"])
            .tail(["four", "five", "six"])
            .tail(["seven", "eight", "nine"]);

        assert_eq!(
            table.to_string(),
            "\
            | one   | two   | three |\n\
            |-------|-------|-------|\n\
            | four  | five  | six   |\n\
            | seven | eight | nine  |\n\
            "
        );
    }
}
