use dynasmrt::{dynasm, DynasmApi, ExecutableBuffer, AssemblyOffset};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Default)]
struct VM {
    memory: HashMap<u8, u64>,
    stack: Vec<u64>,
}

impl VM {
    fn interpret(&mut self, code: &[u8]) {
        let mut pc = 0;
        while pc < code.len() {
            match code[pc] {
                0x01 => { // SLOAD
                    let key = code[pc + 1];
                    let val = *self.memory.get(&key).unwrap_or(&0);
                    self.stack.push(val);
                    pc += 2;
                }
                0x02 => { // SSTORE
                    let key = code[pc + 1];
                    let val = self.stack.pop().unwrap();
                    self.memory.insert(key, val);
                    pc += 2;
                }
                0x03 => { // PUSH
                    let val = code[pc + 1] as u64;
                    self.stack.push(val);
                    pc += 2;
                }
                0xFF => break, // STOP
                _ => panic!("invalid opcode: {}", code[pc]),
            }
        }
    }

    fn print_stack(&self) {
        println!("stack: {:?}", self.stack);
    }

    fn print_memory(&self) {
        println!("memory: {:?}", self.memory);
    }
}

/// Builds a JIT function of signature `fn(*mut u64)`:
/// the JIT code will use its first argument (in RDI) as the base pointer
/// to an array of u64 slots (key*8 indexing).
fn make_jit(code: &[u8]) -> ExecutableBuffer {
    use dynasmrt::x64::Assembler;
    let mut ops = Assembler::new().unwrap();

    dynasm!(ops
        ; .arch x64
        // prologue: save RBX, move arg ptr (RDI) into RBX
        ; push rbx
        ; mov  rbx, rdi
    );

    let mut pc = 0;
    while pc < code.len() {
        match code[pc] {
            0x03 => { // PUSH <imm>
                let val = code[pc + 1] as i32;
                dynasm!(ops
                    ; mov  rax, val
                    ; push rax
                );
                pc += 2;
            }
            0x02 => { // SSTORE <key>
                let key = code[pc + 1] as i32;
                dynasm!(ops
                    ; pop  rax
                    ; mov  [rbx + key * 8], rax
                );
                pc += 2;
            }
            0x01 => { // SLOAD <key>
                let key = code[pc + 1] as i32;
                dynasm!(ops
                    ; mov  rax, [rbx + key * 8]
                    ; push rax
                );
                pc += 2;
            }
            0xFF => { // STOP
                dynasm!(ops
                    ; pop  rax   // remove the last PUSH
                    ; pop  rbx   // restore original RBX
                    ; ret
                );
                pc += 1;
            }
            _ => panic!("bad opcode: {}", code[pc]),
        }
    }

    ops.finalize().unwrap()
}

fn main() {
    println!("=== Debug VM PoC ===");
    println!("Host: {}/{}", std::env::consts::OS, std::env::consts::ARCH);

    let bytecode = vec![0x03, 42, 0x02, 0x10, 0x01, 0x10, 0xFF];

    // Interpreter
    let mut vm = VM::default();
    let t0 = Instant::now();
    vm.interpret(&bytecode);
    let di = t0.elapsed();
    vm.print_stack();
    vm.print_memory();

    // Prepare memory buffer
    const MEM_SLOTS: usize = 256;
    let mut mem = vec![0u64; MEM_SLOTS];

    // Build JIT
    let buf = make_jit(&bytecode);
    let entry = buf.ptr(AssemblyOffset(0));
    println!("JIT entry ptr = {:p}", entry);

    // Debug maps & dump
    println!("\n--- /proc/self/maps ---");
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        println!("{}", maps);
    }

    println!("\n--- JIT bytes @ {:p} ---", entry);
    unsafe {
        let slice = std::slice::from_raw_parts(entry as *const u8, 32);
        for (i, b) in slice.iter().enumerate() {
            if i % 8 == 0 { print!("\n0x{:02x}: ", i); }
            print!("{:02x} ", b);
        }
        println!();
    }

    // Call JIT
    println!("\n--- calling JIT now ---");
    let jit_fn: extern "C" fn(*mut u64) =
        unsafe { std::mem::transmute(entry) };
    let t1 = Instant::now();
    jit_fn(mem.as_mut_ptr());
    let dj = t1.elapsed();

    println!("JIT memory slot 0x10 = {}", mem[0x10]);
    println!("Interpreter time: {:?}", di);
    println!("JIT time: {:?}", dj);
}

