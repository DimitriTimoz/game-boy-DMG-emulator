
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

}

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

impl Registers {
    pub fn new() -> Registers {
        Registers {
            registers: [0; 8],
        }
    }

    pub fn get_register_word(&self, a: usize, b: usize) -> u16 {
        self.registers[a] as u16 + ((self.registers[b] as u16) << 8)
    }

    pub fn set_register_word(&mut self, a: usize, b: usize, value: u16) {
        self.registers[a] = value as u8;
        self.registers[b] = (value >> 8) as u8;
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

    pub fn set_flags(&mut self, flags: FlagsRegister) {
        self.registers[0xF] = u8::from(flags);
    }
}


pub struct Cpu {
    registers: Registers,
    sp: u16,
    pc: u16,
    ram: MemoryBus,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            sp: 0,
            pc: 0,
            ram: MemoryBus::new(),
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