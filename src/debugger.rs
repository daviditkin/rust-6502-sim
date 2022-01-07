use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::bus::Bus;
use crate::processor::ProcessorTrait;

// TODO thinking about how to add a debugger to the system
pub struct Debugger {
    processor: Weak<RefCell<dyn ProcessorTrait>>,
}

enum Commands {
    DumpMemoryRange,
    STEP,
}
impl Debugger {
    pub fn register_processor(&self) {

    }

    pub fn run(&mut self, bus: Rc<RefCell<dyn Bus>>, proc: &Rc<RefCell<dyn ProcessorTrait>>) {
        let mut x = self.processor.upgrade().unwrap();
        x.borrow_mut().tick(bus);


    }
}