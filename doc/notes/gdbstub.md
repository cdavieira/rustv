NOTES ON GDBSTUB RUST 

1. mystub.run_state_machine(): ensures software breakpoints are enabled
(supported) by the target

2. mystub.run_state_machine(): setup the state machine and conection

3. idontknow(): handles one of the 4 states the debugger state machine is
(idle, running, ctrlc, disconnected)

4. idontknow(): if idle, then read 1 byte as encoded for the GDB remote
protocol

5. gdb_stub_state_machine_inner.incoming_data(): (idle) handle that 1 byte
through gdb_stub_state_machine_inner.handle_packet()

6. gdb_stub_state_machine_inner.handle_packet(): (idle) the packet might be
either an ACK, an NACK, an Interrupt (Ctrlc) or a Command. 

7. gdb_stub_state_machine_inner.handle_command(): commands (Base, Resume,
SingleRegisterAccess, Breakpoints, CatchSyscalls, Tracepoints, Memory maps) get
handled
> these are the features to be enabled/negotiated with gdb (a part of the protocol)

8. after all features get negotiated, gdb starts running, which means that
   'idontknow()' starts handling 'Running'

---

NOTES AFTER READING THE GDB OFICIAL DOCS 

Chapter 20 talks about the gdb remote server and gdbstubs

the host -> the program running gdb (my x86-64 computer in this case), which
controls the execution state of the program running remotely through the
gdbstub (which is running remotely as well)

the guest -> the program running the gdbstub (the custom gdbstub i wrote for
riscv32, which is exposed through TCP) and which talks back to the gdb client
running on the host 

gdbserver is a gdbstub 'replacement' which is lighter than a fully
reimplementation of gdb and which can control the execution of a program
running locally whilst still being himself controled by a gdb instance running
remotely (in the host)

unstripped program -> a program which still has symbols and possibly debug information
stripped program -> a program without debug information/symbols

i might be wrong on this, but apparently the gdbstub can interrupt itself and
return control to gdb at any times. Maybe this could allows us to implement
'software' breakpoints (i'm not sure if that's what the actual software
breakpoints found in the rust gdbstub project are about)

Appendix E (Page 777) talks about the gdb serial remote protocol

the host (the program where gdb is running and relying on a gdbstub/gdbserver
running remotely to inspect a program (which is also running remotely)) sends
commands and the target (the gdbstub for example) sends a response

for step/continue commands, the target only returns control when all operations
have been completed

Last paragraph of E.1:
> At a minimum, a stub is required to support the ‘?’ command to tell gdb the reason
> for halting, ‘g’ and ‘G’ commands for register access, and the ‘m’ and ‘M’ commands for
> memory access. Stubs that only control single-threaded targets can implement run control
> with the ‘c’ (continue) command, and if the target architecture supports hardware-assisted
> single-stepping, the ‘s’ (step) command. Stubs that support multi-threading targets should
> support the ‘vCont’ command. All other commands are optiona

---

NOTES ON HOW GDB BEHAVES WHEN COMMUNICATING WITH THE STUB

initially, gdb sends a bunch of commands through the communication channel
(serial and through TCP in my case), encoded according to the GDB remote
protocol, trying to get some information about the remote target.

some important stuff that gdb asks:
* qOffsets (page 799 of the gdb doc): get offset of text, data segments
* g: read general registers
* read first word address (0)
* read last word address for that architecture (for 32bits: 4*1024\*1024\*1024 - 4)
> this is gdb probably trying to understand the memory boundaries of the remote
> target (where gdbstub is running, which in my case is Executor entity/my
> emulator/debugger/the gdbstub i wrote in rust)
>> TODO: read the 'read_memory' callback description to know what to return in
>> order to indicate that the memory offset doesn't exist
* qSymbols: Notify the target that gdb is prepared to serve symbol lookup requests
> in my case, this is not true, because currently i cant compile objects with
> debugging information. I'm going to rely on the gdbstub to manage 'software
> breakpoints' and to return control to the gdb instance at certain given
> times.

---

NOTES ON WHAT TO DO IN GDB AFTER SUCCESSFULLY CONNECTING WITH THE STUB

after everything is correct:
* load the program into the memory with the command `load` in the gdb instance
* watch if the memory was sucessfully written (you can do this either by
inspecting the memory in gdb or by logging this information in the stub's
stdout)

To inspect the stub's remote environment:
* `info registers`: take a look at registers
* `info files`: take a look at where each section is
* `x/1xw 0x10074`: inspect 1 hex word (4 bytes) at address 0x10074

after this, you should be able to run commands such as `s`/`c` in gdb

when entering `s`, gdb will:
* trigger the 'add_sw_breakpoint' in the stub
* invoke the 'resume' handler in the stub (because it has just resumed its
execution)
* invoke the 'wait_for_stop_reason' handler in the stub (this is where the stub
probably takes care of performing modifications related to instructions?) and
the stub is expected to return only after the operations have been completed or
a problem has occurred (therefore, the 'gdb instance' is waiting the 'gdbstub'
for it (gdbstub) to stop (a stop reason))

---

ON THINGS WHICH MIGHT HELP VISUALIZING THIS PROCESS
* an image which depicts the gdb instance running on one machine and the
gdbstub running on another machine remotely (and possibly the communication
channel which is serial and uses TCP and the GDB REMOTE PROTOCOL)
> notice that the gdb instance and gdbstub might be running in the same machine
>> the gdbstub could even be running in an emulated context (just like my rust program is doing!)

* an image depicting a simplified version of the initial negotiation between gdb and the gdbstub

* a documentation which explains how the state machine for the gdbstub works
(at least superficially). This includes what gets triggered when
'step/continue' is executed in gdb, what gets called in the code, how the
initial negotiation is handled in the code, how the gdbstub takes care of
modifying the emulated context once an instruction arrives. All of these things
are (somehow) documented here (somewhere)

---

ON HOW GDB TRANSFERS CONTROL BACK TO THE STUB WHEN THE USER RUNS THE STEP COMMAND

* after the user runs the step command, the gdb client hands control to the
stub, which gets set to the 'Running' state

* the stub then hands control to the target (the emulated context), which
proceeds its normal execution
> notice that the steps yet to be executed by the target are expected to be
> known by it at this point. This makes total sense, considering that the
> target should have the program loaded into memory
>> in code, this corresponds to a call to 'wait_for_stop_reason'

* the target eventually has to hand control back to the stub and this is done
by notifying the reason for why it has decided to stop the execution
> the stop reason lets the stub know why the target decided to stop its
> execution
>> in code, this corresponds to the implementation of the
>> 'wait_for_stop_reason' method
>>> all stop reasons can be found in the enum 'MultiThreadStopReason'

* once the stub gets handed back the control over the transmission, it informs
the gdb client whatever had just happened in the target side
> in code, this corresponds to a call to 'report_stop'
>> the stub might inform different things to the gdb client (the target decided
>> to stop, there's an incoming byte being transmitted, something happened to
>> the connection itself or with the target)

* after notifying the gdb client and if everything worked just fine, then the
stub goes back to the 'Idle' state and keeps on waiting for more instructions

* if the gdb client doesn't see a reason to stop its own execution (or
whatever), it hands control back to the stub, waking it up once more and
setting its state to Running

* this process keeps happening until the machine informs there's nothing else
to be done

---

GDBSTUB SUMMARY
When the Gdb client is active and the Target is idle -> Gdbstub::State = Idle
When the Gdb client is waiting and the Target is running -> Gdbstub::State = Running
If the Gdb client disconnects -> Gdbstub::State = Disconnected
If the Target stops because of a ctrlc -> Gdbstub::State = CtrlCInterrupt

While the target is running, different things might happen to it:
* It might run normally
* It might stop for some reason
* It might return control to the gdbstub when it detects new data sent by the Gdbclient
* It might lose the connection to the Gdbclient
* It might break for some reason (an Error might occurr)

When Gdbstub::State = Idle, the stub listens to what the Gdbclient is sending
over the (Socket) connection.
1. Each byte sent over the connection is handled (one at a time)
2. If the byte was read correctly, then it is passed down to the
   'incoming_data' method, otherwise a custom gdbstub error is generated

The 'incoming_data' is called whenever a new byte arrives on the transmission
channel and it manages the internal state of the Gdbstub. That means it might
transition the Gsbstub::State to either Idle, Running, Disconnected or CtrlCInterrupt.

To do that, it calls the 'pump' method, which basically builds a packet out of
each individual byte transmitted. According to the Gdb protocol documentation,
the transmission of packets begin with '$' and end with '#' followed by two
checksum digits. 'pump' is a small state machine, which takes care of making
the stub keep on reading the transmission channel, while a message/packet is
still being transmitted.

When a packet is ready to be parsed, 'pump' returns a 'packet_buffer',
otherwise it returns None (indicating more bytes are under way and that the
Stub should not transition states).
> Additionally, 'pump' also traces packets that are ready to be handled through
> the 'trace!' macro!

The packet is handled through the 'handle_packet' method, which might require
the stub to:
* Keep on 'pump'ing for other packets ( this depends on the stub
implementation, but likely Gdbstub::State = Idle )
* Become disconnected ( Which means Gdbstub::State = Disconnected )
* Set the target to Run ( Which means Gdbstub::State = Running )
* Become CtrlCInterrupted ( Which means Gdbstub::State = CtrlCInterrupt )
* Issue an Error

'handle_packet' is what actually executes commands sent over the transmission
channel, and there you'll find in which conditions the Stub sets the Target to
'Run'. Basically, if the effect of the packet handling returns
HandlerStatus::DeferredStopReason, then Gdbstub::State = Running (indicating
that the Target is now running and that the control of the execution should be
shifted to it)

For example, the 'handle_core' method invoked inside of 'handle_packet' takes
care of all the handshake communication that takes place when the Gdbclient
first connects to the stub. Additionally, it takes care of handling the
basic/core operations required by the Gdb protocol specification (read
registers, read memory and ??? (i forgor))

Once the target is running, it might transfer the control back to the stub for
different reasons, such as:
* For the stub to manage reading incoming bytes in the transmission channel
* Because of a normal stop reason (step executed)
* Because of an internal error
* Because of a connection error

In case the Target stopped for a normal stop reason, then the Stub proceeds by
calling the 'report_stop' method, which in turn:
* Writes back to the Gdbclient through the communication/transmission channel,
informing that the Target has stopped 
* Sets Gdbstub::State to either Idle or Disconnected (which effectively means
that the control is returned to the Stub, so that it can keep on mediating the
communication between both parts (or not in case it gets disconnected))
