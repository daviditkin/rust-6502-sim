use std::cell::RefCell;
use std::rc::{Rc};

pub type Address = u16;

pub type Data = u8;

// a device on bus that handles read / write / isReadable... callbacks
pub trait BusDevice {
    fn do_read(&self, address: Address) -> Data;
    fn do_write(&mut self, address: Address, data: Data);
    fn is_readable_for(&self, address: Address) -> bool;
    fn is_writable_for(&self, address: Address) -> bool;
}

// holds devices
pub trait Bus {
    fn write(&self, address: Address, data: Data);
    fn read(&self, address: Address) -> Data;
    fn register_device(&mut self, device: &Rc<RefCell<dyn BusDevice>>);
}

pub struct SimpleBus {
    pub registered: Vec<Rc<RefCell<dyn BusDevice>>>,
}

impl Bus for SimpleBus {
    fn write(&self, address: Address, data: Data) {
        for d in self.registered.iter() {
            if d.borrow().is_writable_for(address) {
                d.borrow_mut().do_write(address, data);
            }
        }
    }

    fn read(&self, address: Address) -> Data {
        for d in &self.registered {
            if !(d.borrow().is_readable_for(address)) {
                continue;
            } else {
                return d.borrow_mut().do_read(address);
            }
        }
        0x0
    }

    fn register_device(&mut self, device: &Rc<RefCell<dyn BusDevice>>) {
        self.registered.push(Rc::clone(device));
    }
}

