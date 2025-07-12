///Convert a instruction into its binary representation as specified in the riscv32 ISA
trait BinaryEncoder {
    type Statement;

    fn to_binary(&self, stat: Self::Statement) -> Vec<u8>;

    fn dump(&self, stats: Vec<Self::Statement>) -> Vec<u8> {
        let mut byts = Vec::new();
        for stat in stats {
            let inst_bytes = self.to_binary(stat);
            byts.extend(inst_bytes);
        }
        byts
    }
}

//Binary to Bytecode?
///Convert the binary representation as specified in the riscv32 ISA into a instruction
trait BinaryDecoder {
    type Instruction;

    fn to_instruction(&self, byts: Vec<u8>) -> Self::Instruction;

    fn undump(&self, data: Vec<u8>) -> Vec<Self::Instruction> {
        let mut instructions = Vec::new();
        let mut bag = Vec::new();
        for b in data {
            if bag.len() == 4 {
                let inst_bytes = self.to_instruction(bag.clone());
                instructions.push(inst_bytes);
                bag.clear();
            }
            else {
                bag.push(b);
            }
        }
        instructions
    }
}
