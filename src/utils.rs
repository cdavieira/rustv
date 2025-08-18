// uword -> u32
// word  -> i32
// byte  -> u8

const UWORD_LOWER_3BITS:  u32 = 0b111;
const UWORD_LOWER_5BITS:  u32 = 0b11111;
const UWORD_LOWER_7BITS:  u32 = 0b1111111;
const UWORD_LOWER_12BITS: u32 = 0b111111111111;
const UWORD_LOWER_20BITS: u32 = 0b11111_11111_11111_11111;

const WORD_LOWER_3BITS:  i32 = 0b111;
const WORD_LOWER_5BITS:  i32 = 0b11111;
const WORD_LOWER_7BITS:  i32 = 0b1111111;
const WORD_LOWER_12BITS: i32 = 0b111111111111;
const WORD_LOWER_20BITS: i32 = 0b11111_11111_11111_11111;

const UWORD_B00_B07: u32 = 0b00000000_00000000_00000000_11111111;
const UWORD_B08_B15: u32 = 0b00000000_00000000_11111111_00000000;
const UWORD_B16_B23: u32 = 0b00000000_11111111_00000000_00000000; 
const UWORD_B24_B31: u32 = 0b11111111_00000000_00000000_00000000;


// Conversions

pub fn word_to_byte(n: &i32) -> (u8, u8, u8, u8) {
    let b0 = word_mask_off(n, UWORD_B00_B07 as i32, 0).try_into().unwrap() ;
    let b1 = word_mask_off(n, UWORD_B08_B15 as i32, 8).try_into().unwrap() ;
    let b2 = word_mask_off(n, UWORD_B16_B23 as i32, 16).try_into().unwrap();
    let b3 = word_mask_off(n, UWORD_B24_B31 as i32, 24).try_into().unwrap();
    (b3, b2, b1, b0)
}

pub fn uword_to_byte(n: &u32) -> (u8, u8, u8, u8) {
    let b0 = uword_mask_off(n, UWORD_B00_B07, 0).try_into().unwrap() ;
    let b1 = uword_mask_off(n, UWORD_B08_B15, 8).try_into().unwrap() ;
    let b2 = uword_mask_off(n, UWORD_B16_B23, 16).try_into().unwrap();
    let b3 = uword_mask_off(n, UWORD_B24_B31, 24).try_into().unwrap();
    (b3, b2, b1, b0)
}

pub fn uwords_to_bytes(uwords: &Vec<u32>) -> Vec<u8> {
    let mut v = Vec::new();
    for uword in uwords {
        let b0: u8 = ((uword & 0b00000000_00000000_00000000_11111111) >> 0).try_into().unwrap();
        let b1: u8 = ((uword & 0b00000000_00000000_11111111_00000000) >> 8).try_into().unwrap();
        let b2: u8 = ((uword & 0b00000000_11111111_00000000_00000000) >> 16).try_into().unwrap();
        let b3: u8 = ((uword & 0b11111111_00000000_00000000_00000000) >> 24).try_into().unwrap();
        v.push(b3);
        v.push(b2);
        v.push(b1);
        v.push(b0);
    }
    v
}



// Bit inspection



// Cast

pub fn uword_off_mask(n: &u32, off: u8, mask: u32) -> u32 {
    (n >> off) & mask
}

pub fn uword_mask_off(n: &u32, mask: u32, off: u8) -> u32 {
    (n & mask) >> off
}

pub fn uword_mask_lower_3bits(n: &u32) -> u32 {
    n & UWORD_LOWER_3BITS
}

pub fn uword_mask_lower_5bits(n: &u32) -> u32 {
    n & UWORD_LOWER_5BITS
}

pub fn uword_mask_lower_7bits(n: &u32) -> u32 {
    n & UWORD_LOWER_7BITS
}

pub fn uword_mask_lower_12bits(n: &u32) -> u32 {
    n & UWORD_LOWER_12BITS
}

pub fn uword_mask_lower_20bits(n: &u32) -> u32 {
    n & UWORD_LOWER_20BITS
}



pub fn word_off_mask(n: &i32, off: u8, mask: i32) -> i32 {
    (n >> off) & mask
}

pub fn word_mask_off(n: &i32, mask: i32, off: u8) -> i32 {
    (n & mask) >> off
}

pub fn word_mask_lower_3bits(n: &i32) -> i32 {
    n & WORD_LOWER_3BITS
}

pub fn word_mask_lower_5bits(n: &i32) -> i32 {
    n & WORD_LOWER_5BITS
}

pub fn word_mask_lower_7bits(n: &i32) -> i32 {
    n & WORD_LOWER_7BITS
}

pub fn word_mask_lower_12bits(n: &i32) -> i32 {
    n & WORD_LOWER_12BITS
}

pub fn word_mask_lower_20bits(n: &i32) -> i32 {
    n & WORD_LOWER_20BITS
}
