mod instructions;
mod jit;
mod vm;

use dynasmrt::AssemblyOffset;
use instructions::opcodes::*;
use jit::make_jit;
use rand::{Rng, SeedableRng};
use std::{
    fs::File,
    io::Write,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use vm::VM;

const MEM_SLOTS: usize = 256;
const NUM_CASES: usize = 10;
const PROG_LEN: usize = 20;

fn random_program(rng: &mut impl Rng, len: usize) -> Vec<u8> {
    let mut code = Vec::new();
    let mut depth = 0;
    for _ in 0..len {
        let choices: &[u8] = if depth < 2 {
            &[PUSH, SLOAD]
        } else {
            &[PUSH, SLOAD, SSTORE, ADD, SUB]
        };
        let op = choices[rng.gen_range(0..choices.len())];
        match op {
            PUSH => {
                let val = rng.gen_range(0..100) as u8;
                code.push(PUSH);
                code.push(val);
                depth += 1;
            }
            SLOAD => {
                let key = rng.gen_range(0..16) as u8;
                code.push(SLOAD);
                code.push(key);
                depth += 1;
            }
            SSTORE => {
                let key = rng.gen_range(0..16) as u8;
                code.push(SSTORE);
                code.push(key);
                if depth > 0 {
                    depth -= 1;
                }
            }
            ADD => {
                code.push(ADD);
                if depth >= 2 {
                    depth -= 1;
                }
            }
            SUB => {
                code.push(SUB);
                if depth >= 2 {
                    depth -= 1;
                }
            }
            _ => unreachable!(),
        }
    }
    code.push(STOP);
    code
}

fn hex(code: &[u8]) -> String {
    code.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

fn mem_snapshot(mem: &[u64]) -> Vec<(usize, u64)> {
    mem.iter()
        .enumerate()
        .filter_map(|(i, &v)| if v != 0 { Some((i, v)) } else { None })
        .collect()
}

fn main() -> std::io::Result<()> {
    println!("Running {} random programs", NUM_CASES);
    std::fs::create_dir_all("reports")?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut file = File::create(format!("reports/run-{}.log", ts))?;
    let mut rng = rand::rngs::StdRng::seed_from_u64(ts);

    for case in 0..NUM_CASES {
        let code = random_program(&mut rng, PROG_LEN);

        // Interpreter
        let mut vm = VM::default();
        let t0 = Instant::now();
        vm.interpret(&code);
        let interp_time = t0.elapsed();

        // JIT
        let mut mem = vec![0u64; MEM_SLOTS];
        let buf = make_jit(&code);
        let entry = buf.ptr(AssemblyOffset(0));
        let jit_fn: extern "C" fn(*mut u64) = unsafe { std::mem::transmute(entry) };
        let t1 = Instant::now();
        jit_fn(mem.as_mut_ptr());
        let jit_time = t1.elapsed();

        // Report
        writeln!(file, "case {}", case)?;
        writeln!(file, "  bytecode: {}", hex(&code))?;
        writeln!(file, "  interp_stack: {:?}", vm.stack())?;
        writeln!(file, "  interp_mem: {:?}", vm.memory())?;
        writeln!(file, "  interp_time_ns: {}", interp_time.as_nanos())?;
        writeln!(file, "  jit_mem: {:?}", mem_snapshot(&mem))?;
        writeln!(file, "  jit_time_ns: {}", jit_time.as_nanos())?;
        writeln!(file)?;
    }

    println!("Report written to reports/run-{}.log", ts);
    Ok(())
}
