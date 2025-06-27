mod lexer;
mod tokenizer;
mod spec;
use lexer::Lexer;

fn main() {
    // if let Some(word) = std::env::args().nth(1) {
    //     let pat = Regex::new(r"[a-zA-Z_]+").unwrap();
    //     println!("{}", pat.is_match(&word));
    // }

    // let a: Vec<&str> = RISCV32_ASSEMBLY.split_whitespace().collect();
    // println!("{a:?}");

    // let mut keys = HashMap::new();
    // keys.insert(',', ());

    let tokenizer = tokenizer::IntelTokenizer;
    let code = "
            .text
            .globl main
        //this is gonna be great\n
        main:
            li   a0, 0

            lw   ra, -12(sp)
            lw   s0, +8(sp)
            addi x3, sp, 16 + 9
            ret
    ";
    let lexer = lexer::IntelLexer;
    let lexemes = lexer.parse(&tokenizer, code);
    println!("{:?}", lexemes);
    // lexer::parse(&tokenizer, code);
}
