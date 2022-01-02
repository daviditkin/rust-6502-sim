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
struct Instruction {
    pub mnemonic: String,
    pub operations: Vec<InternalOperations>,
    pub addressing: AddressingMode,
    pub can_overlap_with_next_fetch: bool,
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

pub fn fetch_operations_for_mode(mode: AddressingMode) -> Vec<InternalOperations> {
    match mode {
        Accumulator => vec![],
        Absolute => vec![FetchAddrLo, FetchAddrHi],
        AbsIndexed { reg } => vec![FetchAddrLo, FetchAddrHi, IncrementAddressByReg { reg }],
        Immediate => vec![FetchImmediateOperand],
        Implied => vec![],
        Indirect => vec![FetchAddrLo, FetchAddrHi, FetchOperand],
        IndexedIndirect => todo!(),
        IndirectIndexed => todo!(),
        Relative => todo!(),
        ZeroPage => vec![FetchZeroPageAddr],
        ZeroPageIndexed { reg } => todo!(),
    }
}

pub fn concat_operations(
    v1: Vec<InternalOperations>,
    v2: Vec<InternalOperations>,
) -> Vec<InternalOperations> {
    let mut v: Vec<InternalOperations> = vec![];
    v.extend(v1.iter().copied());
    v.extend(v2.iter().copied());
    v
}

pub fn create6502(bus: Rc<RefCell<dyn Bus>>) -> Proc6502 {
    let mut map_o_instructions: HashMap<u8, Instruction> = HashMap::new();
    map_o_instructions.insert(
        0xea,
        Instruction {
            mnemonic: "NOP".to_string(),
            operations: vec![NOP],
            addressing: Implied,
            can_overlap_with_next_fetch: false,
        },
    );
    map_o_instructions.insert(
        0xa9,
        Instruction {
            mnemonic: "LDA".to_string(),
            addressing: Absolute,
            operations: concat_operations(
                fetch_operations_for_mode(Absolute),
                vec![StoreToAccumulator {
                    src: InternalOperand,
                }],
            ),
            can_overlap_with_next_fetch: true,
        },
    );
    map_o_instructions.insert(
        0x8d,
        Instruction {
            mnemonic: "STA Oper".to_string(),
            addressing: Absolute,
            operations: concat_operations(
                fetch_operations_for_mode(Absolute),
                vec![WriteToAddress {
                    src: A,
                    addr: InternalAddress,
                }],
            ),
            can_overlap_with_next_fetch: false,
        },
    );
    map_o_instructions.insert(
        0x4c,
        Instruction {
            mnemonic: "JMP $XXXX".to_string(),
            operations: concat_operations(fetch_operations_for_mode(Absolute), vec![JumpToAddress]),
            addressing: Absolute,
            can_overlap_with_next_fetch: false,
        },
    );

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


fn read_instructions() -> HashMap<u8, Instruction> {
    todo!()
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
            FetchZeroPageAddr => {}
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
