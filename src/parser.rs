pub trait Parser<'a> {
    type Token;
    type Output;

    fn parse(&'a self, token: &'a Vec<Self::Token>) -> Vec<Self::Output> ;
}



/* The following code was written to ease the process of implementing the 'Parser' trait. */

pub struct Instruction<T> {
    pub keyword: T,
    pub args: Vec<T>
}

pub trait IsInstruction{
    type Token;
    fn is_instruction(&self, token: &Self::Token) -> bool;
}

pub fn get_instructions<'a, T>(
    decoder: &'a impl IsInstruction<Token = T>,
    tokens: &'a Vec<T>
) -> Vec<Instruction<&'a T>>
{
    let mut v = Vec::new();
    let mut it = tokens.iter();
    let mut opt = it.next();
    while let Some(token) = opt {
        if decoder.is_instruction(&token) {
            let mut i = Instruction{
                keyword: token,
                args: Vec::new()
            };
            opt = it.next();
            while let Some(token) = opt {
                if decoder.is_instruction(token) {
                    break;
                }
                i.args.push(token);
                opt = it.next();
            }
            v.push(i);
            // opt = builder.handle_instruction(&mut it, &mut st);
            // v.push(st);
        }
        else {
            opt = it.next();
        }
    }
    v
}
