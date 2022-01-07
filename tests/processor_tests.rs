
use std::cell::RefCell;
use std::rc::Rc;

use rust_6502_emulator::bus::{Bus, SimpleBus};
use rust_6502_emulator::memory::Memory;
use rust_6502_emulator::processor::{create6502, ProcessorTrait};


#[test]
fn test_simple() {
    let bus: Rc<RefCell<dyn Bus>> = Rc::new(RefCell::new(SimpleBus { registered: vec![] }));

    let processor = Rc::new(RefCell::new(create6502()));
    let memory: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new(0x0f00, 0xffff)));

    memory
        .borrow_mut()
        .write(0xffc, vec![0xfe, 0x0f, 0xea, 0x4c, 0xfe, 0x0f]);
    bus.borrow_mut()
        .register_device(&memory.borrow_mut().as_cloned_bus_device(Rc::clone(&memory)));

    processor.borrow_mut().tick(bus.clone());

    memory.borrow().dump_memory(0x0ffc, 0x0fff);

}
