use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::bus::{Address, Bus, BusDevice, Data};
use crate::processor::AddressRegister::*;
use crate::processor::AddressingMode::*;
use crate::processor::DataRegister::*;
use crate::processor::Function::*;
use crate::processor::InternalOperations::*;

pub trait ProcessorTrait: BusDevice {
    fn tick(&mut self, bus: Rc<RefCell<dyn Bus>>) -> (Address, bool);

    fn reset(&mut self);

    fn get_user_cycles(&self) -> usize;
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
pub enum Function {
    OR,
    AND,
    EOR,
    AddWithCarry,
    COMPARE,
    SubtractWithBorrow
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum InternalOperations {
    NOP,
    BRK,
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
    CompareToRegister{
        src: DataRegister,
        reg2: DataRegister,
    },
    StoreToRegister {
        src: DataRegister,
        dst: DataRegister,
    },
    WriteToAddress {
        src: DataRegister,
        addr: AddressRegister,
    },
    ComputeAndStore{
        left: DataRegister, // right is implicitly InternalOperand
        dst: DataRegister,
        func: Function,
    },
    JumpToAddress,
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

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
    at_break: bool,
    overflow: bool,
    carry: bool,
    status: Data,
    operation_stream: Vec<InternalOperations>,
    instructions: HashMap<u8, Instruction>,
    total_cycles: usize,
    boot_cycles: usize,
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
        ZeroPageIndexed { reg } => vec![FetchZeroPageAddr, IncrementAddressByReg {reg: *reg}],
    }
}

pub fn create_instruction_for_mode(opcode: u8, mnemonic: &str, mode: AddressingMode, operations: &[InternalOperations]) -> (u8, Instruction) {
    (opcode, Instruction {
        mnemonic: mnemonic.to_string(),
        operations: fetch_operations_for_mode(&mode).iter().chain(operations.iter()).copied().collect(),
        addressing: mode,
    })
}

//
// An opcode is  0baaabbbcc;
// The base opcode specifies the aaa and cc.  We loop through the b which represents a different addressing mode, none.
// See
pub fn create_instructions(base_opcode: u8, mnemonic: &str, modes: &[Option<AddressingMode>], opcode_operations: &[InternalOperations]) -> Vec<(u8, Instruction)> {
    let b_mask: u8 = 0b00011100;
    let mut instructions: Vec<(u8, Instruction)> = vec!();

    for b in 0..7 {
        if let Some(mode) = modes[b] {
            let opcode = base_opcode | b_mask & ((b as u8) << 2);
            println!("{:#04x}\t{}\t{}", opcode, mnemonic, mode);
            instructions.push((opcode, Instruction {
                mnemonic: mnemonic.to_string(),
                operations: fetch_operations_for_mode(&mode).iter().chain(opcode_operations.iter()).copied().collect(),
                addressing: mode,
            }))
        }
    }
    instructions
}

fn make(f: Function) -> Vec<InternalOperations> {
    let x = vec![ComputeAndStore {
        left: A, // right is implied as InternalOperand
        dst: A,
        func: f
    }];
    x
}

pub fn create6502() -> Proc6502 {
    let mut map_o_instructions: HashMap<u8, Instruction> = HashMap::new();


    let nop = create_instruction_for_mode(0xea, "NOP", Implied, &[NOP]);
    map_o_instructions.insert(nop.0, nop.1);
    let brk = create_instruction_for_mode(0x00, "BRK", Implied, &[BRK]);
    map_o_instructions.insert(brk.0, brk.1);

    // Note we may end up creating illegal instruction opcodes but we can filter
    let fam0 = vec![
        Some(Immediate),
        Some(ZeroPage),
        None,
        Some(Absolute),
        None,
        Some(ZeroPageIndexed { reg: X }),
        None,
        Some(AbsIndexed {reg: X})
    ];

    map_o_instructions.extend(create_instructions(0x80, "STY", &fam0, &[WriteToAddress {src: Y, addr: InternalAddress}]));
    map_o_instructions.extend(create_instructions(0xa0, "LDY", &fam0, &[StoreToRegister {src: InternalOperand, dst: Y}]));
    map_o_instructions.extend(create_instructions(0xc0, "CPY", &fam0, &[CompareToRegister { src: InternalOperand, reg2: Y }]));
    map_o_instructions.extend(create_instructions(0xe0, "CPX", &fam0, &[CompareToRegister {src: InternalOperand, reg2: X}]));

    let fam1 = vec![
        Some(IndirectIndexed),
        Some(ZeroPage),
        Some(Immediate),
        Some(Absolute),
        None,
        Some(ZeroPageIndexed { reg: X }),
        None,
        Some(AbsIndexed {reg: X})
    ];

    map_o_instructions.extend(create_instructions(0x01, "ORA", &fam1, &*make(OR)));
    map_o_instructions.extend(create_instructions(0x21, "AND", &fam1, &*make(AND)));
    map_o_instructions.extend(create_instructions(0x41, "EOR", &fam1, &*make(EOR)));
    map_o_instructions.extend(create_instructions(0x61, "ADC", &fam1, &*make(AddWithCarry)));
    map_o_instructions.extend(create_instructions(0x81, "STA", &fam1, &[WriteToAddress { src: A, addr: InternalAddress }]));
    map_o_instructions.extend(create_instructions(0xA1, "LDA", &fam1, &[StoreToRegister { src: InternalOperand, dst: A }]));
    map_o_instructions.extend(create_instructions(0xC1, "CMP", &fam1, &*make(COMPARE)));
    map_o_instructions.extend(create_instructions(0xE1, "SBC", &fam1, &*make(SubtractWithBorrow)));

    let fam2_y = vec![
        Some(Immediate),
        Some(ZeroPage),
        None,
        Some(Absolute),
        None,
        Some(ZeroPageIndexed { reg: Y }),
        None,
        Some(AbsIndexed {reg: Y})
    ];

    map_o_instructions.extend(create_instructions(0xA2, "LDX", &fam2_y, &[StoreToRegister { src: InternalOperand, dst: X }]));

    let mut p = Proc6502 {
        pc: 0x0FFC,
        x: 0,
        y: 0,
        a: 0,
        internal_address: 0,
        internal_operand: 0,
        at_break: false,
        overflow: false,
        carry: false,
        status: 0,
        operation_stream: Vec::new(),
        instructions: map_o_instructions,
        total_cycles: 0,
        boot_cycles: 0,
    };

    // Prime the operation_stream with the boot sequence
    p.pc = 0x0FFC; // BOOT location
    p.operation_stream.extend(
        vec![FetchAddrLo, FetchAddrHi, JumpToAddress]
            .iter()
            .copied(),
    );
    
    p.boot_cycles = p.operation_stream.len();
    p
}

impl Proc6502 {
    fn set_reg(&mut self, reg: &DataRegister, value: Data)  {
        match reg {
            DataRegister::X => self.x = value,
            DataRegister::Y => self.y = value,
            DataRegister::A => self.a = value,
            InternalOperand => self.internal_operand = value,
        };
    }

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
    fn get_user_cycles(&self) -> usize {
        if self.total_cycles < self.boot_cycles {
            return 0;
        }
        self.total_cycles - self.boot_cycles
    }
    
    fn tick(&mut self, the_bus: Rc<RefCell<dyn Bus>>) -> (Address, bool) {
        self.total_cycles += 1;
        if self.operation_stream.is_empty() {
            // fetch the opcode
            self.operation_stream.extend([FetchOpcode].iter().copied());

            // The end of some instructions imply that a fetch of the next opcode should be done in parallel TODO
        }

        let x = self.operation_stream.remove(0);
        match x {
            NOP => {}
            BRK => {self.at_break = true}
            DummyForOverlap => {}
            FetchOpcode => {
                let opcode = the_bus.borrow().read(self.pc);
                // todo tests for illegal opcode
                if let Some(instruction) = self.instructions.get(&(opcode as u8)) {
                    println!("Excecuting {} ", instruction.mnemonic);
                    self.operation_stream
                        .extend(instruction.operations.iter().copied());
                    self.pc += 1;
                } else {
                    panic!("No definition for opcode {:#04x}", opcode);
                }
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
                self.pc += 1;
            }
            WriteToAddress { src, addr } => {
                the_bus
                    .borrow()
                    .write(self.get_addr_reg(&addr), self.get_reg(&src));
            }
            JumpToAddress => {
                self.pc = self.internal_address;
            }
            CompareToRegister{ src, reg2 } => {
                todo!();
            }
            ReadFromAccumulator => {}
            AddIndexLo => {}
            AluIncr => {}
            InternalOperations::IncrementAddressByReg { reg } => {
                self.internal_address += self.get_reg(&reg) as Address;
            }
            FetchZeroPageAddr => {
                self.internal_address &= 0x0000;
                self.internal_address = the_bus.borrow().read(self.pc) as Address;
                self.pc += 1;
            }
            IncrementPCBySignedOperand => {}
            ReadAddressLo => {}
            ReadAddressHi => {}
            StoreToRegister { src, dst } => {
                self.set_reg(&dst, self.get_reg(&src));
            }
            ComputeAndStore { left, dst, func } => {
                match func {
                    OR => todo!(),
                    AND => todo!(),
                    EOR => todo!(),
                    AddWithCarry => {
                        let (result, carry) = self.a.carrying_add(self.internal_operand, self.carry);
                        self.carry = carry;
                        // overflow is when two signed numbers with the same sign are added and the result is a different sign
                        self.overflow = (self.a ^ result) & (self.internal_operand ^ result) & 0x80 != 0;
                        self.set_reg(&dst, result)}
                    COMPARE => todo!(),
                    SubtractWithBorrow => todo!(),
                }
            }
            CompareToRegister { src, reg2 } => {
                todo!();
            }
        }
        (self.pc, self.at_break)
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
