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
    stack: Vec<u16>, // stack for returning from functions
    pc: u16, // program counter
    i: u16, // register for pointing at memory
    pub delay_timer: u8, // delay timer
    pub sound_timer: u8, // sound timer
    registers: [u8; 16], // 8-bit registers
	pub refresh_display: bool
}
impl Chip8 {
    pub fn init(program: &[u8]) -> Chip8 {
        // alocate 4 kb on the stack
        let mut mem = [0u8; 4096];
        // init display
        let display = [[false; 64]; 32];
        // setup stack and check program len
        let stack: Vec<u16> = Vec::new();
        if 4096 - 512 < program.len() {
            eprintln!("error: program is too large!");
            std::process::exit(1);
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
        let registers = [0u8; 16];
        // set registers to 0
		let refresh_display = false;
		// return full struct
		Chip8 { 
		    mem, 
			display, 
			stack, 
			pc, 
			i, 
			delay_timer, 
			sound_timer,
			registers,
			refresh_display
		}
    }
    pub fn exec(&mut self) {
        // one instruction is 2 8 bit numbers
        let instruction = ((self.mem[self.pc as usize] as u16) << 8) | self.mem[self.pc as usize + 1] as u16;
        self.pc += 2; // increment pc to next instruction
        // extract "nibbles" via bitwise operations
        let first_nibble = ((instruction >> 12) & 0xF) as u8;
        let second_nibble = ((instruction >> 8) & 0xF) as u8;
        let third_nibble = ((instruction >> 4) & 0xF) as u8;
        let fourth_nibble = (instruction & 0xF) as u8;
        // run instructions
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
                    return;
                }
                else {
                    // return for subroutine
                    self.pc = self.stack.pop().unwrap();
                    return;
                }
            }
            0x1 => {
                // jump
                self.pc = ((second_nibble as u16) << 8) | ((third_nibble as u16) << 4) | (fourth_nibble as u16);
                return;
            }
            0x2 => {
                // call subroutine
                self.stack.push(self.pc);
                self.pc = ((second_nibble as u16) << 8) | ((third_nibble as u16) << 4) | (fourth_nibble as u16);
                return;
            }
            0x3 => {
                // skip if VX == NN
                if self.registers[second_nibble as usize] == (third_nibble << 4) | fourth_nibble {
                    self.pc += 2;
                }
                return;
            }
            0x4 => {
                // skip if VX != NN
                if self.registers[second_nibble as usize] != (third_nibble << 4) | fourth_nibble {
                    self.pc += 2;
                }
                return;
            }
            0x5 => {
                // jump if VX == VY
                if self.registers[second_nibble as usize] == self.registers[third_nibble as usize] {
                    self.pc +=2;
                }
                return;
            }
            0x6 => {
                // set vx to NN
                self.registers[second_nibble as usize] = (third_nibble << 4) | fourth_nibble;
                return;
            }
            0x7 => {
                // add NN to vx
                self.registers[second_nibble as usize] += (third_nibble << 4) | fourth_nibble;
                return;
            }
            0x9 => {
                // jump if VX != VY
                if self.registers[second_nibble as usize] != self.registers[third_nibble as usize] {
                    self.pc +=2;
                }
                return;
            }
            0xA => {
                // set index register
                self.i = ((second_nibble as u16) << 8) | ((third_nibble as u16) << 4) | (fourth_nibble as u16);
                return;
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
                return;
            }
            _ => {
                eprintln!("unrecognised or unimplemented instruction: 0x{:x}{:x}{:x}{:x}", first_nibble, second_nibble, third_nibble, fourth_nibble);
                std::process::exit(1);
            }
        }
    }
}