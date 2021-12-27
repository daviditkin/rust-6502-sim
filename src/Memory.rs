use crate::Bus::{Address, BusDevice, Data};

pub struct memory {
    pub lowerBound: Address,
    pub upperBound: Address,
    pub mem: Vec<Data>,
}

impl BusDevice for memory {
    fn isReadableFrom(&self, address: Address) -> bool {
        address >= self.lowerBound && address <= self.upperBound
    }

    fn isWriteableTo(&self, address: Address) -> bool {
        address >= self.lowerBound && address <= self.upperBound
    }

    fn doRead(&self, address: Address) -> Data {
        self.mem[address]
    }

    fn doWrite(&mut self, address: Address, data: Data) {
        self.mem[address] = data;
    }
}

