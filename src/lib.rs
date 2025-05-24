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

use std::ffi::{c_void, c_int};
use std::os::raw::c_char;
use std::mem;
use std::ptr;
use std::slice;

use bitflags::bitflags;

// Import public interfaces
pub use crate::cpu::{CPU, has_feature};
pub use crate::dataset::{Cache, Dataset};
pub use crate::vm::VirtualMachine;

// Core modules
mod argon2;
mod blake2;
mod common;
mod cpu;
mod dataset;
mod instructions;
mod jit;
mod vm;
mod vm_interpreted;

// RandomX flag constants
bitflags! {
    /// RandomX algorithm configuration flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RandomXFlags: u32 {
        /// Default (no flags)
        const DEFAULT = 0;
        
        /// Use full-memory mode for higher security (2GB memory usage)
        const FULL_MEM = 1;
        
        /// Use light-memory mode (256MB memory usage)
        const LIGHT_MEM = 2;
        
        /// Use JIT compilation mode for better performance where available
        const JIT = 4;
        
        /// Use large pages if available, for better performance
        const LARGE_PAGES = 8;
        
        /// Use hardware AES acceleration when available (x86 platforms)
        const HARD_AES = 16;
        
        /// For platforms without hardware AES
        const SOFT_AES = 32;
        
        /// Run in "secure" mode (mitigates some side-channel attacks)
        const SECURE = 64;
        
        /// Use NUMA-aware memory allocation on multi-CPU systems
        const NUMA = 128;
        
        /// Disable CPU-specific optimizations
        const ARGON2_SSSE3 = 256;
        
        /// Enable AVX2 optimizations for Argon2
        const ARGON2_AVX2 = 512;
    }
}

impl Default for RandomXFlags {
    fn default() -> Self {
        RandomXFlags::DEFAULT
    }
}

/// Returns recommended RandomX flags for current CPU architecture
pub fn get_flags() -> RandomXFlags {
    // Get raw flags from native library
    extern "C" {
        fn randomx_get_flags() -> u32;
    }
    
    let flags = unsafe { randomx_get_flags() };
    RandomXFlags::from_bits_truncate(flags)
}

/// Returns the number of items in a RandomX dataset
pub fn dataset_item_count() -> u32 {
    extern "C" {
        fn randomx_dataset_item_count() -> u64;
    }
    
    unsafe { randomx_dataset_item_count() as u32 }
}

/// Allocates a RandomX cache
pub fn alloc_cache(flags: RandomXFlags) -> Option<Cache> {
    extern "C" {
        fn randomx_alloc_cache(flags: u32) -> *mut c_void;
    }
    
    let ptr = unsafe { randomx_alloc_cache(flags.bits()) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { Cache::from_raw(ptr) })
    }
}

/// Allocates a RandomX dataset
pub fn alloc_dataset(flags: RandomXFlags) -> Option<Dataset> {
    extern "C" {
        fn randomx_alloc_dataset(flags: u32) -> *mut c_void;
    }
    
    let ptr = unsafe { randomx_alloc_dataset(flags.bits()) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { Dataset::from_raw(ptr) })
    }
}

/// Creates a RandomX virtual machine
pub fn create_vm(flags: RandomXFlags, cache: Option<&Cache>, dataset: Option<&Dataset>) -> Option<VirtualMachine> {
    extern "C" {
        fn randomx_create_vm(flags: u32, cache: *mut c_void, dataset: *mut c_void) -> *mut c_void;
    }
    
    let cache_ptr = cache.map_or(std::ptr::null_mut(), |c| c.as_ptr() as *mut c_void);
    let dataset_ptr = dataset.map_or(std::ptr::null_mut(), |d| d.as_ptr() as *mut c_void);
    
    let ptr = unsafe { randomx_create_vm(flags.bits(), cache_ptr, dataset_ptr) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { VirtualMachine::from_raw(ptr) })
    }
}

/// Calculate a RandomX hash
pub fn calculate_hash(vm: &VirtualMachine, input: &[u8]) -> [u8; 32] {
    extern "C" {
        fn randomx_calculate_hash(vm: *mut c_void, input: *const c_void, input_size: usize, output: *mut u8);
    }
    
    let mut output = [0u8; 32];
    unsafe {
        randomx_calculate_hash(
            vm.as_ptr() as *mut c_void, 
            input.as_ptr() as *const c_void, 
            input.len(),
            output.as_mut_ptr()
        );
    }
    output
}

/// Calculate multiple RandomX hashes with the same virtual machine
pub fn calculate_hash_first(vm: &VirtualMachine, input: &[u8]) {
    extern "C" {
        fn randomx_calculate_hash_first(vm: *mut c_void, input: *const c_void, input_size: usize);
    }
    
    unsafe {
        randomx_calculate_hash_first(
            vm.as_ptr() as *mut c_void, 
            input.as_ptr() as *const c_void, 
            input.len()
        );
    }
}

/// Calculate next hash in a chain
pub fn calculate_hash_next(vm: &VirtualMachine, input: &[u8]) -> [u8; 32] {
    extern "C" {
        fn randomx_calculate_hash_next(vm: *mut c_void, input: *const c_void, input_size: usize, output: *mut u8);
    }
    
    let mut output = [0u8; 32];
    unsafe {
        randomx_calculate_hash_next(
            vm.as_ptr() as *mut c_void, 
            input.as_ptr() as *const c_void, 
            input.len(),
            output.as_mut_ptr()
        );
    }
    output
}

/// Finalize hash calculation
pub fn calculate_hash_last(vm: &VirtualMachine) -> [u8; 32] {
    extern "C" {
        fn randomx_calculate_hash_last(vm: *mut c_void, output: *mut u8);
    }
    
    let mut output = [0u8; 32];
    unsafe {
        randomx_calculate_hash_last(vm.as_ptr() as *mut c_void, output.as_mut_ptr());
    }
    output
}

/// Create a new RandomX cache
pub fn create_cache(flags: RandomXFlags, key: &[u8]) -> Option<Cache> {
    Cache::new(flags, key)
}

/// Create a new RandomX dataset
pub fn create_dataset(flags: RandomXFlags, cache: &Cache) -> Option<Dataset> {
    Dataset::new(flags, cache)
}

/// Create a new RandomX virtual machine
pub fn create_vm(flags: RandomXFlags, cache: &Cache, dataset: Option<&Dataset>) -> Option<RandomXVM> {
    RandomXVM::new(flags, cache, dataset)
}