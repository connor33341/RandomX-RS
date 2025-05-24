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
pub use crate::common::{FpuReg, IntReg, Addr};
pub use crate::dataset::{Cache, Dataset};
pub use crate::vm::{VirtualMachine};

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

        /// Use LARGE_PAGES for memory allocation if available - Windows only
        const FLAG_LARGE_PAGES = 1;

        /// Initialize the dataset on creation
        const FLAG_FULL_MEM = 2;

        /// Use JIT compilation for VM instructions
        const FLAG_JIT = 4;

        /// Optimize compilation using AVX extension
        const FLAG_HARD_AES = 8;

        /// Calculate dataset without writing it to memory (for embedding)
        const FLAG_FULL_MEM_EMBEDABLE = 16;

        /// Use argon2_ssse3 implementation
        const FLAG_ARGON2_SSSE3 = 32;

        /// Use argon2_avx2 implementation
        const FLAG_ARGON2_AVX2 = 64;

        /// Validate outputs of the execution
        const FLAG_DEBUG = 128;

        /// Enable secure mode (scratchpad cleared after each hash)
        const FLAG_SECURE = 256;
    }
}

/// Calculate a RandomX hash using the provided VM
/// 
/// # Arguments
/// 
/// * `vm` - Reference to a VirtualMachine instance
/// * `input` - Input data to hash
/// * `out` - Mutable slice to store the output hash
pub fn calculate_hash(vm: &VirtualMachine, input: &[u8], out: &mut [u8]) {
    extern "C" {
        fn randomx_calculate_hash(
            vm: *const libc::c_void,
            input: *const libc::c_void,
            inputSize: libc::size_t,
            output: *mut libc::c_void,
        );
    }

    unsafe {
        randomx_calculate_hash(
            vm.as_ptr(),
            input.as_ptr() as *const libc::c_void,
            input.len(),
            out.as_mut_ptr() as *mut libc::c_void,
        );
    }
}

/// Create a virtual machine instance
/// 
/// # Arguments
/// 
/// * `flags` - RandomX flags
/// * `cache` - Optional reference to a RandomX cache (for light mode)
/// * `dataset` - Optional reference to a RandomX dataset (for full mode)
pub fn create_vm(flags: RandomXFlags, cache: Option<&Cache>, dataset: Option<&Dataset>) -> Option<VirtualMachine> {
    VirtualMachine::new(flags, cache, dataset)
}