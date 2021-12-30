mod Bus;
mod Processor;
mod Memory;

use std::cell::RefCell;
use std::rc::Rc;
use crate::Bus::{Bus as TBUS, BusDevice, make_device, SimpleBus, SimpleBusDevice};
use crate::Memory::memory;
use crate::Processor::{create6502, ProcessorTrait};

fn main() {
    let mut rcBus: Rc<RefCell<dyn TBUS>> = Rc::new(RefCell::new(SimpleBus {
        registered: vec![],
    }));

    let mut processor = create6502(Rc::clone(&rcBus));


    let device1 = make_device(Rc::clone(&rcBus));

    {
        rcBus.borrow_mut().registerDevice(&processor);
    }

    println!("Hello, world!");
}

