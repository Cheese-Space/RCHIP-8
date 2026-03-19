use rand::RngExt;
use crate::EmulatorError;
// todo: better error handeling
const FONT: [u8; 80] = [
0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
0x20, 0x60, 0x20, 0x20, 0x70, // 1
0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
0x90, 0x90, 0xF0, 0x10, 0x10, // 4
0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
0xF0, 0x10, 0x20, 0x40, 0x40, // 7
0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
0xF0, 0x90, 0xF0, 0x90, 0x90, // A
0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
0xF0, 0x80, 0x80, 0x80, 0xF0, // C
0xE0, 0x90, 0x90, 0x90, 0xE0, // D
0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
pub struct Chip8 {
    mem: [u8; 4096], // 4kb memory used by programs
    pub display: [[bool; 64]; 32], // 64 x 32 display eiter on (true) or off (false)
    return_stack: Vec<u16>, // stack for returning from functions
    pc: u16, // program counter
    i: u16, // register for pointing at memory
    pub delay_timer: u8, // delay timer
    pub sound_timer: u8, // sound timer
    registers: [u8; 16], // 8-bit registers
    pub keys: [bool; 16], // tracking wich keys are being held down (true) and which keys aren't (false)
	pub refresh_display: bool // if true, display will be refreshed
}
impl Chip8 {
    pub fn init(program: &[u8]) -> Result<Chip8, EmulatorError> {
        // alocate 4 kb on the stack
        let mut mem = [0u8; 4096];
        // init display
        let display = [[false; 64]; 32];
        // setup return_stack and check program len
        let return_stack: Vec<u16> = Vec::new();
        if 4096 - 512 < program.len() {
            return Err(EmulatorError::ProgramTooLarge);
        }
        // load program into ram, first 512 bytes should be free for compatibility
        for (i, j) in program.iter().enumerate() {
            mem[i+512] = *j;
        }
        // load font into ram
        for (i, j) in FONT.iter().enumerate() {
            mem[i+80] = *j;
        }
        // set pc to first instruction
        let pc = 512u16;
        // set register i
        let i = 0u16;
        // set timer
        let sound_timer = 0u8;
        let delay_timer = 0u8;
        // set registers to 0
        let registers = [0u8; 16];
		let refresh_display = false;
		let keys = [false; 16];
		// return full struct
		Ok(Chip8 { 
		    mem, 
			display, 
			return_stack, 
			pc, 
			i, 
			delay_timer, 
			sound_timer,
			registers,
			keys,
			refresh_display
		})
    }
    pub fn exec(&mut self) -> Result<(), EmulatorError> {
        // one instruction is 2 8 bit numbers
        let instruction = ((self.mem[self.pc as usize] as u16) << 8) | self.mem[self.pc as usize + 1] as u16;
        self.pc += 2; // increment pc to next instruction
        // extract "nibbles" via bitwise operations
        let first_nibble = ((instruction >> 12) & 0xF) as u8;
        let second_nibble = ((instruction >> 8) & 0xF) as u8;
        let third_nibble = ((instruction >> 4) & 0xF) as u8;
        let fourth_nibble = (instruction & 0xF) as u8;
        // decode run instructions
        match first_nibble {
            0x0 => {
                if instruction == 0xE0 {
                    // clear screen
                    for y in 0..32usize {
                        for x in 0..64usize {
                            self.display[y][x] = false;
                        }
                    }
                    self.refresh_display = true;
                }
                else if instruction == 0x00EE {
                    // return for subroutine
                    self.pc = match self.return_stack.pop() {
                        Some(addr) => addr,
                        None => return Err(EmulatorError::EmptyReturnStack)
                    }
                }
                else {
                    return Err(EmulatorError::InvalidInstruction(instruction));
                }
            }
            0x1 => {
                // jump
                self.pc = ((second_nibble as u16) << 8) | ((third_nibble as u16) << 4) | (fourth_nibble as u16);
            }
            0x2 => {
                // call subroutine
                self.return_stack.push(self.pc);
                self.pc = ((second_nibble as u16) << 8) | ((third_nibble as u16) << 4) | (fourth_nibble as u16);
            }
            0x3 => {
                // skip if VX == NN
                if self.registers[second_nibble as usize] == (third_nibble << 4) | fourth_nibble {
                    self.pc += 2;
                }
            }
            0x4 => {
                // skip if VX != NN
                if self.registers[second_nibble as usize] != (third_nibble << 4) | fourth_nibble {
                    self.pc += 2;
                }
            }
            0x5 => {
                // jump if VX == VY
                if self.registers[second_nibble as usize] == self.registers[third_nibble as usize] {
                    self.pc +=2;
                }
            }
            0x6 => {
                // set vx to NN
                self.registers[second_nibble as usize] = (third_nibble << 4) | fourth_nibble;
            }
            0x7 => {
                // add NN to vx
                self.registers[second_nibble as usize] = self.registers[second_nibble as usize].wrapping_add((third_nibble << 4) | fourth_nibble);
            }
            0x8 => {
                // mathmatical and logical functions
                match fourth_nibble {
                    0x0 => {
                        // VX = VY
                        self.registers[second_nibble as usize] = self.registers[third_nibble as usize];
                    }
                    0x1 => {
                        // VX |= VY
                        self.registers[second_nibble as usize] |= self.registers[third_nibble as usize];
                    }
                    0x2 => {
                        // VX &= VY
                        self.registers[second_nibble as usize] &= self.registers[third_nibble as usize];
                    }
                    0x3 => {
                        // VX ^= VY
                        self.registers[second_nibble as usize] ^= self.registers[third_nibble as usize];
                    }
                    0x4 => {
                        // vx += VY
                        let old_x = self.registers[second_nibble as usize];
                        self.registers[second_nibble as usize] = self.registers[second_nibble as usize].wrapping_add(self.registers[third_nibble as usize]);
                        if self.registers[second_nibble as usize] > old_x {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }
                    }
                    0x5 => {
                        // VX -= VY
                        if self.registers[second_nibble as usize] > self.registers[third_nibble as usize] {
                            self.registers[0xF] = 1;
                        }
                        else if self.registers[second_nibble as usize] < self.registers[third_nibble as usize] {
                            self.registers[0xF] = 0;
                        }
                        self.registers[second_nibble as usize] = self.registers[second_nibble as usize].wrapping_sub(self.registers[third_nibble as usize]);
                    }
                    0x6 => {
                        // VX = VY >> 1
                        self.registers[second_nibble as usize] = self.registers[third_nibble as usize] >> 1;
                        self.registers[0xF] = self.registers[second_nibble as usize] & 0x80;
                    }
                    0x7 => {
                        // VX = VY - VX
                        if self.registers[second_nibble as usize] > self.registers[third_nibble as usize] {
                            self.registers[0xF] = 0;
                        }
                        else if self.registers[second_nibble as usize] < self.registers[third_nibble as usize] {
                            self.registers[0xF] = 1;
                        }
                        self.registers[second_nibble as usize] = self.registers[third_nibble as usize].wrapping_sub(self.registers[second_nibble as usize]);
                    }
                    0xE => {
                        // VX = VY << 1
                        self.registers[second_nibble as usize] = self.registers[third_nibble as usize] << 1;
                        self.registers[0xF] = self.registers[second_nibble as usize] & 0x1;
                    }
                    _ => {
                        return Err(EmulatorError::InvalidInstruction(instruction));
                    }
                }
            }
            0x9 => {
                // jump if VX != VY
                if self.registers[second_nibble as usize] != self.registers[third_nibble as usize] {
                    self.pc +=2;
                }
            }
            0xA => {
                // set index register
                self.i = ((second_nibble as u16) << 8) | ((third_nibble as u16) << 4) | (fourth_nibble as u16);
            }
            0xB => {
                // jump with ofset NNN
                self.pc = (((second_nibble as u16) << 8) | ((third_nibble as u16) << 4) | (fourth_nibble as u16)) + self.registers[0x0] as u16;
            }
            0xC => {
                // VX = RANDOM_NUMBER & NN
                let num = (third_nibble << 4) | fourth_nibble;
                self.registers[second_nibble as usize] = rand::rng().random::<u8>() & num;
            }
            0xD => {
                // draw sprite to screen
                let x_start = self.registers[second_nibble as usize] % 64;
                let mut y = self.registers[third_nibble as usize] % 32;
                self.registers[0xF] = 0;
                self.refresh_display = true;
                for j in 0..fourth_nibble {
                    let byte = self.mem[(self.i + j as u16) as usize];
                    let mut x = x_start;
                    // loop trough each byte
                    for s in (0..=7).rev() {
                        if x == 64 {
                            break;
                        }
                        let pixel = (byte >> s) & 0x1;
                        if pixel == 1 && self.display[y as usize][x as usize] {
                            self.display[y as usize][x as usize] = false;
                            self.registers[0xF] = 1;
                        }
                        else if pixel == 1 && !self.display[y as usize][x as usize] {
                            self.display[y as usize][x as usize] = true;
                        }
                        x += 1;
                    }
                    y += 1;
                    if y == 32 {
                        break;
                    }
                }
            }
            0xE => {
                // skip if key
                match fourth_nibble {
                    // jump if key in VX is not pressed
                    0x1 => {
                        if !self.keys[self.registers[second_nibble as usize] as usize] {
                            self.pc += 2;
                        }
                    }
                    0xE => {
                        // jump if key in VX is pressed
                        if self.keys[self.registers[second_nibble as usize] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        return Err(EmulatorError::InvalidInstruction(instruction));
                    }
                }
            }
            0xF => {
                // misc. functions
                match fourth_nibble {
                    0x7 => {
                        // VX = delay timer
                        self.registers[second_nibble as usize] = self.delay_timer;
                    }
                    0x5 => {
                        match third_nibble {
                            0x1 => {
                                // delay timer = VX
                                self.delay_timer = self.registers[second_nibble as usize];
                            }
                            0x5 => {
                                // store v0 to vx in i
                                for i in 0usize..=second_nibble as usize {
                                    self.mem[self.i as usize] = self.registers[i];
                                    self.i += 1;
                                }
                            }
                            0x6 => {
                                // load mem into registers
                                for i in 0usize..=second_nibble as usize {
                                    self.registers[i] = self.mem[self.i as usize];
                                    self.i += 1;
                                }
                            }
                            _ => {
                                return Err(EmulatorError::InvalidInstruction(instruction));
                            }
                        }
                    }
                    0x8 => {
                        // set sound timer to VX
                        self.sound_timer = self.registers[second_nibble as usize];
                    }
                    0xE => {
                        // I += VX
                        self.i += self.registers[second_nibble as usize] as u16;
                    }
                    0xA => {
                        // get a key
                        self.pc -= 2;
                        for (i, j) in self.keys.iter().enumerate() {
                            if *j {
                                self.registers[second_nibble as usize] = i as u8;
                                self.pc += 2;
                            }
                        }
                    }
                    0x3 => {
                        let num_as_str = self.registers[second_nibble as usize].to_string();
                        let mut res = String::new();
                        if num_as_str.len() < 3 {
                            for _ in 0..3-num_as_str.len() {
                                res.push('0');
                            }
                        }
                        res.push_str(&num_as_str);
                        for (i, j) in res.as_bytes().iter().enumerate() {
                            self.mem[self.i as usize + i] = *j-48; 
                        }
                    }
                    0x9 => {
                        self.i = self.registers[second_nibble as usize] as u16 * 5 + 80;
                    }
                    _ => {
                        return Err(EmulatorError::InvalidInstruction(instruction));
                    }
                }
            }
            _ => {
                return Err(EmulatorError::InvalidInstruction(instruction));
            }
        }
        Ok(())
    }
}