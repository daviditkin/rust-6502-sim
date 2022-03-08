use std::cell::RefCell;
use std::rc::Rc;

use rust_6502_emulator::bus::{Address, Bus, BusDevice, Data, SimpleBus};
use rust_6502_emulator::memory::Memory;
use rust_6502_emulator::processor::{create6502, ProcessorTrait};

pub fn char_to_hex_byte(c: char) -> u8 {
    if ('0'..='9').contains(&c) {
        (c as u8) - b'0'
    } else if ('a'..='f').contains(&c){
        (c as u8) - b'a' + 0x0a
    } else {
        panic!("{} not in hex byte range", c);
    }
}
pub fn str_to_byte(s: &str) -> Data {
    let x: Vec<u8> = s.to_ascii_lowercase().chars().map(char_to_hex_byte).collect();
    let high = (x[0] << 4) as u8;
    let low = x[1];
    high | low
}

fn write_program_to_memory(mem: &Rc<RefCell<Memory>>, start: Address, obj_code: String) {
    let x:Vec<Data> = obj_code.lines()
        .filter(|s| !s.is_empty())
        .flat_map(|s| s.split_whitespace())
        .filter(|s| !s.ends_with(':'))
        .map(|y| str_to_byte(y))
        .collect();

    mem.borrow_mut().write(start, x);
}

fn make_eprom_for_program(object_code_hex_dump: &str, start: Address) -> Rc<RefCell<Memory>> {
    let memory: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new(0x000, 0xffff)));
    // write the boot vector

    let start_low = (start & 0x00ff) as u8;
    let start_high = ((start & 0xff00) >> 8) as u8;
    memory
        .borrow_mut()
        .write(0xffc, vec![start_low, start_high]); // , 0xea, 0x4c, 0xfe, 0x0f, 0xfe, 0x0f]);
    // write the program
    write_program_to_memory(&memory, start, object_code_hex_dump.to_string());
    memory
}

struct TestCase {
    hex_dump: &'static str,
    test_loc: Address,
    expected: Data,
    expected_cycles: usize,
}

fn test_the_case(test_case: TestCase) {
    let bus: Rc<RefCell<dyn Bus>> = Rc::new(RefCell::new(SimpleBus { registered: vec![] }));

    let processor = Rc::new(RefCell::new(create6502()));
    let memory = make_eprom_for_program(test_case.hex_dump, 0x0200);
    bus.borrow_mut().register_device(&memory.borrow_mut().as_cloned_bus_device(Rc::clone(&memory)));

    loop {
        let (_,at_break) = processor.borrow_mut().tick(Rc::clone(&bus));
        if at_break {
            break;
        }
    }
    assert_eq!(memory.borrow().do_read(test_case.test_loc), test_case.expected);
    assert_eq!(processor.borrow().get_user_cycles(), test_case.expected_cycles);

}

// Test zero page x indexed
//    nop
//    ldx #0x05
//    lda #0xaa
//    sta 0x01,X
//    nop
//    brk
//    nop
//    nop
const STA_ZP_X_TEST: TestCase = TestCase {
    hex_dump: "
        0200: EA A2 05 A9 AA 95 01 EA
        0208: 00 EA EA",
    test_loc: 0x0006,
    expected: 0xaa,
    expected_cycles: 16,  // TODO this value is incorrect
};

const NOP_CYCLE_TEST: TestCase = TestCase {
    hex_dump: "0200: EA 00",
    test_loc: 0x0200,
    expected: 0xEA,
    expected_cycles: 4,  // TODO this value is incorrect
};

#[test]
fn test_addressing_modes() {
    test_the_case(STA_ZP_X_TEST);
    test_the_case(NOP_CYCLE_TEST);
}

