use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type Address = u16;

pub type Data = u8;
#[derive()]

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

// A producer consumer that has a reference???? to the bus
pub struct SimpleBusDevice {
    bus: Weak<RefCell<dyn Bus>>,
    data: Vec<Data>,
    lower_bound: Address,
    upper_bound: Address,
}

impl BusDevice for SimpleBusDevice {
    fn do_read(&self, _address: Address) -> Data {
        0x0
    }

    fn do_write(&mut self, address: Address, data: Data) {
        println!("doing a write of {} to {}", data, address);
        self.bus.upgrade().unwrap().borrow().write(address, data);
    }

    fn is_readable_for(&self, _address: Address) -> bool {
        true
    }

    fn is_writable_for(&self, _address: Address) -> bool {
        true
    }
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
        for d in self.registered.iter() {
            if d.borrow().is_readable_for(address) {
                return d.borrow_mut().do_read(address)
            }
        }
        0x0
    }

    fn register_device(&mut self, device: &Rc<RefCell<dyn BusDevice>>) {
        self.registered.push(Rc::clone(device));
    }
}

pub fn make_device(bus: Rc<RefCell<dyn Bus>>) -> Rc<RefCell<dyn BusDevice>> {
    Rc::new(RefCell::new(SimpleBusDevice {
        bus: Rc::downgrade(&bus),
        data: vec![],
        lower_bound: 0,
        upper_bound: 100,
    }))

}

