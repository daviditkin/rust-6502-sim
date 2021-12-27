pub type Address = usize;

pub type Data = u8;

pub trait BusDevice {
    fn isReadableFrom(&self, address: Address) -> bool;

    fn isWriteableTo(&self, address: Address) -> bool;

    fn doRead(&self, address: Address) -> Data;

    fn doWrite(&mut self, address: Address, data: Data);
}

pub trait Bus {
    fn registerBusDevice(&mut self, device: Box<dyn BusDevice>);

    fn read(&self, address: Address) -> Data;

    fn write(&mut self, address: Address, data: Data);
}

pub struct SimpleBus {
    pub devices:  Vec<Box<dyn BusDevice>>,
}

impl Bus for SimpleBus {
    fn registerBusDevice(&mut self, device: Box<dyn BusDevice>) {
        let mut foo:  Vec<Box<dyn BusDevice>> = Vec::new();
        self.devices.push(device);
    }

    fn read(&self, addr: Address) -> Data {
        for db in &self.devices {
            if (db.isReadableFrom(addr)) {
                return db.doRead(addr)
            }
        }
        0

    }

    fn write(&mut self, addr: Address, data: Data) {
        for db in &mut self.devices {
            if (db.isReadableFrom(addr)) {
                db.doWrite(addr, data)
            }
        }
    }
}
