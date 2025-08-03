use crate::instructions::opcodes::*;
use dynasmrt::{DynasmApi, ExecutableBuffer, dynasm};

/// Builds a JIT function of signature `fn(*mut u64)`
/// the JIT code uses its first argument (in RDI) as the base pointer
/// to an array of u64 slots (key*8 indexing).
pub fn make_jit(code: &[u8]) -> ExecutableBuffer {
    use dynasmrt::x64::Assembler;
    let mut ops = Assembler::new().unwrap();

    dynasm!(ops
        ; .arch x64
        // prologue: save RBX and R12, move arg ptr (RDI) into RBX
        ; push rbx
        ; push r12
        ; mov  rbx, rdi
        ; mov  r12, rsp
    );

    let mut pc = 0;
    while pc < code.len() {
        match code[pc] {
            PUSH => {
                let val = code[pc + 1] as i32;
                dynasm!(ops
                    ; mov  rax, val
                    ; push rax
                );
                pc += 2;
            }
            SSTORE => {
                let key = code[pc + 1] as i32;
                dynasm!(ops
                    ; pop  rax
                    ; mov  [rbx + key * 8], rax
                );
                pc += 2;
            }
            SLOAD => {
                let key = code[pc + 1] as i32;
                dynasm!(ops
                    ; mov  rax, [rbx + key * 8]
                    ; push rax
                );
                pc += 2;
            }
            ADD => {
                dynasm!(ops
                    ; pop  rax
                    ; pop  rdx
                    ; add  rax, rdx
                    ; push rax
                );
                pc += 1;
            }
            SUB => {
                dynasm!(ops
                    ; pop  rdx
                    ; pop  rax
                    ; sub  rax, rdx
                    ; push rax
                );
                pc += 1;
            }
            STOP => {
                dynasm!(ops
                    ; mov rsp, r12
                    ; pop r12
                    ; pop rbx
                    ; ret
                );
                pc += 1;
            }
            _ => panic!("bad opcode: {}", code[pc]),
        }
    }

    ops.finalize().unwrap()
}
