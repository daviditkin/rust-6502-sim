
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::collections::HashMap;

use crate::bus::{Address, Bus, BusDevice, Data};
use crate::processor::InternalOperations::*;
use crate::processor::DataRegister::*;
use crate::processor::AddressRegister::*;

pub trait ProcessorTrait: BusDevice {
    fn tick(&mut self, bus: Rc<RefCell<dyn Bus>>);

    fn reset(&mut self);
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum DataRegister {
    X,
    Y,
    A,
    InternalOperand
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum AddressRegister {
    PC,
    InternalAddress,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum InternalOperations {
    NOP,
    DummyForOverlap,
    FetchOpcode,
    FetchOperand,
    FetchAddrLo,
    FetchAddrHi,
    FetchImmediateOperand,
    StoreToAccumulator { src: DataRegister },
    WriteToAddress { src: DataRegister, addr: AddressRegister },
    JumpToAddress,
    StoreToRegisterX,
    ReadFromAccumulator,
    AddIndexLo,
    AluIncr,
}

// Implementation of an instruction. addressing mode specific
struct Instruction {
    pub mnemonic: String,
    pub operations: Vec<InternalOperations>,
    pub can_overlap_with_next_fetch: bool,
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

pub fn create6502(bus: Rc<RefCell<dyn Bus>>) -> Rc<RefCell<Proc6502>> {
    let mut map_o_instructions: HashMap<u8, Instruction> = HashMap::new();
    map_o_instructions.insert(0xea, Instruction {
        mnemonic: "NOP".to_string(),
        operations: vec![NOP],
        can_overlap_with_next_fetch: false,
    });
    map_o_instructions.insert(0xa9, Instruction {
        mnemonic: "LDA #Oper".to_string(),
        operations: vec![FetchOperand, StoreToAccumulator { src: InternalOperand }],
        can_overlap_with_next_fetch: true
    });
    map_o_instructions.insert(0x8d, Instruction {
        mnemonic: "STA Oper".to_string(),
        operations: vec![FetchAddrLo, FetchAddrHi, WriteToAddress { src: InternalOperand, addr: InternalAddress }],
        can_overlap_with_next_fetch: false
    });
    map_o_instructions.insert(0x4c, Instruction {
        mnemonic: "JMP $XXXX".to_string(),
        operations: vec![FetchAddrLo, FetchAddrHi, JumpToAddress],
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
    p.operation_stream.extend(vec![FetchAddrLo, FetchAddrHi, JumpToAddress].iter().copied());

    Rc::new(RefCell::new(p))
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

    pub fn as_cloned_bus_device(&self, foo: Rc<RefCell<Proc6502>>) -> Rc<RefCell<dyn BusDevice>> {
        let rc:Rc<RefCell<dyn BusDevice>> = foo;
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
                // todo test for illegal opcode
                let instruction = self.instructions.get(&(opcode as u8)).unwrap();
                println!("Excecuting {} ", instruction.mnemonic);
                self.operation_stream.extend(instruction.operations.iter().copied());
                self.pc += 1;
            }
            FetchOperand => {}
            FetchAddrLo => {
                self.internal_address &= 0xff00;
                self.internal_address = the_bus.borrow().read(self.pc) as Address;
                self.pc+=1;
            }
            FetchAddrHi => {
                self.internal_address &= 0x00ff;
                self.internal_address |= (the_bus.borrow().read(self.pc) as Address) << 8;
                self.pc+=1;
            }
            FetchImmediateOperand => {
                self.internal_operand = the_bus.borrow().read(self.pc);
            }
            StoreToAccumulator{ src } => {
                assert_ne!(src, DataRegister::A);
                self.a = self.get_reg(&src);
            }
            WriteToAddress{ src, addr } => {
                the_bus.borrow().write(self.get_addr_reg(&addr), self.get_reg(&src));

            }
            JumpToAddress => {
                self.pc = self.internal_address;
            }
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
