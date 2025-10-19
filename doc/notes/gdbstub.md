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

Chapter 20 (P. 315) talks about the gdb remote server and gdbstubs

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

For example, in case the user enters `si`, here's the flow of functions/methods called:
0. At some point, the gdb client sends to the stub the command associated with
   `si` (which likely is `$vCont;s:{ThreadId}#{checksum}` or `$vCont;c:{ThreadId}#{checksum}`)
1. The stub parses which byte sent over the connection through the
   `incoming_data()` method until the packet is complete
2. Once completed, `Packet::from_buf()` gets called, which translates that
   sequence of bytes into a type known to its type implementation of the
Gdbstub crate. That type is then handled by `handle_packet` -> `handle_command`
-> `handle_stop_resume` -> `do_vcont` -> `do_vcont_single_thread` -> `step` or
`resume`.
Depending on the result of the packet handling, the state of the Gdbstub
becomes 'Running' through a call to `transition()` and is followed by
`report_stop()`
3. Before handing the control over to the Target, the call to `report_stop()`
   makes the stub communicate to the gdb client the reason for why it has
stopped
4. Once in the Running state, the target takes over the control. Later on, it
   decides to stop and returns the reason for that to the stub.

One thing i've noticed is that the command `si` of gdb doesn't translate into
an actual `step` procedure sent over the transmission channel, but rather as a
sequence of basically two operations: the insertion of a software breakpoint at
the next address, followed by a `continue` procedure sent over the transmission
channel. According to chatgpt, the `step` procedure is meant to be used for
when the target supports hardware breakpoints, which is not the case for my
debugger (at least for now). Since only software breakpoints are currently
supported, this effectively means that the 'step' method of my debugger won't
ever be issued, which means that its internal state won't ever be 'Stepping'

---

How the communication between the gdbclient and stub works under the hood

Logs marked with 'recv_packet' correspond to fully-formed packets coming from
the gdb client and sent to the stub

Logs marked with 'response_writer' correspond to the response of the stub to
the gdbclient for the previously packet sent

Logs after running `gdb> target remote :9999` (Handshake)
```
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- +
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qSupported:multiprocess+;swbreak+;hwbreak+;qRelocInsn+;fork-events+;vfork-events+;exec-events+;vContSupported+;QThreadEvents+;QThreadOptions+;no-resumed+;memory-tagging+;error-message+#89
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $PacketSize=1000;vContSupported+;multiprocess+;QStartNoAckMode+;swbreak+;qXfer:features:read+#fd
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- +
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $vCont?#49
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $vCont;c;C;s;S#62
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- +
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $vMustReplyEmpty#3a
[2025-10-15T13:21:14Z INFO  gdbstub::stub::core_impl] Unknown command: Ok("vMustReplyEmpty")
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $#00
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- +
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $QStartNoAckMode#b0
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- +
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $Hgp0.0#ad
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qXfer:features:read:target.xml:0,ffb#79
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $m<?xml version="1.0"?>
    <!-- Copyright (C) 2018-2024 Free Software Foundation, Inc.
    
         Copying and distribution of this file, with or without modification,
         are permitted in any medium without royalty provided the copyright
         notice and this notice are preserved.  -->
    
    <!-- Register numbers are hard-coded in order to maintain backward
         compatibility with older versions of tools that didn't use xml
         register descriptions.  -->
    
    <!DOCTYPE feature SYSTEM "gdb-target.dtd">
    <feature name="org.gnu.gdb.riscv.cpu">
      <reg name="zero" bitsize="32" type="int" regnum="0"/>
      <reg name="ra" bitsize="32" type="code_ptr"/>
      <reg name="sp" bitsize="32" type="data_ptr"/>
      <reg name="gp" bitsize="32" type="data_ptr"/>
      <reg name="tp" bitsize="32" type="data_ptr"/>
      <reg name="t0" bitsize="32" type="int"/>
      <reg name="t1" bitsize="32" type="int"/>
      <reg name="t2" bitsize="32" type="int"/>
      <reg name="fp" bitsize="32" type="data_ptr"/>
      <reg name="s1" bitsize="32" type="int"/>
      <reg name="a0" bitsize="32" type="int"/>
      <reg name="a1" bitsize="32" type="int"/>
      <reg name="a2" bitsize="32" type="int"/>
      <reg name="a3" bitsize="32" type="int"/>
      <reg name="a4" bitsize="32" type="int"/>
      <reg name="a5" bitsize="32" type="int"/>
      <reg name="a6" bitsize="32" type="int"/>
      <reg name="a7" bitsize="32" type="int"/>
      <reg name="s2" bitsize="32" type="int"/>
      <reg name="s3" bitsize="32" type="int"/>
      <reg name="s4" bitsize="32" type="int"/>
      <reg name="s5" bitsize="32" type="int"/>
      <reg name="s6" bitsize="32" type="int"/>
      <reg name="s7" bitsize="32" type="int"/>
      <reg name="s8" bitsize="32" type="int"/>
      <reg name="s9" bitsize="32" type="int"/>
      <reg name="s10" bitsize="32" type="int"/>
      <reg name="s11" bitsize="32" type="int"/>
      <reg name="t3" bitsize="32" type="int"/>
      <reg name="t4" bitsize="32" type="int"/>
      <reg name="t5" bitsize="32" type="int"/>
      <reg name="t6" bitsize="32" type="int"/>
      <reg name="pc" bitsize="32" type="code_ptr"/>
    </feature>#82
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qXfer:features:read:target.xml:7d3,ffb#17
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $l#6c
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qTStatus#49
[2025-10-15T13:21:14Z INFO  gdbstub::stub::core_impl] Unknown command: Ok("qTStatus")
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $#00
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $?#3f
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $T05thread:p01.01;#06
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qfThreadInfo#bb
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $mp01.01#cd
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qsThreadInfo#c8
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $l#6c
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qAttached:1#fa
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $1#31
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $Hc-1#09
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qOffsets#4b
[2025-10-15T13:21:14Z INFO  gdbstub::stub::core_impl] Unknown command: Ok("qOffsets")
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $#00
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $g#67
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000#6a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,2#c5
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10056,2#c7
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10058,2#c9
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1005a,2#f2
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1005c,2#f4
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1005e,2#f6
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10060,2#c2
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10062,2#c4
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10064,2#c6
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10066,2#c8
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10068,2#ca
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1006a,2#f3
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1006c,2#f5
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1006e,2#f7
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10070,2#c3
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10072,2#c5
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10074,2#c7
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10076,2#c9
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10078,2#cb
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1007a,2#f4
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1007c,2#f6
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1007e,2#f8
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10080,2#c4
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10082,2#c6
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10084,2#c8
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10086,2#ca
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10088,2#cc
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1008a,2#f5
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1008c,2#f7
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1008e,2#f9
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10090,2#c5
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10092,2#c7
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10094,2#c9
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10096,2#cb
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10098,2#cd
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1009a,2#f6
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1009c,2#f8
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m1009e,2#fa
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100a0,2#ed
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100a2,2#ef
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100a4,2#f1
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100a6,2#f3
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100a8,2#f5
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100aa,2#1e
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100ac,2#20
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100ae,2#22
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100b0,2#ee
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100b2,2#f0
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100b4,2#f2
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100b6,2#f4
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m10080,40#f6
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000#6c
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m100b8,2#f6
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qfThreadInfo#bb
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $mp01.01#cd
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qsThreadInfo#c8
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $l#6c
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $m0,4#fd
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $00000000#7e
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $mfffffffc,4#fa
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $#00
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::recv_packet] <-- $qSymbol::#5b
[2025-10-15T13:21:14Z INFO  gdbstub::stub::core_impl] Unknown command: Ok("qSymbol::")
[2025-10-15T13:21:14Z TRACE gdbstub::protocol::response_writer] --> $#00
```

Logs after running `gdb> load`

* `$P20=54000100#79` (page 783):
Write register 20 with value 54000100

* `$X10054,0:#e8` (page 787):
Write data (between : and #) to 0 bytes from memory starting
at address 10054

* `$G00...#d1` (page 782):
Write data (0, 0, ...) to general registers

* `$m10054,2#c5` (page 782):
Read 2 bytes from memory starting at address 10054

```
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $X10054,0:#e8
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $X10054,24:0����>s3�g�#a1
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $P20=54000100#79
[2025-10-15T13:22:45Z INFO  gdbstub::stub::core_impl] Unknown command: Ok("P20=54000100")
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $#00
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $G000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000054000100#d1
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,2#c5
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $1303#c7
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m10056,2#c7
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $3000#c3
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m10058,2#c9
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $1305#c9
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m1005a,2#f2
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m1005c,2#f4
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $9305#d1
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m1005e,2#f6
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $1000#c1
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m10060,2#c2
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $ef00#2b
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m10062,2#c4
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $0001#c1
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m10040,40#f2
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $0000000000000000000000000000000000000000130330001305000093051000ef0000019308d0051305803e730000003305b500678000000000000000000000#11
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::recv_packet] <-- $m10060,4#c4
[2025-10-15T13:22:45Z TRACE gdbstub::protocol::response_writer] --> $ef000001#77
```

Logs after running `gdb> b _start`
```
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,2#c5
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $1303#c7
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m10056,2#c7
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $3000#c3
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m10058,2#c9
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $1305#c9
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m1005a,2#f2
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $0000#7a
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m1005c,2#f4
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $9305#d1
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m1005e,2#f6
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $1000#c1
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m10060,2#c2
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $ef00#2b
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m10062,2#c4
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $0001#c1
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::recv_packet] <-- $m10060,4#c4
[2025-10-15T13:23:15Z TRACE gdbstub::protocol::response_writer] --> $ef000001#77
```

Logs after running `gdb> si`
* `$vCont;c:p1.-1#0f` (page 784)
request thread 'p1.-1' to continue (c)

* `$Z0,100060,4#0d` (page 788)
insert a software breakpoint starting at 100060, with 4 bytes length
> A software breakpoint is implemented by replacing the instruction at addr with
> a software breakpoint or trap instruction. The kind is target-specific and typi-
> cally indicates the size of the breakpoint in bytes that should be inserted. E.g.,
> the arm and mips can insert either a 2 or 4 byte breakpoint

* `$T05thread:p01.01;swbreak:;#6a` (page 790)
The program received signal 05 and thread 'p01.01' is the stopped thread.
'swbreak' is the reason why the target stopped, which in this case means a
software breakpoint instruction.
> The program received signal number AA (a two-digit hexadecimal number).
This is equivalent to an ‘S’ response, except that the ‘n:r’ pairs can carry values
of important registers and other information directly in the stop reply packet,
reducing round-trip latency. Single-step and breakpoint traps are reported this
way. Each ‘n:r’ pair is interpreted as follows:

* `$g#..` (page 781)
read general registers

* `$s..#` (page 783)
single step, resuming at that address

```
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,4#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $13033000#8a
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10050,4#c3
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $00000000#7e
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $Z0,10060,4#0d
Trying to add a sw breakpoint at 65632 4
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,2#c5
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $1303#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10056,2#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $3000#c3
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,2#c5
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $1303#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10056,2#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $3000#c3
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10058,4#cb
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $13050000#43
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $Z0,10058,4#14
Trying to add a sw breakpoint at 65624 4
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $vCont;c:p1.-1#0f
Calling 'resume' (Now Running)
[2025-10-15T13:23:29Z TRACE gdbstub::stub::state_machine] transition: "Idle<rustv::emu::debugger::SimpleTarget<rustv::emu::machine::SimpleMachine>>" --> "Running"
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $T05thread:p01.01;swbreak:;#6a
[2025-10-15T13:23:29Z TRACE gdbstub::stub::state_machine] transition: "Running" --> "Idle<rustv::emu::debugger::SimpleTarget<rustv::emu::machine::SimpleMachine>>"
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $g#67
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000058000100#98
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $z0,10058,4#34
Trying to rm a sw breakpoint at 65624 4
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10040,40#f2
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $0000000000000000000000000000000000000000130330001305000093051000ef0000019308d0051305803e730000003305b500678000000000000000000000#11
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10058,4#cb
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $13050000#43
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10058,4#cb
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $13050000#43
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,4#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $13033000#8a
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,2#c5
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $1303#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10056,2#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $3000#c3
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $qfThreadInfo#bb
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $mp01.01#cd
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $qsThreadInfo#c8
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $l#6c
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $z0,10060,4#2d
Trying to rm a sw breakpoint at 65632 4
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10058,4#cb
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $13050000#43
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,4#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $13033000#8a
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10054,2#c5
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $1303#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::recv_packet] <-- $m10056,2#c7
[2025-10-15T13:23:29Z TRACE gdbstub::protocol::response_writer] --> $3000#c3
```


Logs after disconnecting:
```
[2025-10-15T13:58:17Z TRACE gdbstub::protocol::recv_packet] <-- $D;1#b0
[2025-10-15T13:58:17Z TRACE gdbstub::protocol::response_writer] --> $OK#9a
[2025-10-15T13:58:17Z TRACE gdbstub::stub::state_machine] transition: "Idle<rustv::emu::debugger::SimpleTarget<rustv::emu::machine::SimpleMachine>>" --> "Disconnected"
```
