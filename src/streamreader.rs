#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub(self) seq: usize,
    pub(self) row: usize,
    pub(self) col: usize,
}

impl Position {
    pub fn new(seq: usize, row: usize, col: usize) -> Self {
        Position { seq, row, col }
    }

    pub fn seq(&self) -> usize {
        self.seq
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

pub trait StreamReader<T: PartialEq> {
    fn delimiter(&self) -> T;

    fn advance(&mut self) -> () ;
    fn advance_and_read(&mut self) -> Option<T> {
        self.advance();
        self.current_token()
    }
    fn read_and_advance(&mut self) -> Option<T> {
        let token = self.current_token();
        self.advance();
        token
    }
    fn advance_if(&mut self, f: impl Fn(&T) -> bool) -> Option<T> {
        let Some(next_token) = self.next_token() else {
            return None;
        };

        if f(&next_token) {
            self.advance_and_read()
        }
        else {
            None
        }
    }

    fn current_token(&self) -> Option<T> ;
    fn current_token_ref(&self) -> &Option<T> ;
    fn current_position(&self) -> Option<Position> ;

    fn next_token(&mut self) -> Option<T> ;
    fn next_token_ref(&mut self) -> Option<&T> ;
    fn next_position(&mut self) -> Option<Position> {
        let current_position = self.current_position();
        let current_token = self.current_token();
        let next_token = self.next_token();
        match (current_token, current_position, next_token) {
            (Some(current_token), Some(current_position), Some(_)) => {
                let new_line = current_token == self.delimiter();
                let seq = current_position.seq + 1;
                let row = current_position.row + if new_line { 1 } else { 0 };
                let col = if new_line { 0 } else { current_position.col + 1 };
                Some(Position::new(seq, row, col))
            },
            _ => None,
        }
    }
}




use std::iter::Peekable;

pub struct GenericStreamReader<T, I>
where
    T: PartialEq,
    I: Iterator,
{
    it: Peekable<I>,
    token: Option<T>,
    position: Option<Position>,
    delimiter: T,
}

impl<T, I> GenericStreamReader<T, I>
where
    T: PartialEq,
    I: Iterator<Item = T>,
{
    pub fn new(i: I, delimiter: T) -> Self {
        let mut it = i.peekable();
        let token = it.next();
        let position = if token.is_none() { None } else { Some(Position::new(0, 0, 0)) };
        GenericStreamReader {
            it,
            token,
            position,
            delimiter,
        }
    }
}

impl<T, I> StreamReader<T> for GenericStreamReader<T, I>
where
    T: PartialEq + Clone,
    I: Iterator<Item = T>,
{
    fn delimiter(&self) -> T {
        self.delimiter.clone()
    }

    fn advance(&mut self) -> () {
        self.position = self.next_position();
        self.token = self.it.next();
    }

    fn current_token(&self) -> Option<T> {
        self.token.clone()
    }

    fn current_token_ref(&self) -> &Option<T>  {
        &self.token
    }

    fn current_position(&self) -> Option<Position> {
        self.position
    }

    fn next_token(&mut self) -> Option<T> {
        self.it.peek().cloned()
    }

    fn next_token_ref(&mut self) -> Option<&T>  {
        self.it.peek()
    }
}





use std::str::Chars;
use std::vec::IntoIter;
pub type CharStreamReader<'a> = GenericStreamReader<char, Chars<'a>>;
pub type StringStreamReader<'a> = GenericStreamReader<String, IntoIter<String>>;
pub type PositionedStringStreamReader<'a> = GenericStreamReader<(String, Position), IntoIter<(String, Position)>>;
