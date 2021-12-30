use crate::bus::{Address, BusDevice, Data};

pub struct Memory {
    pub lower_bound: Address,
    pub upper_bound: Address,
    pub mem: Vec<Data>,
}

impl BusDevice for Memory {
    fn do_read(&self, address: Address) -> Data {
        self.mem[address as usize]
    }

    fn do_write(&mut self, address: Address, data: Data) {
        self.mem[address as usize] = data;
    }

    fn is_readable_for(&self, address: Address) -> bool {
        address >= self.lower_bound && address <= self.upper_bound
    }

    fn is_writable_for(&self, address: Address) -> bool {
        address >= self.lower_bound && address <= self.upper_bound
    }
}

