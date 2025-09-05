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

struct StreamLookAhead<T, I>
where
        T: PartialEq,
        I: Iterator<Item = T>
{
        it: I,
        buffer_size: usize,
        buffer: Vec<Option<T>>,
        position: Position,
        delimiter: T,
}

impl<T, I> StreamLookAhead<T, I>
where
        T: PartialEq,
        I: Iterator<Item = T>
{
    pub fn new(mut it: I, lhn: usize, delimiter: T) -> Self {
        let position = Position::new(0, 0, 0);
        let mut buffer = Vec::new();
        (0..lhn).for_each(|_| buffer.push(it.next()));
        StreamLookAhead {
            it,
            buffer_size: lhn,
            buffer,
            position,
            delimiter
        }
    }

    pub fn advance(&mut self) -> Option<T> {
        self.buffer.rotate_left(1);
        let item = self.it.next();
        self.buffer[self.buffer_size - 1] = item;
        item
    }
}
