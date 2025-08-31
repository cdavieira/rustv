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
