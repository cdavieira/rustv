pub trait Machine {
    fn fetch(&self) -> u32 ;

    //TODO: should i execute a instruction as a u32 or as a Statement?
    fn decode(&mut self, instr: u32) -> () ;

    fn cycle(&mut self) -> () ;
}
