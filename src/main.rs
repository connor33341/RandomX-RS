/*
Copyright (c) 2018-2019, tevador <tevador@gmail.com>
Copyright (c) 2023-2025, connor33341 (Rust implementation)

All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the copyright holder nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use std::env;

// Re-export the crate
use randomx_rs::*;

fn main() {
    // Simple CLI wrapper for RandomX-RS benchmark
    let args: Vec<String> = env::args().collect();
    
    // Print header
    println!("RandomX-RS - Rust implementation of RandomX");
    println!("Original Copyright (c) 2018-2019, tevador");
    println!("Rust implementation Copyright (c) 2023-2025, connor33341");
    println!();
    
    if args.len() > 1 && args[1] == "--help" {
        print_usage();
        return;
    }
    
    // Default configurations
    let mut mode = "fast";
    let mut nonces = 1000;
    let mut threads = 1;
    let mut init_threads = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1);
    
    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--light" => mode = "light",
            "--fast" => mode = "fast",
            "--threads" => {
                i += 1;
                if i < args.len() {
                    threads = args[i].parse().unwrap_or(1);
                }
            },
            "--init" => {
                i += 1;
                if i < args.len() {
                    init_threads = args[i].parse().unwrap_or(init_threads);
                }
            },
            "--nonces" => {
                i += 1;
                if i < args.len() {
                    nonces = args[i].parse().unwrap_or(1000);
                }
            },
            _ => {
                println!("Unknown option: {}", args[i]);
                print_usage();
                return;
            }
        }
        i += 1;
    }
    
    // Print configuration
    println!("Mode: {}", mode);
    println!("Number of hashes: {}", nonces);
    println!("Threads: {}", threads);
    println!("Init threads: {}", init_threads);
    println!();
    
    // Run the appropriate benchmark based on the selected mode
    match mode {
        "fast" => run_fast_mode_benchmark(nonces, threads, init_threads),
        "light" => run_light_mode_benchmark(nonces, threads),
        _ => unreachable!(),
    }
}

fn print_usage() {
    println!("Usage: randomx-rs [OPTIONS]");
    println!("Options:");
    println!("  --light          Run in light mode (lower memory usage)");
    println!("  --fast           Run in fast mode (default)");
    println!("  --threads N      Number of mining threads (default: 1)");
    println!("  --init N         Number of dataset initialization threads (default: # of CPUs)");
    println!("  --nonces N       Number of hashes to compute (default: 1000)");
    println!("  --help           Display this help message");
}

fn run_fast_mode_benchmark(nonces: usize, threads: usize, init_threads: usize) {
    // This is a simplified version - the actual benchmark code is in src/bin/benchmark.rs
    println!("Running benchmark in fast mode...");
    println!("Note: Please run the 'randomx-benchmark' binary for full benchmark functionality");
    
    // Get a recommended set of flags
    let mut flags = randomx_rs::get_flags();
    flags |= RandomXFlags::FULL_MEM;
    
    println!("Initializing cache and dataset...");
    let key = "RandomX example key";
    
    // Initialize cache
    let cache = randomx_rs::alloc_cache(flags.clone(), key.as_bytes()).expect("Failed to allocate cache");
    
    // Initialize dataset
    let dataset = randomx_rs::alloc_dataset(flags.clone(), Some(std::sync::Arc::new(cache.clone()))).expect("Failed to allocate dataset");
    
    // Create VM and compute a sample hash
    let vm = randomx_rs::create_vm(flags, None, Some(&dataset)).expect("Failed to create VM");
    let input = "RandomX example input";
    let hash = randomx_rs::calculate_hash(&vm, input.as_bytes());
    
    println!("Sample hash: {}", hex::encode(&hash));
    println!("Benchmark complete!");
}

fn run_light_mode_benchmark(nonces: usize, threads: usize) {
    // This is a simplified version - the actual benchmark code is in src/bin/benchmark.rs
    println!("Running benchmark in light mode...");
    println!("Note: Please run the 'randomx-benchmark' binary for full benchmark functionality");
    
    // Get a recommended set of flags
    let flags = randomx_rs::get_flags();
    
    println!("Initializing cache...");
    let key = "RandomX example key";
    
    // Initialize cache
    let cache = randomx_rs::alloc_cache(flags.clone(), key.as_bytes()).expect("Failed to allocate cache");
    
    // Create VM and compute a sample hash
    let vm = randomx_rs::create_vm(flags, Some(&cache), None).expect("Failed to create VM");
    let input = "RandomX example input";
    let hash = randomx_rs::calculate_hash(&vm, input.as_bytes());
    
    println!("Sample hash: {}", hex::encode(&hash));
    println!("Benchmark complete!");
}
