pub mod spec;
pub mod tokenizer;
pub mod lexer;
pub mod parser;
pub mod syntax;
pub mod assembler;
pub mod memory;
pub mod elf;
pub mod cpu;
pub mod machine;
pub mod utils;
pub mod elfwriter;

use gdbstub::stub::state_machine::GdbStubStateMachine;
use rustv::utils::uwords_to_bytes;
use crate::assembler::Assembler;
use crate::lexer::Lexer;
use crate::spec::AssemblySection;
use crate::tokenizer::Tokenizer;
use crate::parser::Parser;
use crate::elf::{read_elf, write_elf, write_elf2};

fn main() {
    let mut mem  = Vec::new();
    let memsize = 1024*1024;
    mem.reserve(memsize);
    for _ in 0..memsize {
        mem.push(0);
    }
    let mut mytarget = MyTarget::new(mem);

    let conx = wait_for_gdb_connection(9999u16).unwrap();

    let mut mystub = GdbStub::new(conx);

    {
        let support_software_breakpoints = mytarget
            .support_breakpoints()
            .map(|ops| ops.support_sw_breakpoint().is_some())
            .unwrap_or(false);
        let cond1 = !support_software_breakpoints;
        let cond2 = !mytarget.guard_rail_implicit_sw_breakpoints();
        println!("{} {}", cond1, cond2);
    }
    match mystub.run_state_machine(&mut mytarget) {
        Ok(s) => {
            let mut res = idontknow(s, &mut mytarget);
            while let Ok(ss) = res {
                res = idontknow(ss, &mut mytarget);
            }
        }
        Err(e) => {
            println!("Failed when running machine: {:?}", e);
        }
    }

    // gdb_event_loop_thread(mystub, mytarget);

    // let code = "
    //     li a7, 93
    //     li a0, 1000
    //     ecall
    // ";
    // let mut t = syntax::gas::Tokenizer;
    // let l = syntax::gas::Lexer;
    // let p = syntax::gas::Parser;
    // let s = syntax::gas::Assembler;
    // let tokens = t.get_tokens(code);
    // // println!("{:?}", &tokens);
    // let lexemes = l.parse(tokens);
    // // println!("{:?}", &lexemes);
    // let parser_output = p.parse(lexemes);
    // let (metadata, mut symbol_table, section_table, sections) = parser_output.get_all();
    // let sections: Vec<spec::AssemblyData> = sections
    //     .into_iter()
    //     .map(|section| {
    //         s.to_words(section)
    //     })
    //     .collect();
    // let mut writer = elfwriter::ElfWriter::new();
    // if symbol_table.contains_key("_start") {
    //     let (_, start_symbol_addr) = *symbol_table.get("_start").unwrap();
    //     writer.set_start_address(start_symbol_addr.try_into().unwrap());
    //     let _ = symbol_table.remove("_start").unwrap();
    // }
    // else {
    //     writer.set_start_address(0);
    // }
    // // TODO: handle what length actually means
    // for (symb, (sect_name, rel_addr)) in symbol_table {
    //     let length = 0;
    //     writer.add_symbol(sect_name, rel_addr.try_into().unwrap(), &symb, length);
    // }
    // for section in sections {
    //     if section.data.len() > 0 {
    //         let name = section.name;
    //         let data = swap_words_endianness(uwords_to_bytes(&section.data));
    //         //REVIEW: THIS ALIGNMENT IS WRONG, I ONLY DID THIS TO TEST ONE THING
    //         match name {
    //             spec::AssemblySectionName::TEXT => {
    //                 writer.set_section_data(name, data, 4);
    //             },
    //             spec::AssemblySectionName::DATA => {
    //                 writer.set_section_data(name, data, 1);
    //             },
    //             _ => {}
    //         }
    //     }
    // }
    // writer.save("main2.o");
}

fn swap_words_endianness(v: Vec<u8>) -> Vec<u8> {
    let mut u: Vec<u8> = Vec::new();
    for b in v.chunks_exact(4) {
        u.push(b[3]);
        u.push(b[2]);
        u.push(b[1]);
        u.push(b[0]);
    }
    u
}

fn idontknow<'a>(
    s: GdbStubStateMachine<'a, MyTarget, TcpStream>,
    mytarget: &mut MyTarget
) -> Result<GdbStubStateMachine<'a, MyTarget, TcpStream>, ()> {
    match s {
        gdbstub::stub::state_machine::GdbStubStateMachine::Idle(mut gdb_stub_state_machine_inner) => {
            println!("Idle");
            let resbyte = gdb_stub_state_machine_inner.borrow_conn().read();
            match resbyte {
                Ok(byte) => {
                    if byte.is_ascii_graphic() {
                        let ch: char = byte.try_into().unwrap();
                        println!("Got byte {}", ch);
                    }
                    else {
                        println!("Got byte {}", byte);
                    }
                    let stubresponse = gdb_stub_state_machine_inner.incoming_data(mytarget, byte);
                    match stubresponse {
                        Ok(stubresponseok) => {
                            println!("Stub response Ok");
                            Ok(stubresponseok)
                        },
                        Err(stubresponseerr) => {
                            println!("Stub response err: {:?}", stubresponseerr);
                            Err(())
                        }
                    }
                },
                Err(err) => {
                    println!("Error in idle {:?}", err);
                    Err(())
                }
            }
        },
        gdbstub::stub::state_machine::GdbStubStateMachine::Running(mut gdb_stub_state_machine_inner) => {
            println!("Running");
            use run_blocking::Event as BlockingEventLoopEvent;
            use run_blocking::WaitForStopReasonError;

            // block waiting for the target to return a stop reason
            let event = <MyGdbBlockingEventLoop as
                run_blocking::BlockingEventLoop>::wait_for_stop_reason(mytarget,
                    gdb_stub_state_machine_inner.borrow_conn());
            match event {
                Ok(BlockingEventLoopEvent::TargetStopped(stop_reason)) => {
                    println!("Running - Got target stopped");
                    let _ = gdb_stub_state_machine_inner.report_stop(mytarget, stop_reason);
                }

                Ok(BlockingEventLoopEvent::IncomingData(byte)) => {
                    if byte.is_ascii_graphic() {
                        let ch: char = byte.try_into().unwrap();
                        println!("Running - Got byte {}", ch);
                    }
                    else {
                        println!("Running - Got byte {}", byte);
                    }
                    gdb_stub_state_machine_inner.incoming_data(mytarget, byte);
                }

                Err(WaitForStopReasonError::Target(e)) => {
                    println!("Running - Got target");
                    // break Err(InternalError::TargetError(e).into());
                }
                Err(WaitForStopReasonError::Connection(e)) => {
                    println!("Running - Got connection");
                    // break Err(InternalError::conn_read(e).into());
                }
            }
            Err(())
        },
        gdbstub::stub::state_machine::GdbStubStateMachine::CtrlCInterrupt(gdb_stub_state_machine_inner) => {
            println!("Ctrlc");
            Err(())
        },
        gdbstub::stub::state_machine::GdbStubStateMachine::Disconnected(gdb_stub_state_machine_inner) => {
            println!("Disconnected");
            Err(())
        }
    }
}






// Connection
use std::io;
use std::net::{TcpListener, TcpStream};

fn wait_for_gdb_connection(port: u16) -> io::Result<TcpStream> {
    let sockaddr = format!("localhost:{}", port);
    eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);
    let sock = TcpListener::bind(sockaddr)?;
    let (stream, addr) = sock.accept()?;

    // Blocks until a GDB client connects via TCP.
    // i.e: Running `target remote localhost:<port>` from the GDB prompt.

    eprintln!("Debugger connected from {}", addr);
    Ok(stream) // `TcpStream` implements `gdbstub::Connection`
}







// Target
struct MyTarget {
    machine: SimpleMachine,
}

impl MyTarget {
    pub fn new(mem: Vec<u32>) -> Self {
        let machine = SimpleMachine::new(&mem);
        MyTarget { machine }
    }
}

use gdbstub::common::Signal;
use gdbstub::target::{Target, TargetResult};
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::singlethread::{
    SingleThreadResumeOps, SingleThreadSingleStepOps
};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadSingleStep
};
use gdbstub::target::ext::breakpoints::{Breakpoints, SwBreakpoint};
use gdbstub::target::ext::breakpoints::{BreakpointsOps, SwBreakpointOps};
use machine::{Machine, SimpleMachine};

impl Target for MyTarget {
    type Error = ();
    type Arch = gdbstub_arch::riscv::Riscv32; // as an example

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    // opt-in to support for setting/removing breakpoints
    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<Self>> {
        Some(self)
    }
}

impl SingleThreadBase for MyTarget {
    fn read_registers(
        &mut self,
        regs: &mut gdbstub_arch::riscv::reg::RiscvCoreRegs<u32>,
    ) -> TargetResult<(), Self> { 
        let myregs = self.machine.read_registers();
        let gps = &myregs[..32];
        let pc  = &myregs[32];
        for (idx, reg) in gps.iter().enumerate() {
            regs.x[idx] = (*reg).into();
        }
        regs.pc = (*pc).into();
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &gdbstub_arch::riscv::reg::RiscvCoreRegs<u32>,
    ) -> TargetResult<(), Self> { 
        let gprs = regs.x.to_vec();
        let pc: usize = regs.pc.try_into().unwrap();
        self.machine.write_registers(gprs, pc);
        Ok(())
    }

    // TODO: return less bytes than data.len() to inform that there aren't that many bytes to be
    // read
    fn read_addrs(
        &mut self,
        start_addr: u32,
        data: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let count = data.len();
        let start: usize = start_addr.try_into().unwrap();
        let bytes = self.machine.read_memory_bytes(start, count);
        for (idx, byte) in bytes.into_iter().enumerate() {
            data[idx] = byte;
        }
        Ok(count)
    }

    fn write_addrs(
        &mut self,
        start_addr: u32,
        data: &[u8],
    ) -> TargetResult<(), Self> {
        let start: usize = start_addr.try_into().unwrap();
        for (idx, byte) in data.iter().enumerate() {
            self.machine.write_memory_byte(start+idx, *byte);
        }
        Ok(())
    }

    // most targets will want to support at resumption as well...

    #[inline(always)]
    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<Self>> {
        Some(self)
    }
}

impl SingleThreadResume for MyTarget {
    fn resume(
        &mut self,
        signal: Option<Signal>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    // ...and if the target supports resumption, it'll likely want to support
    // single-step resume as well

    #[inline(always)]
    fn support_single_step(
        &mut self
    ) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadSingleStep for MyTarget {
    fn step(
        &mut self,
        signal: Option<Signal>,
    ) -> Result<(), Self::Error> {
        self.machine.decode();
        Ok(())
    }
}

impl Breakpoints for MyTarget {
    // there are several kinds of breakpoints - this target uses software breakpoints
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<Self>> {
        Some(self)
    }
}

impl SwBreakpoint for MyTarget {
    fn add_sw_breakpoint(
        &mut self,
        addr: u32,
        kind: usize,
    ) -> TargetResult<bool, Self> {
        println!("Trying to add a sw breakpoint at {} {}", addr, kind);
        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: u32,
        kind: usize,
    ) -> TargetResult<bool, Self> {
        println!("Trying to rm a sw breakpoint at {} {}", addr, kind);
        Ok(false)
    }
}







// Event loop
use gdbstub::conn::{Connection, ConnectionExt}; // note the use of `ConnectionExt`
use gdbstub::stub::{run_blocking, DisconnectReason, GdbStub};
use gdbstub::stub::SingleThreadStopReason;

enum MyGdbBlockingEventLoop {}

// The `run_blocking::BlockingEventLoop` groups together various callbacks
// the `GdbStub::run_blocking` event loop requires you to implement.
impl run_blocking::BlockingEventLoop for MyGdbBlockingEventLoop {
    type Target = MyTarget;
    type Connection = TcpStream;

    // or MultiThreadStopReason on multi threaded targets
    type StopReason = SingleThreadStopReason<u32>;

    // Invoked immediately after the target's `resume` method has been
    // called. The implementation should block until either the target
    // reports a stop reason, or if new data was sent over the connection.
    fn wait_for_stop_reason(
        target: &mut MyTarget,
        conn: &mut Self::Connection,
    ) -> Result<
        run_blocking::Event<SingleThreadStopReason<u32>>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error,
        >,
    > {
        // the specific mechanism to "select" between incoming data and target
        // events will depend on your project's architecture.
        //
        // some examples of how you might implement this method include: `epoll`,
        // `select!` across multiple event channels, periodic polling, etc...
        //
        // in this example, lets assume the target has a magic method that handles
        // this for us.
        // let event = match target.run_and_check_for_incoming_data(conn) {
        //     MyTargetEvent::IncomingData => {
        //         let byte = conn
        //             .read() // method provided by the `ConnectionExt` trait
        //             .map_err(run_blocking::WaitForStopReasonError::Connection)?;
        //
        //         run_blocking::Event::IncomingData(byte)
        //     }
        //     MyTargetEvent::StopReason(reason) => {
        //         run_blocking::Event::TargetStopped(reason)
        //     }
        // };
        //
        // Ok(event)
        let byte = conn.read()
            .map_err(run_blocking::WaitForStopReasonError::Connection)?;
        Ok(run_blocking::Event::IncomingData(byte))
    }

    // Invoked when the GDB client sends a Ctrl-C interrupt.
    fn on_interrupt(
        target: &mut MyTarget,
    ) -> Result<Option<SingleThreadStopReason<u32>>, <MyTarget as Target>::Error> {
        // notify the target that a ctrl-c interrupt has occurred.
        // target.stop_in_response_to_ctrl_c_interrupt()?;

        // a pretty typical stop reason in response to a Ctrl-C interrupt is to
        // report a "Signal::SIGINT".
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT).into()))
    }
}

fn gdb_event_loop_thread(
    debugger: GdbStub<MyTarget, TcpStream>,
    mut target: MyTarget
) {
    match debugger.run_blocking::<MyGdbBlockingEventLoop>(&mut target) {
        Ok(disconnect_reason) => match disconnect_reason {
            DisconnectReason::Disconnect => {
                println!("Client disconnected")
            }
            DisconnectReason::TargetExited(code) => {
                println!("Target exited with code {}", code)
            }
            DisconnectReason::TargetTerminated(sig) => {
                println!("Target terminated with signal {}", sig)
            }
            DisconnectReason::Kill => println!("GDB sent a kill command"),
        },
        Err(e) => {
            if e.is_target_error() {
                // println!(
                //     "target encountered a fatal error: {}",
                //     e.into_target_error().unwrap()
                // )
            } else if e.is_connection_error() {
                let (e, kind) = e.into_connection_error().unwrap();
                println!("connection error: {:?} - {}", kind, e,)
            } else {
                // println!("gdbstub encountered a fatal error: {}", e)
                println!("fila da puta");
            }
        }
    }
}
