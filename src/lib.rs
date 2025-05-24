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

use std::ffi::{c_void, c_ulong};
use std::ptr::NonNull;
use bitflags::bitflags;
use libc::{size_t};

// Constants matching the C/C++ implementation
pub const RANDOMX_HASH_SIZE: usize = 32;
pub const RANDOMX_DATASET_ITEM_SIZE: usize = 64;

// Modules
mod argon2;
mod dataset;
mod vm;
mod blake2;
mod common;
mod cpu;
mod jit;

// Re-exports
pub use dataset::{Cache, Dataset};
pub use vm::VirtualMachine;

bitflags! {
    /// Flags to configure RandomX behavior
    #[repr(C)]
    pub struct RandomXFlags: u32 {
        const DEFAULT = 0;
        const LARGE_PAGES = 1;
        const HARD_AES = 2;
        const FULL_MEM = 4;
        const JIT = 8;
        const SECURE = 16;
        const ARGON2_SSSE3 = 32;
        const ARGON2_AVX2 = 64;
        const ARGON2 = Self::ARGON2_SSSE3.bits() | Self::ARGON2_AVX2.bits();
    }
}

/// Gets the recommended flags to be used on the current machine.
/// 
/// This doesn't include:
/// - `RANDOMX_FLAG_LARGE_PAGES`
/// - `RANDOMX_FLAG_FULL_MEM`
/// - `RANDOMX_FLAG_SECURE`
/// 
/// These flags must be added manually if desired.
pub fn get_flags() -> RandomXFlags {
    // Call into the C implementation for now, we'll replace this with pure Rust later
    unsafe { randomx_get_flags() }
}

/// Allocates and initializes a new RandomX cache
/// 
/// # Arguments
/// 
/// * `flags` - Configuration flags for the cache allocation
/// 
/// # Returns
/// 
/// A new RandomX cache or None if allocation fails
pub fn alloc_cache(flags: RandomXFlags) -> Option<Cache> {
    // For now, this is just a thin wrapper around the C implementation
    unsafe {
        let ptr = randomx_alloc_cache(flags.bits());
        if ptr.is_null() {
            None
        } else {
            Some(Cache::from_raw(ptr))
        }
    }
}

/// Allocates and initializes a new RandomX dataset
/// 
/// # Arguments
/// 
/// * `flags` - Configuration flags for the dataset allocation
/// 
/// # Returns
/// 
/// A new RandomX dataset or None if allocation fails
pub fn alloc_dataset(flags: RandomXFlags) -> Option<Dataset> {
    // For now, this is just a thin wrapper around the C implementation
    unsafe {
        let ptr = randomx_alloc_dataset(flags.bits());
        if ptr.is_null() {
            None
        } else {
            Some(Dataset::from_raw(ptr))
        }
    }
}

/// Gets the number of items contained in the dataset
pub fn dataset_item_count() -> u64 {
    unsafe { randomx_dataset_item_count() as u64 }
}

/// Creates and initializes a RandomX virtual machine
/// 
/// # Arguments
/// 
/// * `flags` - Configuration flags for the VM
/// * `cache` - An initialized cache, can be None if FULL_MEM flag is set
/// * `dataset` - An initialized dataset, can be None if FULL_MEM flag is not set
/// 
/// # Returns
/// 
/// A new RandomX virtual machine or None if initialization fails
pub fn create_vm(flags: RandomXFlags, cache: Option<&Cache>, dataset: Option<&Dataset>) -> Option<VirtualMachine> {
    unsafe {
        let cache_ptr = match cache {
            Some(c) => c.as_raw(),
            None => std::ptr::null_mut(),
        };
        
        let dataset_ptr = match dataset {
            Some(d) => d.as_raw(),
            None => std::ptr::null_mut(),
        };
        
        let vm_ptr = randomx_create_vm(flags.bits(), cache_ptr, dataset_ptr);
        if vm_ptr.is_null() {
            None
        } else {
            Some(VirtualMachine::from_raw(vm_ptr))
        }
    }
}

/// Calculate a RandomX hash
/// 
/// # Arguments
/// 
/// * `vm` - Virtual machine instance
/// * `input` - Input data to hash
/// 
/// # Returns
/// 
/// A 32-byte hash
pub fn calculate_hash(vm: &VirtualMachine, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
    let mut output = [0u8; RANDOMX_HASH_SIZE];
    unsafe {
        randomx_calculate_hash(
            vm.as_raw(),
            input.as_ptr() as *const c_void,
            input.len(),
            output.as_mut_ptr() as *mut c_void
        );
    }
    output
}

/// Calculate multiple hashes efficiently using multi-part API
pub mod multi_hash {
    use super::*;
    
    pub fn calculate_first(vm: &VirtualMachine, input: &[u8]) {
        unsafe {
            randomx_calculate_hash_first(
                vm.as_raw(),
                input.as_ptr() as *const c_void,
                input.len()
            );
        }
    }
    
    pub fn calculate_next(vm: &VirtualMachine, next_input: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
        let mut output = [0u8; RANDOMX_HASH_SIZE];
        unsafe {
            randomx_calculate_hash_next(
                vm.as_raw(),
                next_input.as_ptr() as *const c_void,
                next_input.len(),
                output.as_mut_ptr() as *mut c_void
            );
        }
        output
    }
    
    pub fn calculate_last(vm: &VirtualMachine) -> [u8; RANDOMX_HASH_SIZE] {
        let mut output = [0u8; RANDOMX_HASH_SIZE];
        unsafe {
            randomx_calculate_hash_last(
                vm.as_raw(),
                output.as_mut_ptr() as *mut c_void
            );
        }
        output
    }
}

/// Calculate a RandomX commitment from a hash and its input
/// 
/// # Arguments
/// 
/// * `input` - The input data that was hashed
/// * `hash` - The resulting hash
/// 
/// # Returns
/// 
/// A 32-byte commitment
pub fn calculate_commitment(input: &[u8], hash: &[u8; RANDOMX_HASH_SIZE]) -> [u8; RANDOMX_HASH_SIZE] {
    let mut output = [0u8; RANDOMX_HASH_SIZE];
    unsafe {
        randomx_calculate_commitment(
            input.as_ptr() as *const c_void, 
            input.len(),
            hash.as_ptr() as *const c_void,
            output.as_mut_ptr() as *mut c_void
        );
    }
    output
}

// Foreign function interface to the C implementation
// We'll gradually replace these calls with pure Rust implementations
#[link(name = "randomx")]
extern "C" {
    fn randomx_get_flags() -> u32;
    fn randomx_alloc_cache(flags: u32) -> *mut c_void;
    fn randomx_init_cache(cache: *mut c_void, key: *const c_void, key_size: size_t);
    fn randomx_release_cache(cache: *mut c_void);
    fn randomx_alloc_dataset(flags: u32) -> *mut c_void;
    fn randomx_dataset_item_count() -> c_ulong;
    fn randomx_init_dataset(dataset: *mut c_void, cache: *mut c_void, start_item: c_ulong, item_count: c_ulong);
    fn randomx_get_dataset_memory(dataset: *mut c_void) -> *mut c_void;
    fn randomx_release_dataset(dataset: *mut c_void);
    fn randomx_create_vm(flags: u32, cache: *mut c_void, dataset: *mut c_void) -> *mut c_void;
    fn randomx_vm_set_cache(machine: *mut c_void, cache: *mut c_void);
    fn randomx_vm_set_dataset(machine: *mut c_void, dataset: *mut c_void);
    fn randomx_destroy_vm(machine: *mut c_void);
    fn randomx_calculate_hash(machine: *mut c_void, input: *const c_void, input_size: size_t, output: *mut c_void);
    fn randomx_calculate_hash_first(machine: *mut c_void, input: *const c_void, input_size: size_t);
    fn randomx_calculate_hash_next(machine: *mut c_void, next_input: *const c_void, next_input_size: size_t, output: *mut c_void);
    fn randomx_calculate_hash_last(machine: *mut c_void, output: *mut c_void);
    fn randomx_calculate_commitment(input: *const c_void, input_size: size_t, hash_in: *const c_void, com_out: *mut c_void);
}