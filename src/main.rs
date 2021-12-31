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
        upper_bound: 0x0fff,
        mem: vec![0xfe,0x0f,3,4,5,6],
    }));

    {
        // I currently don't register the processor because the bus loops through all the registered devices and
        // doesn't know how to skip the processor.  Which ends up causing a borrow runtime issue.  TODO

        //rc_bus.borrow_mut().register_device(&processor.borrow_mut().as_cloned_bus_device(Rc::clone(&processor)));
        rc_bus.borrow_mut().register_device(&Rc::clone(&memory));

    }
    // Give me a tick, Vasili. One tick only.  Just testing for now. TODO
    // Maybe three ticks for now to see if it will complete boot sequence
    processor.borrow_mut().tick(Rc::clone(&rc_bus));
    processor.borrow_mut().tick(Rc::clone(&rc_bus));
    processor.borrow_mut().tick(Rc::clone(&rc_bus));
}

