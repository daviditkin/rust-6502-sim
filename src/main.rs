mod bus;
mod processor;
mod memory;

use std::cell::RefCell;
use std::rc::Rc;
use crate::bus::{Bus, BusDevice, SimpleBus};
use crate::memory::Memory;
use crate::processor::{create6502, ProcessorTrait};


fn main() {
    let rc_bus: Rc<RefCell<dyn Bus>> = Rc::new(RefCell::new(SimpleBus {
        registered: vec![],
    }));

    let processor = create6502(Rc::clone(&rc_bus));
    let memory:Rc<RefCell<dyn BusDevice>> = Rc::new(RefCell::new(Memory {
        lower_bound: 0x0ffc,
        upper_bound: 0xffff,
        mem: vec![0xfe,0x0f, 0xea,0x4c,0xfe,0x0f],
    }));

    {
        rc_bus.borrow_mut().register_device(&Rc::clone(&memory));
        //rc_bus.borrow_mut().register_device(&processor.borrow_mut().as_cloned_bus_device(Rc::clone(&processor)));
    }
    loop {
        processor.borrow_mut().tick(Rc::clone(&rc_bus));
    }
}

