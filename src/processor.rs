use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bus::{Address, Bus, BusDevice, Data};
use crate::processor::AddressRegister::*;
use crate::processor::AddressingMode::*;
use crate::processor::DataRegister::*;
use crate::processor::InternalOperations::*;

pub trait ProcessorTrait: BusDevice {
    fn tick(&mut self, bus: Rc<RefCell<dyn Bus>>);

    fn reset(&mut self);
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum DataRegister {
    X,
    Y,
    A,
    InternalOperand,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum AddressRegister {
    PC,
    InternalAddress,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum InternalOperations {
    NOP,
    ReadAddressLo,
    ReadAddressHi,
    IncrementPCBySignedOperand,
    IncrementAddressByReg {
        reg: DataRegister,
    },
    DummyForOverlap,
    FetchOpcode,
    FetchOperand,
    FetchAddrLo,
    FetchAddrHi,
    FetchImmediateOperand,
    FetchZeroPageAddr,
    StoreToAccumulator {
        src: DataRegister,
    },
    WriteToAddress {
        src: DataRegister,
        addr: AddressRegister,
    },
    JumpToAddress,
    StoreToRegisterX,
    ReadFromAccumulator,
    AddIndexLo,
    AluIncr,
}

/**

A       Accumulator             OPC A           operand is AC (implied single byte instruction)
abs     absolute                OPC $LLHH       operand is address $HHLL *
abs,X   absolute, X-indexed     OPC $LLHH,X     operand is address; effective address is address incremented by X with carry **
abs,Y   absolute, Y-indexed     OPC $LLHH,Y     operand is address; effective address is address incremented by Y with carry **
#       immediate               OPC #$BB        operand is byte BB
impl    implied                 OPC             operand implied
ind     indirect                OPC ($LLHH)     operand is address; effective address is contents of word at address: C.w($HHLL)
X,ind   X-indexed, indirect     OPC ($LL,X)     operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
ind,Y   indirect, Y-indexed     OPC ($LL),Y     operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
rel     relative                OPC $BB         branch target is PC + signed offset BB ***
zpg     zeropage                OPC $LL         operand is zeropage address (hi-byte is zero, address = $00LL)
zpg,X   zeropage, X-indexed     OPC $LL,X       operand is zeropage address; effective address is address incremented by X without carry **
zpg,Y   zeropage, Y-indexed     OPC $LL,Y       operand is zeropage address; effective address is address incremented by Y without carry **

**/

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum AddressingMode {
    Accumulator,
    Absolute,
    AbsIndexed { reg: DataRegister },
    Immediate,
    Implied,
    Indirect,
    IndexedIndirect, // This is always X reg. ea is word (LL + X, LL + X + 1) w/o carry
    IndirectIndexed, // This is always Y reg. ea is word (LL, LL+1) + Y w carry
    Relative,
    ZeroPage,
    ZeroPageIndexed { reg: DataRegister },
}

// Implementation of an instruction. addressing mode specific
pub struct Instruction {
    pub mnemonic: String,
    pub operations: Vec<InternalOperations>,
    pub addressing: AddressingMode
}

pub struct Proc6502 {
    pc: Address,
    x: Data,
    y: Data,
    a: Data,
    internal_address: Address,
    internal_operand: Data,
    status: Data,
    operation_stream: Vec<InternalOperations>,
    instructions: HashMap<u8, Instruction>,
}

pub fn fetch_operations_for_mode(mode: &AddressingMode) -> Vec<InternalOperations> {
    match mode {
        Accumulator => vec![],
        Absolute => vec![FetchAddrLo, FetchAddrHi],
        AbsIndexed { reg } => vec![FetchAddrLo, FetchAddrHi, IncrementAddressByReg { reg: *reg }],
        Immediate => vec![FetchImmediateOperand],
        Implied => vec![],
        Indirect => vec![FetchAddrLo, FetchAddrHi, ReadAddressLo, ReadAddressHi],
        IndexedIndirect => vec![], // TODO
        IndirectIndexed => vec![], // TODO
        Relative => vec![FetchOperand, IncrementPCBySignedOperand],
        ZeroPage => vec![FetchZeroPageAddr],
        ZeroPageIndexed { reg } => vec![FetchZeroPageAddr, IncrementAddressByReg {reg: *reg}, FetchOperand],
    }
}

pub fn create_instructions(th_mnemonic: &str, modes: Vec<(u8, AddressingMode)>, the_operations: &[InternalOperations]) -> Vec<(u8, Instruction)> {
    let mut instructions: Vec<(u8, Instruction)> = vec!();

    for (opcode, mode) in modes.iter() {
        let mut operations = fetch_operations_for_mode(mode);
        operations.extend(the_operations);
        let x = Instruction {
            mnemonic: th_mnemonic.to_string(),
            operations,
            addressing: *mode,
        };
        instructions.push((*opcode, x));
    }
    instructions
}

pub fn create6502() -> Proc6502 {
    let mut map_o_instructions: HashMap<u8, Instruction> = HashMap::new();

    let lda_modes = vec![
        (0xa9, Immediate),
        (0xa5, ZeroPage),
        (0xb5, ZeroPageIndexed {reg: X}),
        (0xad, Absolute),
        (0xbd, AbsIndexed {reg: X}),
        (0xb9, AbsIndexed {reg: X}),
        (0xa1, IndexedIndirect),
        (0xb1, IndirectIndexed)
    ];
    let lda_instructions = vec![FetchOperand, StoreToAccumulator {src: InternalOperand}];

    let sta_modes = vec![
        (0x85, ZeroPage),
        (0x95, ZeroPageIndexed {reg: X}),
        (0x8d, Absolute),
        (0x8d, AbsIndexed {reg: X}),
        (0x99, AbsIndexed {reg: X}),
        (0x81, IndexedIndirect),
        (0x91, IndirectIndexed)
    ];
    let sta_instructions = vec![WriteToAddress {src: A, addr: InternalAddress}];

    let jmp_modes = vec![
        (0x4c, Absolute),
        (0x6c, Indirect)
    ];
    let jmp_instructions = vec![JumpToAddress];

    let nop_modes = vec![(0xea, Implied)];
    let nop_instructions = vec![NOP];

    map_o_instructions.extend(create_instructions("JMP", jmp_modes, &jmp_instructions));
    map_o_instructions.extend(create_instructions("LDA", lda_modes, &lda_instructions));
    map_o_instructions.extend(create_instructions("STA", sta_modes, &sta_instructions));
    map_o_instructions.extend(create_instructions("NOP", nop_modes, &nop_instructions));

    let mut p = Proc6502 {
        pc: 0x0FFC,
        x: 0,
        y: 0,
        a: 0,
        internal_address: 0,
        internal_operand: 0,
        status: 0,
        operation_stream: Vec::new(),
        instructions: map_o_instructions,
    };

    // Prime the operation_stream with the boot sequence
    p.pc = 0x0FFC; // BOOT location
    p.operation_stream.extend(
        vec![FetchAddrLo, FetchAddrHi, JumpToAddress]
            .iter()
            .copied(),
    );

    p
}

impl Proc6502 {
    fn get_reg(&self, reg: &DataRegister) -> Data {
        match reg {
            DataRegister::X => self.x,
            DataRegister::Y => self.y,
            DataRegister::A => self.a,
            InternalOperand => self.internal_operand,
        }
    }

    fn get_addr_reg(&self, reg: &AddressRegister) -> Address {
        match reg {
            PC => self.pc,
            InternalAddress => self.internal_address,
        }
    }

    pub fn as_cloned_bus_device(&self, me: Rc<RefCell<Proc6502>>) -> Rc<RefCell<dyn BusDevice>> {
        let rc: Rc<RefCell<dyn BusDevice>> = me;
        Rc::clone(&rc)
    }
}

impl ProcessorTrait for Proc6502 {
    fn tick(&mut self, the_bus: Rc<RefCell<dyn Bus>>) {
        if self.operation_stream.is_empty() {
            // fetch the opcode
            self.operation_stream.extend([FetchOpcode].iter().copied());

            // The end of some instructions imply that a fetch of the next opcode should be done in parallel TODO
        }

        let x = self.operation_stream.remove(0);
        match x {
            NOP => {}
            DummyForOverlap => {}
            FetchOpcode => {
                let opcode = the_bus.borrow().read(self.pc);
                // todo tests for illegal opcode
                let instruction = self.instructions.get(&(opcode as u8)).unwrap();
                println!("Excecuting {} ", instruction.mnemonic);
                self.operation_stream
                    .extend(instruction.operations.iter().copied());
                self.pc += 1;
            }
            FetchOperand => {
                self.internal_operand = the_bus.borrow().read(self.internal_address);
            }
            FetchAddrLo => {
                self.internal_address &= 0xff00;
                self.internal_address = the_bus.borrow().read(self.pc) as Address;
                self.pc += 1;
            }
            FetchAddrHi => {
                self.internal_address &= 0x00ff;
                self.internal_address |= (the_bus.borrow().read(self.pc) as Address) << 8;
                self.pc += 1;
            }
            FetchImmediateOperand => {
                self.internal_operand = the_bus.borrow().read(self.pc);
            }
            StoreToAccumulator { src } => {
                assert_ne!(src, DataRegister::A);
                self.a = self.get_reg(&src);
            }
            WriteToAddress { src, addr } => {
                the_bus
                    .borrow()
                    .write(self.get_addr_reg(&addr), self.get_reg(&src));
            }
            JumpToAddress => {
                self.pc = self.internal_address;
            }
            StoreToRegisterX => {}
            ReadFromAccumulator => {}
            AddIndexLo => {}
            AluIncr => {}
            InternalOperations::IncrementAddressByReg { .. } => {}
            FetchZeroPageAddr => {
                self.internal_address &= 0x0000;
                self.internal_address = the_bus.borrow().read(self.pc) as Address;
                self.pc += 1;
            }
            IncrementPCBySignedOperand => {}
            ReadAddressLo => {}
            ReadAddressHi => {}
        }
    }

    // TODO should tick through the
    fn reset(&mut self) {}
}

impl BusDevice for Proc6502 {
    fn do_read(&self, _: Address) -> Data {
        panic!("I can not be read from");
    }

    fn do_write(&mut self, _: Address, _: Data) {
        panic!("I can not be written to");
    }

    fn is_readable_for(&self, _: Address) -> bool {
        false
    }

    fn is_writable_for(&self, _: Address) -> bool {
        false
    }
}
