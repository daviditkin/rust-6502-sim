use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type Address = u16;

pub type Data = u8;


// a device on bus that handles read / write / isReadable... callbacks
pub trait BusDevice {
    fn doRead(&self, address: Address) -> Data;
    fn doWrite(&mut self, address: Address, data: Data);
    fn isReadableFor(&self, address: Address) -> bool;
    fn isWritableFor(&self, address: Address) -> bool;
}

// holds devices
pub trait Bus {
    fn write(&self, address: Address, data: Data);
    fn read(&self, address: Address) -> Data;
    fn registerDevice(&mut self, device: &Rc<RefCell<dyn BusDevice>>);
}

pub struct SimpleBus {
    pub registered: Vec<Rc<RefCell<dyn BusDevice>>>,
}

// A producer consumer that has a reference???? to the bus
pub struct SimpleBusDevice {
    bus: Weak<RefCell<dyn Bus>>,
    data: Vec<Data>,
    lowerBound: Address,
    upperBound: Address,
}

impl BusDevice for SimpleBusDevice {
    fn doRead(&self, address: Address) -> Data {
        0x0
    }

    fn doWrite(&mut self, address: Address, data: Data) {
        println!("doing a write of {} to {}", data, address);
        self.bus.upgrade().unwrap().borrow().write(address, data);
    }

    fn isReadableFor(&self, address: Address) -> bool {
        true
    }

    fn isWritableFor(&self, address: Address) -> bool {
        true
    }
}

impl Bus for SimpleBus {
    fn write(&self, address: Address, data: Data) {
        for d in self.registered.iter() {
            if d.borrow().isWritableFor(address) {
                d.borrow_mut().doWrite(address, data);
            }
        }
    }

    fn read(&self, address: Address) -> Data {
        for d in self.registered.iter() {
            if d.borrow().isReadableFor(address) {
                return d.borrow_mut().doRead(address)
            }
        }
        0x0
    }

    fn registerDevice(&mut self, device: &Rc<RefCell<dyn BusDevice>>) {
        self.registered.push(Rc::clone(device));
    }
}

pub fn make_device(bus: Rc<RefCell<dyn Bus>>) -> Rc<RefCell<dyn BusDevice>> {
    Rc::new(RefCell::new(SimpleBusDevice {
        bus: Rc::downgrade(&bus),
        data: vec![],
        lowerBound: 0,
        upperBound: 100,
    }))

}

