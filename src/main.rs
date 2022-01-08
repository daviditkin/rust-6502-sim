use std::cell::RefCell;
use std::rc::Rc;

use rust_6502_emulator::bus::{Address, Bus, SimpleBus};
use rust_6502_emulator::memory::Memory;
use rust_6502_emulator::processor::{create6502, ProcessorTrait};

fn main() {
    let bus: Rc<RefCell<dyn Bus>> = Rc::new(RefCell::new(SimpleBus { registered: vec![] }));

    let processor = Rc::new(RefCell::new(create6502()));
    let memory: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new(0x000, 0xffff)));

    // write the boot vector
    memory
        .borrow_mut()
        .write(0xffc, vec![0x00, 0x02]); // , 0xea, 0x4c, 0xfe, 0x0f, 0xfe, 0x0f]);
    // write a program starting at boot vector
    memory
        .borrow_mut()
        .write(0x0200, vec![
            0xea, // NOP
            0xa2, // LDX #
            0x05,
            0xa9, // LDA #
            0xaa,
            0x95, // STA zp,X
            0x01,
            0xea,
        ]);
    bus.borrow_mut()
        .register_device(&memory.borrow_mut().as_cloned_bus_device(Rc::clone(&memory)));
        //   bus.borrow_mut().register_device(&processor.borrow().as_cloned_bus_device(Rc::clone(&processor)));

    let break_address: Address = 0x0208;
    loop {
        let address = processor.borrow_mut().tick(Rc::clone(&bus));
        if address.1 {
            break
        }
    }

    memory.borrow().dump_memory(0x0000, 0x0010);
}
