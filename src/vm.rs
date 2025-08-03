use crate::instructions::opcodes::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct VM {
    memory: HashMap<u8, u64>,
    stack: Vec<u64>,
}

impl VM {
    pub fn interpret(&mut self, code: &[u8]) {
        let mut pc = 0;
        while pc < code.len() {
            match code[pc] {
                PUSH => {
                    let val = code[pc + 1] as u64;
                    self.stack.push(val);
                    pc += 2;
                }
                SSTORE => {
                    let key = code[pc + 1];
                    let val = self.stack.pop().unwrap_or(0);
                    self.memory.insert(key, val);
                    pc += 2;
                }
                SLOAD => {
                    let key = code[pc + 1];
                    let val = *self.memory.get(&key).unwrap_or(&0);
                    self.stack.push(val);
                    pc += 2;
                }
                ADD => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    self.stack.push(a.wrapping_add(b));
                    pc += 1;
                }
                SUB => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    self.stack.push(a.wrapping_sub(b));
                    pc += 1;
                }
                STOP => break,
                _ => panic!("invalid opcode: {}", code[pc]),
            }
        }
    }

    pub fn stack(&self) -> &[u64] {
        &self.stack
    }

    pub fn memory(&self) -> &HashMap<u8, u64> {
        &self.memory
    }
}
