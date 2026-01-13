pub mod gas {
    use crate::lang::{
        directive::Directive, directive::DirectiveInstruction, ext::Extension, ext::M, ext::RV32I,
        highassembly::ArgValue, highassembly::GenericBlock, highassembly::KeyValue,
        highassembly::Register, highassembly::SectionName, pseudo::Pseudo,
        pseudo::PseudoInstruction,
    };

    use crate::streamreader::{
        CharStreamReader, Position, PositionedStringStreamReader, StreamReader, StringStreamReader,
    };

    use crate::lexer::CommonClassifier;

    use crate::tokenizer::{
        GenericToken, ToDirective, ToExtension, ToGenericToken, ToPseudo, ToRegister,
        TokenClassifier,
    };

    use crate::parser::{self};

    use crate::assembler::{self, AssemblerTools};

    /* Lexer */

    pub struct Lexer;

    impl CommonClassifier for Lexer {
        fn is_ambiguous(&self, ch: char) -> bool {
            ch == '+' || ch == '-'
        }
        fn handle_ambiguous(&self, it: &mut CharStreamReader) -> Option<String> {
            let Some(first_ch) = it.current_token() else {
                return None;
            };

            let mut s = String::from(first_ch);

            let Some(second_ch) = it.advance_and_read() else {
                return Some(s);
            };

            if self.is_number(second_ch) {
                if let Some(n) = self.handle_number(it) {
                    s.push_str(&n);
                }
                return Some(s);
            }

            return Some(s);
        }

        fn is_unit(&self, ch: char) -> bool {
            matches!(ch, ',' | '(' | ')')
        }

        fn is_comment(&self, ch: char) -> bool {
            ch == '/'
        }
        fn handle_comment(&self, it: &mut CharStreamReader) -> Option<String> {
            while let Some(ch) = it.read_and_advance() {
                if ch == '\n' {
                    break;
                }
            }
            None
        }

        fn is_identifier(&self, ch: char) -> bool {
            ch.is_ascii_alphanumeric() || ch == '.' || ch == ':' || ch == '_'
        }
        fn handle_identifier(&self, it: &mut CharStreamReader) -> Option<String> {
            let mut name = String::new();
            while let Some(ch) = it.current_token() {
                if !self.is_identifier(ch) {
                    break;
                }
                name.push(ch);
                let _ = it.advance_and_read();
            }
            Some(name)
        }
    }

    /* Tokenizer */
    pub struct Tokenizer;

    #[derive(Debug)]
    pub enum Token {
        Op(Box<dyn Extension>, Position),
        Pseudo(Box<dyn Pseudo>, Position),
        AssemblyDirective(Box<dyn Directive>, Position),
        LinkerDirective(String, Position),
        Reg(Register),
        Name(String, i32),
        Str(String),
        Label(String, Position),
        Number(i32),
        Section(String, Position),
        Plus,
        Minus,
        Lpar,
        Rpar,
        Comma,
    }

    impl ToRegister for Tokenizer {
        fn to_register(&self, token: &str) -> Option<Register> {
            match token {
                "x0" | "zero" => Some(Register::X0),
                "x1" | "ra" => Some(Register::X1),
                "x2" | "sp" => Some(Register::X2),
                "x3" | "gp" => Some(Register::X3),
                "x4" | "tp" => Some(Register::X4),
                "x5" | "t0" => Some(Register::X5),
                "x6" | "t1" => Some(Register::X6),
                "x7" | "t2" => Some(Register::X7),
                "x8" | "s0" | "fp" => Some(Register::X8),
                "x9" | "s1" => Some(Register::X9),
                "x10" | "a0" => Some(Register::X10),
                "x11" | "a1" => Some(Register::X11),
                "x12" | "a2" => Some(Register::X12),
                "x13" | "a3" => Some(Register::X13),
                "x14" | "a4" => Some(Register::X14),
                "x15" | "a5" => Some(Register::X15),
                "x16" | "a6" => Some(Register::X16),
                "x17" | "a7" => Some(Register::X17),
                "x18" | "s2" => Some(Register::X18),
                "x19" | "s3" => Some(Register::X19),
                "x20" | "s4" => Some(Register::X20),
                "x21" | "s5" => Some(Register::X21),
                "x22" | "s6" => Some(Register::X22),
                "x23" | "s7" => Some(Register::X23),
                "x24" | "s8" => Some(Register::X24),
                "x25" | "s9" => Some(Register::X25),
                "x26" | "s10" => Some(Register::X26),
                "x27" | "s11" => Some(Register::X27),
                "x28" | "t3" => Some(Register::X28),
                "x29" | "t4" => Some(Register::X29),
                "x30" | "t5" => Some(Register::X30),
                "x31" | "t6" => Some(Register::X31),
                "pc" => Some(Register::PC),
                _ => None,
            }
        }
    }

    impl ToPseudo for Tokenizer {
        fn to_pseudo(&self, token: &str) -> Option<Box<dyn Pseudo>> {
            match token {
                "ret" => Some(Box::new(PseudoInstruction::RET)),
                "li" => Some(Box::new(PseudoInstruction::LI)),
                "mv" => Some(Box::new(PseudoInstruction::MV)),
                "la" => Some(Box::new(PseudoInstruction::LA)),
                "nop" => Some(Box::new(PseudoInstruction::NOP)),
                _ => None,
            }
        }
    }

    impl ToDirective for Tokenizer {
        fn to_directive(&self, token: &str) -> Option<Box<dyn Directive>> {
            match token {
                ".byte" => Some(Box::new(DirectiveInstruction::Byte)),
                ".word" => Some(Box::new(DirectiveInstruction::Word)),
                ".ascii" => Some(Box::new(DirectiveInstruction::Ascii)),
                ".skip" => Some(Box::new(DirectiveInstruction::Skip)),
                _ => None,
            }
        }
    }

    impl ToExtension<&str> for Tokenizer {
        fn to_extension(&self, token: &str) -> Option<Box<dyn Extension>> {
            match token {
                "lui" => Some(Box::new(RV32I::LUI)),
                "auipc" => Some(Box::new(RV32I::AUIPC)),
                "addi" => Some(Box::new(RV32I::ADDI)),
                "andi" => Some(Box::new(RV32I::ANDI)),
                "ori" => Some(Box::new(RV32I::ORI)),
                "xori" => Some(Box::new(RV32I::XORI)),
                "add" => Some(Box::new(RV32I::ADD)),
                "sub" => Some(Box::new(RV32I::SUB)),
                "and" => Some(Box::new(RV32I::AND)),
                "or" => Some(Box::new(RV32I::OR)),
                "xor" => Some(Box::new(RV32I::XOR)),
                "sll" => Some(Box::new(RV32I::SLL)),
                "srl" => Some(Box::new(RV32I::SRL)),
                "sra" => Some(Box::new(RV32I::SRA)),
                "fence" => Some(Box::new(RV32I::FENCE)),
                "slti" => Some(Box::new(RV32I::SLTI)),
                "sltiu" => Some(Box::new(RV32I::SLTIU)),
                "slli" => Some(Box::new(RV32I::SLLI)),
                "srli" => Some(Box::new(RV32I::SRLI)),
                "srai" => Some(Box::new(RV32I::SRAI)),
                "slt" => Some(Box::new(RV32I::SLT)),
                "sltu" => Some(Box::new(RV32I::SLTU)),
                "lw" => Some(Box::new(RV32I::LW)),
                "lh" => Some(Box::new(RV32I::LH)),
                "lhu" => Some(Box::new(RV32I::LHU)),
                "lb" => Some(Box::new(RV32I::LB)),
                "lbu" => Some(Box::new(RV32I::LBU)),
                "sw" => Some(Box::new(RV32I::SW)),
                "sh" => Some(Box::new(RV32I::SH)),
                "sb" => Some(Box::new(RV32I::SB)),
                "jal" => Some(Box::new(RV32I::JAL)),
                "jalr" => Some(Box::new(RV32I::JALR)),
                "ecall" => Some(Box::new(RV32I::ECALL)),
                "beq" => Some(Box::new(RV32I::BEQ)),
                "bne" => Some(Box::new(RV32I::BNE)),
                "blt" => Some(Box::new(RV32I::BLT)),
                "bltu" => Some(Box::new(RV32I::BLTU)),
                "bge" => Some(Box::new(RV32I::BGE)),
                "bgeu" => Some(Box::new(RV32I::BGEU)),
                "mul" => Some(Box::new(M::MUL)),
                "mulh" => Some(Box::new(M::MULH)),
                "mulhu" => Some(Box::new(M::MULHU)),
                "mulhsu" => Some(Box::new(M::MULHSU)),
                "div" => Some(Box::new(M::DIV)),
                "divu" => Some(Box::new(M::DIVU)),
                "rem" => Some(Box::new(M::REM)),
                "remu" => Some(Box::new(M::REMU)),
                _ => None,
            }
        }
    }

    impl TokenClassifier for Tokenizer {
        type Token = Token;

        fn is_register(&self, token: &str) -> bool {
            ToRegister::is_register(self, token)
        }

        fn is_symbol(&self, token: &str) -> bool {
            matches!(token, "," | "(" | ")" | "+" | "-")
        }

        fn is_opcode(&self, token: &str) -> bool {
            self.in_extension(token)
        }

        fn is_identifier(&self, token: &str) -> bool {
            let mut chs = token.chars();
            let f: char = chs.nth(0).unwrap_or(' ');
            let first_ch_check = f.is_ascii_alphabetic() || matches!(f, '_' | '.');
            let remaining_string_check =
                chs.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_'));
            first_ch_check && remaining_string_check
        }

        fn is_section(&self, token: &str) -> bool {
            token == ".section"
        }

        fn is_directive(&self, token: &str) -> bool {
            ToDirective::is_directive(self, token)
        }

        fn is_label(&self, token: &str) -> bool {
            token.ends_with(':')
        }

        fn is_custom(&self, token: &str) -> bool {
            ToPseudo::is_pseudo(self, token) || token == ".globl"
        }

        fn handle_number(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };

            let token = &token.0;

            let decimal = if token.contains('x') {
                let hex = token.replace("0x", "");
                i32::from_str_radix(&hex, 16)
            } else {
                token.parse::<i32>()
            };

            let Ok(decimal) = decimal else {
                panic!("Error converting number to decimal");
            };

            // Check for syntax: <offset> '(' <Identifier> ')'
            let Some(_) = it.advance_if(|next_token| &next_token.0 == "(") else {
                return Some(Token::Number(decimal));
            };
            let Some(identifier) = it.advance_if(|next_token| {
                !TokenClassifier::is_register(self, &next_token.0)
                    && self.is_identifier(&next_token.0)
            }) else {
                return Some(Token::Number(decimal));
            };
            let Some(_) = it.advance_if(|next_token| &next_token.0 == ")") else {
                return Some(Token::Number(decimal));
            };
            Some(Token::Name(identifier.0, decimal))
        }

        fn handle_string(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            Some(Token::Str(token.0.trim_matches('"').to_string()))
        }

        fn handle_symbol(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            match token.0.as_str() {
                "," => Some(Token::Comma),
                "(" => Some(Token::Lpar),
                ")" => Some(Token::Rpar),
                "+" => Some(Token::Plus),
                "-" => Some(Token::Minus),
                _ => None,
            }
        }

        fn handle_opcode(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            match self.to_extension(&token.0) {
                Some(e) => Some(Token::Op(e, token.1)),
                None => None,
            }
        }

        fn handle_identifier(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            Some(Token::Name(token.0.to_string(), 0))
        }

        fn handle_section(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.advance_and_read() else {
                return None;
            };
            Some(Token::Section(token.0[1..].to_string(), token.1))
        }

        fn handle_directive(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            if let Some(d) = ToDirective::to_directive(self, &token.0) {
                Some(Token::AssemblyDirective(d, token.1))
            } else {
                None
            }
        }

        fn handle_register(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            let Some(reg) = ToRegister::to_register(self, token.0.trim().to_lowercase().as_str())
            else {
                return None;
            };
            Some(Token::Reg(reg))
        }

        fn handle_custom(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            if let Some(p) = ToPseudo::to_pseudo(self, &token.0) {
                Some(Token::Pseudo(p, token.1))
            } else if &token.0 == ".globl" {
                Some(Token::LinkerDirective(token.0.to_string(), token.1))
            } else {
                None
            }
        }

        fn handle_label(&self, it: &mut PositionedStringStreamReader) -> Option<Self::Token> {
            let Some(token) = it.current_token_ref() else {
                return None;
            };
            Some(Token::Label(
                token.0.trim_end_matches(':').to_string(),
                token.1,
            ))
        }
    }

    impl ToGenericToken for Token {
        fn to_generic_token(self) -> Option<GenericToken> {
            match self {
                Token::Plus => None,
                Token::Minus => None,
                Token::Lpar => None,
                Token::Rpar => None,
                Token::Comma => None,
                Token::Op(extension, pos) => {
                    Some(GenericToken::KeyToken(KeyValue::Op(extension), pos))
                }
                Token::Pseudo(pseudo, pos) => {
                    Some(GenericToken::KeyToken(KeyValue::Pseudo(pseudo), pos))
                }
                Token::Label(label, pos) => {
                    Some(GenericToken::KeyToken(KeyValue::Label(label), pos))
                }
                Token::AssemblyDirective(directive, pos) => Some(GenericToken::KeyToken(
                    KeyValue::AssemblyDirective(directive),
                    pos,
                )),
                Token::Reg(register) => Some(GenericToken::ArgToken(ArgValue::Register(register))),
                Token::Name(name, off) => Some(GenericToken::ArgToken(ArgValue::Use(name, off))),
                Token::Str(literal) => Some(GenericToken::ArgToken(ArgValue::Literal(literal))),
                Token::Number(n) => Some(GenericToken::ArgToken(ArgValue::Number(n))),
                Token::Section(sec, pos) => match sec.as_str() {
                    "text" => Some(GenericToken::KeyToken(
                        KeyValue::Section(SectionName::Text),
                        pos,
                    )),
                    "data" => Some(GenericToken::KeyToken(
                        KeyValue::Section(SectionName::Data),
                        pos,
                    )),
                    "bss" => Some(GenericToken::KeyToken(
                        KeyValue::Section(SectionName::Bss),
                        pos,
                    )),
                    other => Some(GenericToken::KeyToken(
                        KeyValue::Section(SectionName::Custom(other.to_string())),
                        pos,
                    )),
                },
                Token::LinkerDirective(s, pos) => match s.as_str() {
                    ".globl" => Some(GenericToken::KeyToken(KeyValue::LinkerDirective(s), pos)),
                    _ => None,
                },
            }
        }
    }

    /* Parser */

    pub struct Parser;

    impl parser::Parser for Parser {
        type Token = Token;
        type Output = Vec<GenericBlock>;

        fn parse(&self, tokens: Vec<Self::Token>) -> Self::Output {
            parser::parse(tokens)
        }
    }

    // /* Assembler */
    pub struct Assembler;

    impl assembler::Assembler for Assembler {
        type Input = Vec<GenericBlock>;

        fn assemble(&self, instruction: Self::Input) -> AssemblerTools {
            assembler::assemble(instruction)
        }
    }

    /* Loader */
}
