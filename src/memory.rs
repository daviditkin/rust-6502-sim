use crate::bus::{Address, BusDevice, Data};

use core::hash::{BuildHasherDefault, Hasher};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityHasher(Address);

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("Only implemented for u16 (Address)");
    }

    fn write_u16(&mut self, i: u16) {
        self.0 = i;
    }
}

type BuildIdentityHasher = BuildHasherDefault<IdentityHasher>;

pub struct Memory {
    pub lower_bound: Address,
    pub upper_bound: Address,
    pub mem: HashMap<Address, Data, BuildIdentityHasher>,
}

impl Memory {
    pub fn write(&mut self, start: Address, data: Vec<Data>) {
        let mut addr = start;
        data.iter().for_each(|d| {
            self.do_write(addr, *d);
            addr += 1;
        })
    }

    pub fn as_cloned_bus_device(&self, me: Rc<RefCell<Memory>>) -> Rc<RefCell<dyn BusDevice>> {
        let rc: Rc<RefCell<dyn BusDevice>> = me;
        Rc::clone(&rc)
    }

    pub fn new(start: Address, end: Address) -> Memory {
        Memory {
            lower_bound: start,
            upper_bound: end,
            mem: Default::default(),
        }
    }
}

impl BusDevice for Memory {
    fn do_read(&self, address: Address) -> Data {
        let x = self.mem.get(&(address - self.lower_bound)).unwrap_or(&0);
        *x
    }

    fn do_write(&mut self, address: Address, data: Data) {
        self.mem.insert(address - self.lower_bound, data);
    }

    fn is_readable_for(&self, address: Address) -> bool {
        address >= self.lower_bound && address <= self.upper_bound
    }

    fn is_writable_for(&self, address: Address) -> bool {
        address >= self.lower_bound && address <= self.upper_bound
    }
}
