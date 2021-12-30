mod bus;
mod processor;
mod memory;

use std::cell::RefCell;
use std::rc::Rc;
use crate::bus::{Bus, make_device, SimpleBus};
use crate::processor::create6502;


fn main() {
    let mut rc_bus: Rc<RefCell<dyn Bus>> = Rc::new(RefCell::new(SimpleBus {
        registered: vec![],
    }));

    let mut processor = create6502(Rc::clone(&rc_bus));
    rc_bus.borrow_mut().register_device(&processor);

//    processor.borrow_mut().tick();
}

