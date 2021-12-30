use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::collections::HashMap;

use crate::bus::{Address, Bus, BusDevice, Data};
use crate::processor::InternalOperations::*;
use crate::processor::DataRegister::*;
use crate::processor::AddressRegister::*;

pub trait ProcessorTrait: BusDevice {
    fn tick(&mut self);

    fn reset(&mut self);
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum DataRegister {
    X,
    Y,
    A,
    InternalOperand
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum AddressRegister {
    PC,
    InternalAddress,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum InternalOperations {
    NOP,
    DummyForOverlap,
    FetchOpcode,
    FetchOperand,
    FetchAddrLo,
    FetchAddrHi,
    FetchImmediateOperand,
    StoreToAccumulator{src: DataRegister},
    WriteToAddress{src: DataRegister, addr: AddressRegister},
    JumpToAddress,
    StoreToRegisterX,
    ReadFromAccumulator,
    AddIndexLo,
    AluIncr
}

// Implementation of an instruction. addressing mode specific
struct Instruction {
    mnemonic: String,
    operations: Vec<InternalOperations>,
    can_overlap_with_next_fetch: bool,
}


struct InstructionExecution {
    instruction: Instruction,
    internal_operation_stream: Vec<InternalOperations>,
}

pub struct Proc6502 {
    bus: Weak<RefCell<dyn Bus>>,
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

pub fn create6502(bus: Rc<RefCell<dyn Bus>>) -> Rc<RefCell<dyn BusDevice>> {
    let mut map_o_instructions: HashMap<u8, Instruction> = HashMap::new();
    map_o_instructions.insert(0xea, Instruction {
        mnemonic: "NOP".to_string(),
        operations: vec![FetchOpcode, NOP],
        can_overlap_with_next_fetch: false,
    });
    map_o_instructions.insert(0xa9, Instruction {
        mnemonic: "LDA #Oper".to_string(),
        operations: vec![FetchOpcode, FetchOperand, StoreToAccumulator { src: InternalOperand }],
        can_overlap_with_next_fetch: true
    });
    map_o_instructions.insert(0x8d, Instruction {
        mnemonic: "STA Oper".to_string(),
        operations: vec![FetchOpcode, FetchAddrLo, FetchAddrHi, WriteToAddress { src: InternalOperand, addr: InternalAddress }],
        can_overlap_with_next_fetch: false
    });

    let mut p =  Proc6502 {
        bus: Rc::downgrade(&bus),
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
    let mut boot_seq = vec![FetchAddrLo, FetchAddrHi, JumpToAddress];
    p.operation_stream.append(&mut boot_seq);

    Rc::new(RefCell::new(p))

}

impl Proc6502 {
    fn get_reg(&self, reg: DataRegister) -> Data {
        match reg {
            DataRegister::X => self.x,
            DataRegister::Y => self.y,
            DataRegister::A => self.a,
            InternalOperand => self.internal_operand,
        }
    }

    fn get_addr_reg(&self, reg: AddressRegister) -> Address {
        match reg {
            PC => self.pc,
            InternalAddress => self.internal_address,
        }
    }
}

impl ProcessorTrait for Proc6502 {

    fn tick(&mut self) {
        let x = self.operation_stream.pop().unwrap();
        match x {
            NOP => {}
            InternalOperations::DummyForOverlap => {}
            FetchOpcode => {}
            FetchOperand => {}
            FetchAddrLo => {}
            FetchAddrHi => {}
            InternalOperations::FetchImmediateOperand => {}
            InternalOperations::StoreToAccumulator { .. } => {}
            InternalOperations::WriteToAddress { .. } => {}
            JumpToAddress => {}
            InternalOperations::StoreToRegisterX => {}
            InternalOperations::ReadFromAccumulator => {}
            InternalOperations::AddIndexLo => {}
            InternalOperations::AluIncr => {}
        }
        match x {
            NOP => {}
            DummyForOverlap => {}
            FetchOpcode => {}
            FetchOperand => {}
            FetchAddrLo => {
                self.internal_address = self.bus.upgrade().unwrap().borrow().read(self.pc) as Address;
                self.pc+=1;
            }
            FetchAddrHi => {
                self.internal_address |= (self.bus.upgrade().unwrap().borrow().read(self.pc) as Address) << 8;
                self.pc+=1;
            }
            FetchImmediateOperand => {
                self.internal_operand = self.bus.upgrade().unwrap().borrow().read(self.pc);
            }
            StoreToAccumulator{ src } => {
                assert_ne!(src, DataRegister::A);
                self.a = self.get_reg(src);
            }
            WriteToAddress{ src, addr } => {
                self.bus.upgrade().unwrap().borrow().write(self.get_addr_reg(addr), self.get_reg(src));

            }
            JumpToAddress => {}
            StoreToRegisterX => {}
            ReadFromAccumulator => {}
            AddIndexLo => {}
            AluIncr => {}
        }
    }

    // TODO should tick through the
    fn reset(&mut self) {

    }

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
