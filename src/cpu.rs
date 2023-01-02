use std::fmt::{Formatter, Display};


const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

struct MemoryBus {
    memory: [u8; 0xFFFF + 1],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; 0xFFFF + 1],
        }
    }

    pub fn set_range(&mut self, start: usize, len: usize, values: &[u8]) {
       self.memory[start..(start + len) as usize].copy_from_slice(values);
    }
}

#[derive(Debug)]
struct FlagsRegister {
    pub zero: bool,
    pub subtraction: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl std::convert::From<FlagsRegister> for u8  {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero       { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION |
        (if flag.subtraction   { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION |
        (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION |
        (if flag.carry      { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtraction = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtraction,
            half_carry,
            carry
        }
    }
}

struct Registers {
    registers: [u8; 8],
}

impl Display for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str("Registers:");
        output.push_str(&format!(" AF: {:04X}\n", self.get_register_word("af".codes())));
        output.push_str(&format!(" BC: {:04X}\n", self.get_register_word("bc".codes())));
        output.push_str(&format!(" DE: {:04X}\n", self.get_register_word("de".codes())));
        output.push_str(&format!(" HL: {:04X}\n", self.get_register_word("hl".codes())));
        output.push_str(&format!(" Flags: {:?}\n", self.get_flags()));
        write!(f, "{}", output)
    }
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            registers: [0; 8],
        }
    }

    pub fn get_register_word(&self, codes: (usize, usize)) -> u16 {
        self.registers[codes.0] as u16 + ((self.registers[codes.1] as u16) << 8)
    }

    pub fn set_register_word(&mut self, codes: (usize, usize), value: u16) {
        self.registers[codes.0] = value as u8;
        self.registers[codes.1] = (value >> 8) as u8;
    }

    pub fn get_register(&self, register: usize) -> u8 {
        self.registers[register]
    }

    pub fn set_register(&mut self, register: usize, value: u8) {
        self.registers[register] = value;
    }

    pub fn get_flags(&self) -> FlagsRegister {
        FlagsRegister::from(self.registers[0xF])
    }

    pub fn set_flags(&mut self, zero: Option<bool>, subtraction: Option<bool>, half_carry: Option<bool>, carry: Option<bool>) {
        let c_flag = self.get_flags();
        let flags = FlagsRegister {
            zero: zero.unwrap_or(c_flag.zero),
            subtraction: subtraction.unwrap_or(c_flag.subtraction),
            half_carry: half_carry.unwrap_or(c_flag.half_carry),
            carry: carry.unwrap_or(c_flag.carry),
        };
        self.registers[0xF] = u8::from(flags);
    }
}

pub trait ToRegisterCode {
    fn codes(&self) -> (usize, usize);
    fn code(&self) -> usize {
        Self::codes(self).0
    }
}

impl ToRegisterCode for &str {
    fn codes(&self) -> (usize, usize) {
        match self.to_lowercase().as_str() {
            "af" => (0, 1),
            "bc" => (2, 3),
            "de" => (4, 5),
            "hl" => (6, 7),
            _ => panic!("Unknown register code"),
        }
    }
}

pub struct Cpu {
    registers: Registers,
    sp: u16,
    pc: u16,
    ram: MemoryBus,
    dump_registers_after: Option<u8>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            sp: 0,
            pc: 0,
            ram: MemoryBus::new(),
            dump_registers_after: None,
        }
    }

    pub fn load_rom(&mut self, rom_in: Vec<u8>) {
        let mut rom = [0; 0x3FFF + 1];
        rom.copy_from_slice(&rom_in);
        self.ram.set_range(0x0000, 0x3FFF + 1, &rom);
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.ram.memory[self.pc as usize];
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let byte = self.fetch_byte();
        let word = byte as u16 + ((self.ram.memory[self.pc as usize + 1] as u16) << 8);
        word
    }

    fn inc_reg_byte(&mut self, register: usize) {
        let value = self.registers.get_register(register);
        let (result, carry) = value.overflowing_add(1);
        self.registers.set_register(register, result);
        self.registers.set_flags(Some(result == 0), Some(false), Some(carry), None);
    }

    fn dec_reg_byte(&mut self, register: usize) {
        let value = self.registers.get_register(register);
        let (result, carry) = value.overflowing_sub(1);
        self.registers.set_register(register, result);
        self.registers.set_flags(Some(result == 0), Some(true), Some(carry), None);
    }

    pub fn execute(&mut self) {
        let opcode = self.ram.memory[self.pc as usize];
        let increment = match opcode & 0xFF {
            0x00 => 1,
            0x01 => { let nn = self.fetch_word(); self.registers.set_register_word("bc".codes(), nn); 3 },
            0x02 => { let v = self.registers.get_register_word("bc".codes()); self.ram.memory[v as usize] = self.registers.get_register("a".code()); 1 },
            0x03 => { let v = self.registers.get_register_word("bc".codes()); self.registers.set_register_word("bc".codes(), v.wrapping_add(1)); 1 },
            0x04 => { self.inc_reg_byte("b".code()); 1},
            0x05 => { self.dec_reg_byte("b".code()); 1},
            0x06 => { let v = self.fetch_byte(); self.registers.set_register("b".code(), v); 2 },
            0x07 => todo!("RLCA"),
            0x08 => { let nn = self.fetch_word(); self.sp = (nn & 0xFF) as u16 + (nn as u16) << 8;  3},
            _=> {
                println!("Unknown opcode: {:X}", opcode);
                panic!("Unknown opcode");
            }
        };

        self.sp = self.sp.wrapping_add(increment);
        if self.dump_registers_after.is_some() {
            if self.dump_registers_after.unwrap() == opcode {
                
                println!("Registers: {}", self.registers.to_string());
                println!("SP: {:X}", self.sp);
                println!("PC: {:X}", self.pc);
                println!("Opcode: {:X}", opcode);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags_register() {
        let flags = FlagsRegister {
            zero: true,
            subtraction: true,
            half_carry: true,
            carry: true,
        };

        let byte = u8::from(flags);

        assert_eq!(byte, 0b1111_0000);

        let flags = FlagsRegister::from(byte);

        assert_eq!(flags.zero, true);
        assert_eq!(flags.subtraction, true);
        assert_eq!(flags.half_carry, true);
        assert_eq!(flags.carry, true);


        let flags = FlagsRegister {
            zero: false,
            subtraction: false,
            half_carry: false,
            carry: false,
        };

        let byte = u8::from(flags);

        assert_eq!(byte, 0b0000_0000);
    }
}