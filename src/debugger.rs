use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::marker::PhantomData;


use gdbstub::common::Signal;
use gdbstub::target::{
    Target,
    TargetResult
};
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::singlethread::{
    SingleThreadResumeOps,
    SingleThreadSingleStepOps
};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase,
    SingleThreadResume,
    SingleThreadSingleStep
};
use gdbstub::target::ext::breakpoints::{
    Breakpoints,
    SwBreakpoint
};
use gdbstub::target::ext::breakpoints::{
    BreakpointsOps,
    SwBreakpointOps
};
use gdbstub::stub::state_machine::GdbStubStateMachine;


use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::{run_blocking, DisconnectReason, GdbStub};
use gdbstub::stub::SingleThreadStopReason;


use crate::machine::Machine;
use crate::utils::DataEndianness;





/// TCP based Stub
pub struct SimpleGdbStub<'a, T: Machine> {
    // Connection
    addr: SocketAddr,

    // Target
    target: SimpleTarget<T>,

    // GdbStug
    stub: GdbStub<'a, SimpleTarget<T>, TcpStream>,

    // Loop
    // _
}

impl<'a, T: Machine> SimpleGdbStub<'a, T> {
    pub fn new(memsize: usize, port: u16) -> io::Result<Self> {
        let mut mem = Vec::new();
        mem.reserve(memsize);
        for _ in 0..memsize {
            mem.push(0);
        }
        let target = SimpleTarget::from_words(mem);
        let (stream, addr) = wait_for_gdb_connection(port)?;
        let stub = GdbStub::new(stream);
        Ok(
            SimpleGdbStub {
                addr,
                target,
                stub
            }
        )
    }

    pub fn custom_gdb_event_loop_thread(mut self){
        match self.stub.run_state_machine(&mut self.target) {
            Ok(sm_ok) => {
                let mut handle_res = custom_handle_machine_state(sm_ok, &mut self.target);
                while let Ok(sm_ok) = handle_res {
                    handle_res = custom_handle_machine_state(sm_ok, &mut self.target);
                }
            }
            Err(sm_err) => {
                println!("Failed when running state machine: {:?}", sm_err);
            }
        }
    }

    pub fn default_gdb_event_loop_thread(mut self){
        match self.stub.run_blocking::<SimpleGdbBlockingEventLoop<T>>(&mut self.target) {
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
                    println!("gdbstub encountered a fatal error (target error)");
                } else if e.is_connection_error() {
                    let (e, kind) = e.into_connection_error().unwrap();
                    println!("connection error: {:?} - {}", kind, e,);
                } else {
                    // println!("gdbstub encountered a fatal error: {}", e)
                    println!("gdbstub encountered a fatal error");
                }
            }
        }
    }
}





// Connection

/// Blocks until a GDB client connects via TCP.
/// i.e: Running `target remote localhost:<port>` from the GDB prompt.
/// `TcpStream` implements `gdbstub::Connection`
fn wait_for_gdb_connection(port: u16) -> io::Result<(TcpStream, SocketAddr)> {
    let sockaddr = format!("localhost:{}", port);
    let sock = TcpListener::bind(sockaddr)?;
    println!("Waiting for GDB to connect to target at localhost:{}", port);
    println!("Enter gdb and type:");
    println!("  gdb> target remote :{}", port);
    println!("  gdb> load");
    println!("  gdb> x/1xw 0x10074");
    sock.accept()
}






// Target
struct SimpleTarget<T: Machine> {
    machine: T,
}

impl<T: Machine> SimpleTarget<T> {
    pub fn from_words(mem: Vec<u32>) -> Self {
        let machine = <T>::from_words(&mem, DataEndianness::LE);
        SimpleTarget { machine }
    }
}

impl<T: Machine> Target for SimpleTarget<T> {
    type Error = ();
    type Arch = gdbstub_arch::riscv::Riscv32;

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<Self::Arch, Self::Error>
    {
        BaseOps::SingleThread(self)
    }

    // opt-in to support for setting/removing breakpoints
    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<Self>>
    {
        Some(self)
    }
}

impl<T: Machine> SingleThreadBase for SimpleTarget<T> {
    fn read_registers(
        &mut self,
        regs: &mut gdbstub_arch::riscv::reg::RiscvCoreRegs<u32>,
    ) -> TargetResult<(), Self>
    { 
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
    ) -> TargetResult<(), Self>
    { 
        let gprs = regs.x.to_vec();
        let pc: usize = regs.pc.try_into().unwrap();
        self.machine.write_registers(gprs, pc);
        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: u32,
        data: &mut [u8],
    ) -> TargetResult<usize, Self>
    {
        let start_addr: usize = start_addr.try_into().unwrap();
        let data_size = data.len();
        let mem_size = self.machine.bytes_count();
        if start_addr < mem_size {
            let free_mem_size = mem_size - start_addr;
            let bytes_size = if data_size < free_mem_size { data_size } else { free_mem_size };
            let bytes = self.machine.read_memory_bytes(start_addr, bytes_size, 1);
            for (idx, byte) in bytes.into_iter().enumerate() {
                data[idx] = byte;
            }
            Ok(bytes_size)
        }
        else {
            Ok(0usize)
        }
    }

    fn write_addrs(
        &mut self,
        start_addr: u32,
        data: &[u8],
    ) -> TargetResult<(), Self>
    {
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

impl<T: Machine> SingleThreadResume for SimpleTarget<T> {
    fn resume(
        &mut self,
        _signal: Option<Signal>,
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

impl<T: Machine> SingleThreadSingleStep for SimpleTarget<T> {
    fn step(
        &mut self,
        _signal: Option<Signal>,
    ) -> Result<(), Self::Error>
    {
        // self.machine.decode();
        Ok(())
    }
}

impl<T: Machine> Breakpoints for SimpleTarget<T> {
    // there are several kinds of breakpoints - this target uses software breakpoints
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<Self>>
    {
        Some(self)
    }
}

impl<T: Machine> SwBreakpoint for SimpleTarget<T> {
    fn add_sw_breakpoint(
        &mut self,
        addr: u32,
        kind: usize,
    ) -> TargetResult<bool, Self>
    {
        // println!("Trying to add a sw breakpoint at {} {}", addr, kind);
        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: u32,
        kind: usize,
    ) -> TargetResult<bool, Self>
    {
        // println!("Trying to rm a sw breakpoint at {} {}", addr, kind);
        Ok(true)
    }
}






// Loop

struct SimpleGdbBlockingEventLoop<T: Machine> {
    _marker: PhantomData<T>,
}

// The `run_blocking::BlockingEventLoop` groups together various callbacks
// the `GdbStub::run_blocking` event loop requires you to implement.
impl<T: Machine> run_blocking::BlockingEventLoop for SimpleGdbBlockingEventLoop<T> {
    type Target = SimpleTarget<T>;
    type Connection = TcpStream;

    // or MultiThreadStopReason on multi threaded targets
    type StopReason = SingleThreadStopReason<u32>;

    // Invoked immediately after the target's `resume` method has been
    // called. The implementation should block until either the target
    // reports a stop reason, or if new data was sent over the connection.
    fn wait_for_stop_reason(
        target: &mut SimpleTarget<T>,
        _conn: &mut Self::Connection,
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

        target.machine.decode();
        Ok(run_blocking::Event::TargetStopped(SingleThreadStopReason::DoneStep))
    }

    // Invoked when the GDB client sends a Ctrl-C interrupt.
    fn on_interrupt(
        _target: &mut SimpleTarget<T>,
    ) -> Result<Option<SingleThreadStopReason<u32>>, <SimpleTarget<T> as Target>::Error> {
        // notify the target that a ctrl-c interrupt has occurred.
        // target.stop_in_response_to_ctrl_c_interrupt()?;

        // a pretty typical stop reason in response to a Ctrl-C interrupt is to
        // report a "Signal::SIGINT".
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT).into()))
    }
}

fn custom_handle_machine_state<'a, T: Machine>(
    stub_sm: GdbStubStateMachine<'a, SimpleTarget<T>, TcpStream>,
    target: &mut SimpleTarget<T>
) -> Result<GdbStubStateMachine<'a, SimpleTarget<T>, TcpStream>, ()>
{
    match stub_sm {
        gdbstub::stub::state_machine::GdbStubStateMachine::Idle(mut gdb_stub_state_machine_inner) => {
            // println!("Idle");
            let read_result = gdb_stub_state_machine_inner.borrow_conn().read();
            match read_result {
                Ok(byte) => {
                    if byte.is_ascii_graphic() {
                        let ch: char = byte.try_into().unwrap();
                        // println!("Got byte {}", ch);
                    }
                    else {
                        // println!("Got byte {}", byte);
                    }
                    let stub_result = gdb_stub_state_machine_inner.incoming_data(target, byte);
                    match stub_result {
                        Ok(stub_ok) => {
                            // println!("Stub Ok");
                            Ok(stub_ok)
                        },
                        Err(stub_err) => {
                            // println!("Stub Err: {:?}", stub_err);
                            Err(())
                        }
                    }
                },
                Err(err) => {
                    // println!("Error in idle {:?}", err);
                    Err(())
                }
            }
        },
        gdbstub::stub::state_machine::GdbStubStateMachine::Running(mut gdb_stub_state_machine_inner) => {
            use run_blocking::Event as BlockingEventLoopEvent;
            use run_blocking::WaitForStopReasonError;

            // println!("Running");

            // block waiting for the target to return a stop reason
            let event = <SimpleGdbBlockingEventLoop<T> as run_blocking::BlockingEventLoop>::
                wait_for_stop_reason(target, gdb_stub_state_machine_inner.borrow_conn());

            match event {
                Ok(BlockingEventLoopEvent::TargetStopped(stop_reason)) => {
                    // println!("Running - Got target stopped");
                    let gdb_res = gdb_stub_state_machine_inner.report_stop(target, stop_reason);
                    if let Ok(gdb_ok) = gdb_res {
                        Ok(gdb_ok)
                    }
                    else {
                        Err(())
                    }
                }

                Ok(BlockingEventLoopEvent::IncomingData(byte)) => {
                    if byte.is_ascii_graphic() {
                        let ch: char = byte.try_into().unwrap();
                        // println!("Running - Got byte {}", ch);
                    }
                    else {
                        // println!("Running - Got byte {}", byte);
                    }
                    let gdb_res = gdb_stub_state_machine_inner.incoming_data(target, byte);
                    if let Ok(gdb_ok) = gdb_res {
                        Ok(gdb_ok)
                    }
                    else {
                        Err(())
                    }
                }

                Err(WaitForStopReasonError::Target(_e)) => {
                    // println!("Running - Got target");
                    // break Err(InternalError::TargetError(e).into());
                    Err(())
                }
                Err(WaitForStopReasonError::Connection(_e)) => {
                    // println!("Running - Got connection");
                    // break Err(InternalError::conn_read(e).into());
                    Err(())
                }
            }
        },
        gdbstub::stub::state_machine::GdbStubStateMachine::CtrlCInterrupt(_gdb_stub_state_machine_inner) => {
            println!("Ctrlc");
            Err(())
        },
        gdbstub::stub::state_machine::GdbStubStateMachine::Disconnected(_gdb_stub_state_machine_inner) => {
            println!("Disconnected");
            Err(())
        }
    }
}
