use crate::cpu::Cpu;


pub struct Emulator {
    ram: [u8; 0x1FFFF],
    cpu: Cpu,
}