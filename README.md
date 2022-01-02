# rust-6502-sim
Emulate a 6502 processor  Learning rust the wrong way!

Abstractions:
- Bus
- BusDevice
- Processor
- Memory: BusDevice
- Instruction
- InternalOperation

The two BusDevice's implemented are the Proc6502 and Memory.

Instructions Implemented
- NOP
- JMP $ 
- LDA #
- STA $

I am using this [low level 6502 instruction set document](https://www.nesdev.com/6502_cpu.txt) as a guide.
