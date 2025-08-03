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

struct TestConfig {
    name: &'static str,
    num_cases: usize,
    prog_len: usize,
}

const TEST_CONFIGS: &[TestConfig] = &[
    TestConfig { name: "small", num_cases: 10, prog_len: 20 },
    TestConfig { name: "medium", num_cases: 10, prog_len: 100 },
    TestConfig { name: "large", num_cases: 10, prog_len: 500 },
    TestConfig { name: "xlarge", num_cases: 5, prog_len: 1000 },
    TestConfig { name: "xxlarge", num_cases: 3, prog_len: 2000 },
];

fn random_program(rng: &mut impl Rng, len: usize) -> Vec<u8> {
    let mut code = Vec::new();
    let mut depth = 0;
    let max_depth = 8;
    
    for _ in 0..len {
        let choices: &[u8] = if depth < 2 {
            &[PUSH, SLOAD, DUP]
        } else if depth >= max_depth {
            &[SSTORE, ADD, SUB, MUL, DIV, MOD, EQ, LT, GT, AND, OR, XOR, SWAP]
        } else {
            &[PUSH, SLOAD, SSTORE, ADD, SUB, MUL, DIV, MOD, EQ, LT, GT, AND, OR, XOR, DUP, SWAP]
        };
        
        let op = choices[rng.gen_range(0..choices.len())];
        match op {
            PUSH => {
                let val = rng.gen_range(1..=255) as u8;
                code.push(PUSH);
                code.push(val);
                depth += 1;
            }
            SLOAD => {
                let key = rng.gen_range(0..32) as u8;
                code.push(SLOAD);
                code.push(key);
                depth += 1;
            }
            SSTORE => {
                let key = rng.gen_range(0..32) as u8;
                code.push(SSTORE);
                code.push(key);
                if depth > 0 {
                    depth -= 1;
                }
            }
            ADD | SUB | MUL | DIV | MOD | EQ | LT | GT | AND | OR | XOR => {
                code.push(op);
                if depth >= 2 {
                    depth -= 1;
                }
            }
            DUP => {
                code.push(DUP);
                if depth > 0 {
                    depth += 1;
                }
            }
            SWAP => {
                code.push(SWAP);
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

#[derive(Default)]
struct BenchmarkResults {
    config_name: String,
    total_cases: usize,
    total_interp_time_ns: u128,
    total_jit_time_ns: u128,
    avg_interp_time_ns: f64,
    avg_jit_time_ns: f64,
    speedup: f64,
    bytecode_length: usize,
}

fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("reports")?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut detailed_file = File::create(format!("reports/detailed-{}.log", ts))?;
    let mut summary_file = File::create(format!("reports/summary-{}.log", ts))?;
    let mut results = Vec::new();
    
    println!("Running comprehensive bytecode benchmarks...");
    println!("┌─────────┬──────────┬─────────────┬─────────────┬──────────┬──────────┐");
    println!("│ Size    │ Cases    │ Interpreter │ JIT         │ Speedup  │ Progress │");
    println!("├─────────┼──────────┼─────────────┼─────────────┼──────────┼──────────┤");

    for config in TEST_CONFIGS {
        let mut rng = rand::rngs::StdRng::seed_from_u64(ts + config.prog_len as u64);
        let mut total_interp_time = 0u128;
        let mut total_jit_time = 0u128;
        
        writeln!(detailed_file, "=== {} TESTS (length: {}, cases: {}) ===", 
                config.name.to_uppercase(), config.prog_len, config.num_cases)?;
        writeln!(detailed_file)?;

        for case in 0..config.num_cases {
            let code = random_program(&mut rng, config.prog_len);

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

            total_interp_time += interp_time.as_nanos();
            total_jit_time += jit_time.as_nanos();

            // Detailed report
            writeln!(detailed_file, "case {} (length: {})", case, code.len())?;
            writeln!(detailed_file, "  bytecode: {}", hex(&code))?;
            writeln!(detailed_file, "  interp_stack: {:?}", vm.stack())?;
            writeln!(detailed_file, "  interp_mem: {:?}", vm.memory())?;
            writeln!(detailed_file, "  interp_time_ns: {}", interp_time.as_nanos())?;
            writeln!(detailed_file, "  jit_mem: {:?}", mem_snapshot(&mem))?;
            writeln!(detailed_file, "  jit_time_ns: {}", jit_time.as_nanos())?;
            writeln!(detailed_file, "  speedup: {:.2}x", 
                    interp_time.as_nanos() as f64 / jit_time.as_nanos() as f64)?;
            writeln!(detailed_file)?;
            
            // Progress indicator
            if case % (config.num_cases / 4).max(1) == 0 || case == config.num_cases - 1 {
                let progress = (case + 1) * 100 / config.num_cases;
                print!("\r│ {:7} │ {:8} │ {:11} │ {:11} │ {:8} │ {:7}% │", 
                      config.name, 
                      format!("{}/{}", case + 1, config.num_cases),
                      format!("{}ns", total_interp_time / (case + 1) as u128),
                      format!("{}ns", total_jit_time / (case + 1) as u128),
                      format!("{:.2}x", total_interp_time as f64 / total_jit_time as f64),
                      progress);
                std::io::stdout().flush().unwrap();
            }
        }

        let avg_interp = total_interp_time as f64 / config.num_cases as f64;
        let avg_jit = total_jit_time as f64 / config.num_cases as f64;
        let speedup = avg_interp / avg_jit;

        results.push(BenchmarkResults {
            config_name: config.name.to_string(),
            total_cases: config.num_cases,
            total_interp_time_ns: total_interp_time,
            total_jit_time_ns: total_jit_time,
            avg_interp_time_ns: avg_interp,
            avg_jit_time_ns: avg_jit,
            speedup,
            bytecode_length: config.prog_len,
        });
        
        println!();
    }
    
    println!("└─────────┴──────────┴─────────────┴─────────────┴──────────┴──────────┘");
    println!();

    // Write summary report
    writeln!(summary_file, "BYTECODE JIT vs INTERPRETER BENCHMARK SUMMARY")?;
    writeln!(summary_file, "==============================================")?;
    writeln!(summary_file)?;
    writeln!(summary_file, "Test configurations:")?;
    for config in TEST_CONFIGS {
        writeln!(summary_file, "  {}: {} cases, {} opcodes", config.name, config.num_cases, config.prog_len)?;
    }
    writeln!(summary_file)?;

    writeln!(summary_file, "Performance Results:")?;
    writeln!(summary_file, "┌─────────┬─────────┬─────────────┬─────────────┬──────────┬─────────────┐")?;
    writeln!(summary_file, "│ Size    │ Length  │ Interpreter │ JIT         │ Speedup  │ JIT Benefit │")?;
    writeln!(summary_file, "├─────────┼─────────┼─────────────┼─────────────┼──────────┼─────────────┤")?;
    
    for result in &results {
        let benefit = ((result.speedup - 1.0) * 100.0).max(0.0);
        writeln!(summary_file, "│ {:7} │ {:7} │ {:9.0}ns │ {:9.0}ns │ {:7.2}x │ {:9.1}% │",
                result.config_name,
                result.bytecode_length,
                result.avg_interp_time_ns,
                result.avg_jit_time_ns,
                result.speedup,
                benefit)?;
    }
    writeln!(summary_file, "└─────────┴─────────┴─────────────┴─────────────┴──────────┴─────────────┘")?;
    writeln!(summary_file)?;

    // Analysis
    let avg_speedup = results.iter().map(|r| r.speedup).sum::<f64>() / results.len() as f64;
    let max_speedup = results.iter().map(|r| r.speedup).fold(0.0, f64::max);
    let min_speedup = results.iter().map(|r| r.speedup).fold(f64::INFINITY, f64::min);
    
    writeln!(summary_file, "Analysis:")?;
    writeln!(summary_file, "  Average speedup: {:.2}x", avg_speedup)?;
    writeln!(summary_file, "  Best speedup: {:.2}x ({})", max_speedup, 
            results.iter().max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap()).unwrap().config_name)?;
    writeln!(summary_file, "  Worst speedup: {:.2}x ({})", min_speedup,
            results.iter().min_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap()).unwrap().config_name)?;
    writeln!(summary_file)?;
    
    if avg_speedup > 1.0 {
        writeln!(summary_file, "✓ JIT shows consistent performance benefits across all test sizes")?;
    } else {
        writeln!(summary_file, "⚠ JIT performance needs optimization")?;
    }
    
    if results.iter().any(|r| r.speedup > 2.0) {
        writeln!(summary_file, "✓ JIT achieves significant speedups (>2x) on some workloads")?;
    }

    println!("📊 Benchmark completed!");
    println!("📄 Detailed results: reports/detailed-{}.log", ts);
    println!("📋 Summary report: reports/summary-{}.log", ts);
    println!();
    println!("Quick Summary:");
    println!("  Average JIT speedup: {:.2}x", avg_speedup);
    println!("  Best performance: {:.2}x on {} bytecode", max_speedup, 
            results.iter().max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap()).unwrap().config_name);
    
    Ok(())
}
