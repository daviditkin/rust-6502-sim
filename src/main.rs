mod Bus;
mod Processor;
mod Memory;

use crate::Bus::{Bus as TBUS, SimpleBus};
use crate::Memory::memory;
use crate::Processor::Proc6502;

fn main() {

    let mut b =  SimpleBus {
        devices: Vec::new(),
    };

    let mut p = Proc6502 {

    };

    let mut mem = memory {
        lowerBound: 0x200,
        upperBound: 0x1FFF,
        mem: vec!(0; 0x1FFF),
    };

    b.registerBusDevice(Box::new(p));
    b.registerBusDevice(Box::new(mem));
    b.write(0x201, 0xcc);
    assert_eq!(b.read(0x201), 0xcc);
    println!("Hello, world!");
}
