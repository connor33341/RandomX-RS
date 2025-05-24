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

#![allow(non_camel_case_types)]
#![allow(dead_code)]

extern crate bitflags;
extern crate libc;

// Re-export primary types
pub use crate::common::{FpuReg, IntReg, Addr, RandomXError, Result, RANDOMX_HASH_SIZE};
pub use crate::dataset::{Cache, Dataset};
pub use crate::vm::{VirtualMachine, RandomXVM};

// Import project modules
pub mod common;
pub mod dataset;
pub mod vm;
pub mod instructions;
pub mod cpu;
pub mod jit;
pub mod blake2;
pub mod argon2;
pub mod vm_interpreted;

/// RandomX algorithm flags
bitflags::bitflags! {
    pub struct RandomXFlags: u32 {
        /// No flags (default mode)
        const DEFAULT = 0;
        
        /// Use large memory mode (requires 2080 MiB)
        const LARGE_PAGES = 1;
        
        /// Use full dataset (requires 2080 MiB)
        const FULL_MEM = 2;
        
        /// Use JIT compiler (faster but potentially unsafe)
        const JIT = 4;
        
        /// Use secure JIT (where available)
        const SECURE = 8;
        
        /// Use hardware AES instructions if available
        const HARD_AES = 16;
        
        /// Initialize full dataset
        const FULL_DATASET = 32;
        
        /// Enable Argon2 SSSE3 optimization
        const ARGON2_SSSE3 = 64;
        
        /// Enable Argon2 AVX2 optimization
        const ARGON2_AVX2 = 128;
        
        /// Enable blake2 AVX2 optimization
        const BLAKE2_AVX2 = 256;
    }
}

/// Gets recommended RandomX flags based on current hardware capabilities.
/// 
/// This function will check for available CPU features and return the most 
/// appropriate set of flags for optimal RandomX performance.
///
/// # Returns
/// A `RandomXFlags` value with the recommended flags for the current hardware.
pub fn get_flags() -> RandomXFlags {
    extern "C" {
        fn randomx_get_flags() -> u32;
    }
    
    unsafe {
        RandomXFlags::from_bits_truncate(randomx_get_flags())
    }
}

/// Allocates a RandomX cache.
///
/// # Parameters
/// * `flags`: RandomX flags that may affect cache allocation
///
/// # Returns
/// A `Result` containing either the allocated `Cache` or an error
pub fn alloc_cache(flags: RandomXFlags) -> Result<Cache> {
    Cache::new(flags)
}

/// Returns the count of dataset items.
///
/// # Returns
/// Number of items in the full RandomX dataset
pub fn dataset_item_count() -> usize {
    extern "C" {
        fn randomx_dataset_item_count() -> usize;
    }
    
    unsafe {
        randomx_dataset_item_count()
    }
}

/// Allocates a RandomX dataset.
///
/// # Parameters
/// * `flags`: RandomX flags that may affect dataset allocation
///
/// # Returns
/// A `Result` containing either the allocated `Dataset` or an error
pub fn alloc_dataset(flags: RandomXFlags) -> Result<Dataset> {
    Dataset::new(flags)
}

/// Creates a RandomX virtual machine instance.
///
/// # Parameters
/// * `flags`: RandomX flags that affect VM behavior
/// * `cache`: Optional cache to use for light-mode verification
/// * `dataset`: Optional dataset to use for full verification mode
///
/// # Returns
/// A `Result` containing either the created `RandomXVM` or an error
pub fn create_vm(
    flags: RandomXFlags, 
    cache: Option<&Cache>, 
    dataset: Option<&Dataset>
) -> Result<RandomXVM> {
    RandomXVM::new(flags, cache, dataset)
}

/// Creates a pure Rust interpreted virtual machine instance.
///
/// # Parameters
/// * `dataset`: Optional dataset to use for full verification mode
/// * `mem_size`: Scratchpad memory size to allocate
///
/// # Returns
/// A `Result` containing either the created interpreted VM or an error
pub fn create_interpreted_vm(
    dataset: Option<&Dataset>,
    mem_size: usize
) -> Result<impl VirtualMachine> {
    use crate::vm_interpreted::InterpretedVirtualMachine;
    
    let dataset_ptr = dataset.map(|ds| {
        use std::ptr::NonNull;
        unsafe { NonNull::new_unchecked(ds.as_ptr() as *mut std::ffi::c_void) }
    });
    
    InterpretedVirtualMachine::new(dataset_ptr, mem_size)
        .map_err(|_| RandomXError::AllocationError)
}

/// Calculates a RandomX hash using the provided virtual machine.
///
/// # Parameters
/// * `vm`: Reference to a virtual machine instance
/// * `input`: Input data to hash
///
/// # Returns
/// A 32-byte hash as an array
pub fn calculate_hash(vm: &impl VirtualMachine, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
    unsafe {
        let mut vm_mut = std::mem::transmute::<&impl VirtualMachine, &mut impl VirtualMachine>(vm);
        vm_mut.calculate(input)
    }
}

/// Calculates multiple RandomX hashes using the provided virtual machine.
///
/// # Parameters
/// * `vm`: Reference to a virtual machine instance
/// * `inputs`: Vector of inputs to hash
///
/// # Returns
/// Vector of 32-byte hash arrays
pub fn calculate_hashes(vm: &impl VirtualMachine, inputs: &[&[u8]]) -> Vec<[u8; RANDOMX_HASH_SIZE]> {
    unsafe {
        let mut vm_mut = std::mem::transmute::<&impl VirtualMachine, &mut impl VirtualMachine>(vm);
        inputs.iter().map(|input| vm_mut.calculate(input)).collect()
    }
}

/// Calculates a RandomX hash with specified randomization info.
///
/// # Parameters
/// * `vm`: Reference to a virtual machine instance
/// * `input`: Input data to hash
/// * `info`: Randomization info for VM configuration
///
/// # Returns
/// A 32-byte hash as an array
pub fn calculate_hash_with_info(
    vm: &impl VirtualMachine, 
    input: &[u8], 
    info: &[u8]
) -> [u8; RANDOMX_HASH_SIZE] {
    unsafe {
        let mut vm_mut = std::mem::transmute::<&impl VirtualMachine, &mut impl VirtualMachine>(vm);
        vm_mut.calculate_with_info(input, info)
    }
}

/// Creates a RandomX hasher for convenient one-off hashing.
///
/// # Parameters
/// * `key`: Key used for VM initialization
/// * `flags`: RandomX flags to use
/// * `use_dataset`: Whether to use full dataset mode
/// * `threads`: Number of threads for dataset initialization
///
/// # Returns
/// A `Result` containing either a `RandomXHasher` or an error
pub fn create_hasher(
    key: &[u8], 
    flags: RandomXFlags, 
    use_dataset: bool, 
    threads: usize
) -> Result<RandomXHasher> {
    RandomXHasher::new(key, flags, use_dataset, threads)
}

/// Convenience type for performing one-off RandomX hashing
pub struct RandomXHasher {
    vm: RandomXVM,
    _cache: Cache,
    _dataset: Option<Dataset>,
}

impl RandomXHasher {
    /// Creates a new RandomX hasher.
    ///
    /// # Parameters
    /// * `key`: Key used for VM initialization
    /// * `flags`: RandomX flags to use
    /// * `use_dataset`: Whether to use full dataset mode
    /// * `threads`: Number of threads for dataset initialization
    ///
    /// # Returns
    /// A `Result` containing either a `RandomXHasher` or an error
    pub fn new(key: &[u8], flags: RandomXFlags, use_dataset: bool, threads: usize) -> Result<Self> {
        // Initialize cache
        let cache = Cache::new(flags)?;
        cache.init(key);
        
        // Initialize dataset if requested
        let dataset = if use_dataset {
            let ds = if threads > 1 {
                Dataset::init_parallel(flags, &cache, threads)?
            } else {
                Dataset::new(flags, &cache)?
            };
            Some(ds)
        } else {
            None
        };
        
        // Create VM
        let vm = RandomXVM::new(
            flags,
            if dataset.is_none() { Some(&cache) } else { None },
            dataset.as_ref()
        )?;
        
        Ok(Self {
            vm,
            _cache: cache,
            _dataset: dataset,
        })
    }
    
    /// Calculate hash of the input data
    ///
    /// # Parameters
    /// * `input`: Data to hash
    ///
    /// # Returns
    /// A 32-byte hash as an array
    pub fn hash(&self, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
        unsafe {
            let mut vm_mut = std::mem::transmute::<&RandomXVM, &mut RandomXVM>(&self.vm);
            vm_mut.calculate(input)
        }
    }
    
    /// Calculate hash with specific randomization info
    ///
    /// # Parameters
    /// * `input`: Data to hash
    /// * `info`: Randomization info
    ///
    /// # Returns
    /// A 32-byte hash as an array
    pub fn hash_with_info(&self, input: &[u8], info: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
        unsafe {
            let mut vm_mut = std::mem::transmute::<&RandomXVM, &mut RandomXVM>(&self.vm);
            vm_mut.calculate_with_info(input, info)
        }
    }
}