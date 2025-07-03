pub trait Reader {
    fn up(&self) -> bool ;
    fn peek_current_pos(&self) -> Option<(usize, usize, usize)> ;
    fn peek_current_ch(&self) -> Option<char> ;
    fn peek_next_pos(&mut self) -> Option<(usize, usize, usize)> ;
    fn peek_next_ch(&mut self) -> Option<char> ;
}

pub struct SimpleReader<'a> {
    it: std::iter::Peekable<std::str::Chars<'a>>,
    ch: Option<char>,
    cur: usize,
    col: usize,
    lin: usize,
}

impl<'a> SimpleReader<'a> {
    pub fn new(src: &'a str) -> Self {
        SimpleReader { it: src.chars().peekable(), ch: None, cur: 0, col: 0, lin: 0 }
    }

    fn get_next_pos(&self, cur_ch: char, next_ch: char, pos: (usize, usize, usize)) -> (usize, usize, usize) {
        let line_change = cur_ch == '\n' && next_ch != '\n';
        let col = if line_change { 0 } else { pos.0 + 1};
        let lin = if line_change { pos.1 + 1 } else { pos.1 };
        let cur = pos.2 + 1;
        (col, lin, cur)
    }

    fn advance(&mut self) -> Option<char> {
        let prev_ch = self.ch;
        let next_ch = self.it.next();

        if prev_ch.is_none() {
            (self.col, self.lin, self.cur) = (0, 0, 0);
        }
        else if next_ch.is_some() {
            let prev_ch = prev_ch.unwrap();
            let next_ch = next_ch.unwrap();
            let prev_pos = self.peek_current_pos().unwrap();
            (self.col, self.lin, self.cur) = self.get_next_pos(prev_ch, next_ch, prev_pos);
        }

        self.ch = next_ch;
        self.ch
    }
}

impl<'a> Reader for SimpleReader<'a> {
    fn up(&self) -> bool {
        self.ch.is_some()
    }

    fn peek_current_pos(&self) -> Option<(usize, usize, usize)> {
        if self.up() {
            Some((self.col, self.lin, self.cur))
        }
        else {
            None
        }
    }

    fn peek_current_ch(&self) -> Option<char> {
        self.ch
    }

    fn peek_next_pos(&mut self) -> Option<(usize, usize, usize)> {
        let opt = self.peek_next_ch();
        if let Some(next_ch) = opt {
            let prev_pos = self.peek_current_pos().unwrap();
            let prev_ch = self.peek_current_ch().unwrap();
            Some(self.get_next_pos(prev_ch, next_ch, prev_pos))
        }
        else {
            None
        }
    }

    fn peek_next_ch(&mut self) -> Option<char> {
        self.it.peek().copied()
    }
}

impl<'a> std::iter::Iterator for SimpleReader<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance()
    }
}

// pub fn enunciate_chars(s: &str) {
//     let mut reader = Reader::new(s);
//     while reader.up() {
//         let cur_ch = reader.current_char().unwrap();
//         let cur_pos = reader.current_position().unwrap();
//         let nxt_ch = reader.next_char();
//         if cur_ch != '\n' {
//             println!("Ch: {}, Pos: {:?}", cur_ch, cur_pos);
//         }
//         else {
//             println!("Ch: ..., Pos: {:?}", cur_pos);
//         }
//         if nxt_ch.is_some() {
//             let nxt_ch = nxt_ch.unwrap();
//             let nxt_pos = reader.next_position().unwrap();
//             if nxt_ch != '\n' {
//                 println!("Next Ch: {}, Next Pos: {:?}", nxt_ch, nxt_pos);
//             }
//             else {
//                 println!("Next Ch: ..., Next Pos: {:?}", nxt_pos);
//             }
//         }
//         reader.advance();
//     }
// }
