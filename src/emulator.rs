use crate::cpu::Cpu;


pub struct Emulator {
    cpu: Cpu,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            cpu: Cpu::new(),
        }
    }

    pub fn load_rom(&mut self, rom: &str) {
        let rom = std::fs::read(rom).unwrap();
        self.cpu.load_rom(rom);
    }
}