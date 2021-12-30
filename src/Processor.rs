use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Add;
use std::rc::{Rc, Weak};
use InternalOperations::{ADD_INDEX_LO, ALU_INCR, DUMMY_FOR_OVERLAP, NOP};
use crate::Bus::{Address, Bus, BusDevice, Data};
use crate::Processor::AddressRegister::INTERNAL_ADDRESS;
use crate::Processor::DataRegister::INTERNAL_OPERAND;
use crate::Processor::InternalOperations::{FETCH_ADDR_HI, FETCH_ADDR_LO, FETCH_IMMEDIATE_OPERAND, FETCH_OPCODE, FETCH_OPERAND, JUMP_TO_ADDRESS, READ_FROM_ACCUMULATOR, STORE_TO_ACCUMULATOR, STORE_TO_REGISTER_X, WRITE_TO_ADDRESS};

struct memory {

}

pub trait ProcessorTrait: BusDevice {
    fn tick(&mut self);

    fn reset(&mut self);
}

#[derive(PartialEq)]
#[derive(Debug)]
enum DataRegister {
    X,
    Y,
    A,
    INTERNAL_OPERAND
}

#[derive(PartialEq)]
#[derive(Debug)]
enum AddressRegister {
    PC,
    INTERNAL_ADDRESS,
}

#[derive(PartialEq)]
#[derive(Debug)]
enum InternalOperations {
    NOP,
    DUMMY_FOR_OVERLAP,
    FETCH_OPCODE,
    FETCH_OPERAND,
    FETCH_ADDR_LO,
    FETCH_ADDR_HI,
    FETCH_IMMEDIATE_OPERAND,
    STORE_TO_ACCUMULATOR{src: DataRegister},
    WRITE_TO_ADDRESS{src: DataRegister, addr: AddressRegister},
    JUMP_TO_ADDRESS,
    STORE_TO_REGISTER_X,
    READ_FROM_ACCUMULATOR,
    ADD_INDEX_LO,
    ALU_INCR
}

// Implementation of an instruction. addressing mode specific
struct Instruction {
    mnemonic: String,
    operations: Vec<InternalOperations>,
    canOverlapWithNextFetch: bool,
}


struct InstructionExecution {
    instruction: Instruction,
    internalOperationStream: Vec<InternalOperations>,
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
    operationStream: Vec<InternalOperations>,
    instructions: HashMap<u8, Instruction>,
}

pub fn create6502(bus: Rc<RefCell<dyn Bus>>) -> Rc<RefCell<dyn ProcessorTrait>> {
    let mut map_o_instructions: HashMap<u8, Instruction> = HashMap::new();
    map_o_instructions.insert(0xea, Instruction {
        mnemonic: "NOP".to_string(),
        operations: vec![FETCH_OPCODE, NOP],
        canOverlapWithNextFetch: false,
    });
    map_o_instructions.insert(0xa9, Instruction {
        mnemonic: "LDA #Oper".to_string(),
        operations: vec![FETCH_OPCODE, FETCH_OPERAND, STORE_TO_ACCUMULATOR { src: INTERNAL_OPERAND }],
        canOverlapWithNextFetch: true
    });
    map_o_instructions.insert(0x8d, Instruction {
        mnemonic: "STA Oper".to_string(),
        operations: vec![FETCH_OPCODE, FETCH_ADDR_LO, FETCH_ADDR_HI, WRITE_TO_ADDRESS { src: INTERNAL_OPERAND, addr: INTERNAL_ADDRESS }],
        canOverlapWithNextFetch: false
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
        operationStream: Vec::new(),
        instructions: map_o_instructions,
    };

    p.pc = 0x0FFC; // BOOT location
    let mut bootSeq = vec![FETCH_ADDR_LO, FETCH_ADDR_HI, JUMP_TO_ADDRESS];
    p.operationStream.append(&mut bootSeq);

    Rc::new(RefCell::new(p))

}

impl Proc6502 {
    fn get_reg(&self, reg: DataRegister) -> Data {
        match reg {
            DataRegister::X => return self.x,
            DataRegister::Y => return self.y,
            DataRegister::A => return self.a,
            INTERNAL_OPERAND => return self.internal_operand,
        }
    }
}

impl ProcessorTrait for Proc6502 {

    fn tick(&mut self) {
        let x = self.operationStream.pop().unwrap();
        match x {
            NOP => {}
            DUMMY_FOR_OVERLAP => {}
            FETCH_OPCODE => {}
            FETCH_OPERAND => {}
            FETCH_ADDR_LO => {
                self.internal_address = self.bus.upgrade().unwrap().borrow().read(self.pc) as Address;
                self.pc+=1;
            }
            FETCH_ADDR_HI => {
                self.internal_address |= (self.bus.upgrade().unwrap().borrow().read(self.pc) as Address) << 8;
                self.pc+=1;
            }
            FETCH_IMMEDIATE_OPERAND => {
                self.internal_operand = self.bus.upgrade().unwrap().borrow().read(self.pc);
            }
            STORE_TO_ACCUMULATOR{ src } => {
                assert_ne!(src, DataRegister::A);
                self.a = self.get_reg(src);
            }
            WRITE_TO_ADDRESS{ src, addr } => {

            }
            JUMP_TO_ADDRESS => {}
            STORE_TO_REGISTER_X => {}
            READ_FROM_ACCUMULATOR => {}
            ADD_INDEX_LO => {}
            ALU_INCR => {}
        }
    }

    // TODO should tick through the
    fn reset(&mut self) {

    }

}

impl BusDevice for Proc6502 {
    fn doRead(&self, _: Address) -> Data {
        panic!("I can not be read from");
    }

    fn doWrite(&mut self, _: Address, _: Data) {
        panic!("I can not be written to");
    }


    fn isReadableFor(&self, _: Address) -> bool {
        false
    }

    fn isWritableFor(&self, _: Address) -> bool {
        false
    }
}
