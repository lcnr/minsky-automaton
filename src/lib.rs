#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Halt,
    Increment(u8, u16),
    Decrement(u8, u16, u16),
    Purged,
}


pub struct Program {
    registers: [u64; 1 << 8],
    instructions: [Instruction; 1 << 16],
    ptr: u16,
}

impl Program {
    pub fn empty() -> Program {
        Program {
            registers: [0; 1 << 8],
            instructions: [Instruction::Halt; 1 << 16],
            ptr: 0,
        }
    }

    pub fn new(instructions: impl IntoIterator<Item = Instruction>) -> Program {
        let mut program = Program::empty();
        for (tgt, src) in program.instructions.iter_mut().zip(instructions) {
            *tgt = src;
        }

        program
    }

    pub fn set_register(&mut self, reg: u8, value: u64) {
        self.registers[reg as usize] = value;
    }

    pub fn get_register(&self, reg: u8) -> u64 {
        self.registers[reg as usize]
    }

    pub fn step(&mut self) {
        match self.instructions[self.ptr as usize] {
            Instruction::Halt => {}
            Instruction::Increment(reg, target) => {
                self.registers[reg as usize] += 1;
                self.ptr = target;
            }
            Instruction::Decrement(reg, then, els) => {
                if let Some(v) = self.registers[reg as usize].checked_sub(1) {
                    self.registers[reg as usize] = v;
                    self.ptr = then;
                } else {
                    self.ptr = els;
                }
            }
            Instruction::Purged => unreachable!("Reached purged instruction"),
        }
    }

    pub fn run(&mut self, max_steps: u64) -> u64 {
        for step in 0..max_steps {
            if self.instructions[self.ptr as usize] == Instruction::Halt {
                return step;
            }

            self.step();
        }

        max_steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halt() {
        let mut prog = Program::empty();
        assert_eq!(prog.run(u64::MAX), 0);
    }

    #[test]
    fn substract() {
        // $0 = $0 - $1
        let mut prog = Program::new(
            [
                // Move $1 to a placeholder location.
                Instruction::Decrement(1, 1, 2), // 0
                Instruction::Increment(2, 0),    // 1
                // Move placeholder back to $1 while decreasing $0
                Instruction::Decrement(2, 3, 5), // 2
                Instruction::Increment(1, 4),    // 3
                Instruction::Decrement(0, 2, 2), // 4
                Instruction::Halt,               // 5
            ]
            .iter()
            .copied(),
        );

        prog.set_register(0, 98);
        prog.set_register(1, 81);

        assert_eq!(prog.run(u64::MAX), 407);
        assert_eq!(prog.get_register(0), 17);
        assert_eq!(prog.get_register(1), 81);
    }

    #[test]
    fn add() {
        // $0 = $0 + $1
        let mut prog = Program::new(
            [
                // Move $1 to a placeholder location.
                Instruction::Decrement(1, 1, 2), // 0
                Instruction::Increment(2, 0),    // 1
                // Move placeholder back to $1 while also increasing $0
                Instruction::Decrement(2, 3, 5), // 2
                Instruction::Increment(1, 4),    // 3
                Instruction::Increment(0, 2),    // 4
                Instruction::Halt,               // 5
            ]
            .iter()
            .copied(),
        );

        prog.set_register(0, 98);
        prog.set_register(1, 81);

        assert_eq!(prog.run(u64::MAX), 407);
        assert_eq!(prog.get_register(0), 179);
        assert_eq!(prog.get_register(1), 81);
    }

    #[test]
    fn multiply() {
        // $0 = $0 * $1
        let mut prog = Program::new(
            [
                // Move $1 to a placeholder location ($2).
                Instruction::Decrement(1, 1, 2), // 0
                Instruction::Increment(2, 0),    // 1
                // Move $0 to a placeholder location ($3)
                Instruction::Decrement(0, 3, 4), // 2
                Instruction::Increment(3, 2),    // 3
                // Move $2 back into $1
                // At each step, take turns either moving $3 to $0 and $4
                // or moving $4 to $0 and $3.
                Instruction::Decrement(2, 5, 14), // 4
                Instruction::Increment(1, 6),     // 5
                // Move $3 into $0 and $4
                Instruction::Decrement(3, 7, 9), // 6
                Instruction::Increment(0, 8),    // 7
                Instruction::Increment(4, 6),    // 8
                // Decrement $2
                Instruction::Decrement(2, 10, 14), // 9
                Instruction::Increment(1, 11),     // 10
                // Move $4 into $0 and $3
                Instruction::Decrement(4, 12, 4), // 11
                Instruction::Increment(0, 13),    // 12
                Instruction::Increment(3, 11),    // 13
                Instruction::Halt,                // 14
            ]
            .iter()
            .copied(),
        );

        prog.set_register(0, 98);
        prog.set_register(1, 81);
        assert_eq!(prog.run(100000), 24418);
        assert_eq!(prog.get_register(0), 7938);
        assert_eq!(prog.get_register(1), 81);
    }
}
