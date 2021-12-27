use crate::Bus::{Address, BusDevice, Data};

struct memory {

}

pub trait Processor {
    fn tick(&self);

    fn reset(&self);
}

pub struct Proc6502 {

}

impl Processor for Proc6502 {


    fn tick(&self) {

    }

    fn reset(&self) {

    }

}

impl BusDevice for Proc6502 {
    fn isReadableFrom(&self, _: Address) -> bool {
        false
    }

    fn isWriteableTo(&self, _: Address) -> bool {
        false
    }


    fn doRead(&self, _: Address) -> Data {
        panic!("I can not be read from");
    }

    fn doWrite(&mut self, _: Address, _: Data) {
        panic!("I can not be written to");
    }
}
