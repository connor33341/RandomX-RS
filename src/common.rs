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

// Constants from configuration.h
// These would be auto-generated in Rust during the build process in real implementation
pub const RANDOMX_PROGRAM_SIZE: usize = 256;
pub const RANDOMX_PROGRAM_ITERATIONS: usize = 2048;
pub const RANDOMX_PROGRAM_COUNT: usize = 8;
pub const RANDOMX_SCRATCHPAD_L3: usize = 2097152;
pub const RANDOMX_SCRATCHPAD_L2: usize = 262144;
pub const RANDOMX_SCRATCHPAD_L1: usize = 16384;
pub const RANDOMX_DATASET_BASE_SIZE: usize = 2147483648;
pub const RANDOMX_DATASET_EXTRA_SIZE: usize = 33554368;
pub const RANDOMX_ARGON_MEMORY: usize = 262144;
pub const RANDOMX_ARGON_ITERATIONS: usize = 3;
pub const RANDOMX_ARGON_LANES: usize = 1;
pub const RANDOMX_ARGON_SALT: &[u8] = b"RandomX\x03";
pub const RANDOMX_CACHE_ACCESSES: usize = 8;
pub const RANDOMX_SUPERSCALAR_LATENCY: usize = 170;
pub const RANDOMX_DATASET_ITEM_SIZE: usize = 64;
pub const RANDOMX_JUMP_BITS: usize = 8;
pub const RANDOMX_JUMP_OFFSET: isize = 8;

// Address size
pub type Addr = u32;

// Integer register type
pub type IntReg = u64;

// Floating point register type
#[repr(C)]
pub struct FpuReg {
    pub lo: u64,
    pub hi: u64,
}

// Scratchpad constants
pub const SCRATCHPAD_L1: usize = RANDOMX_SCRATCHPAD_L1 / std::mem::size_of::<IntReg>();
pub const SCRATCHPAD_L2: usize = RANDOMX_SCRATCHPAD_L2 / std::mem::size_of::<IntReg>();
pub const SCRATCHPAD_L3: usize = RANDOMX_SCRATCHPAD_L3 / std::mem::size_of::<IntReg>();
pub const SCRATCHPAD_L1_MASK: usize = (SCRATCHPAD_L1 - 1) * 8;
pub const SCRATCHPAD_L2_MASK: usize = (SCRATCHPAD_L2 - 1) * 8;
pub const SCRATCHPAD_L1_MASK16: usize = (SCRATCHPAD_L1 / 2 - 1) * 16;
pub const SCRATCHPAD_L2_MASK16: usize = (SCRATCHPAD_L2 / 2 - 1) * 16;
pub const SCRATCHPAD_L3_MASK64: usize = (SCRATCHPAD_L3 - 1) * 8;
pub const SCRATCHPAD_L3_MASK: usize = SCRATCHPAD_L3_MASK64;
pub const SCRATCHPAD_L3_SIZE: usize = SCRATCHPAD_L3 * 8;

// Cache and Dataset constants
pub const CACHE_LINE_SIZE: usize = 64;
pub const ARGON_BLOCK_SIZE: usize = 1024;
pub const ARGON_MEMORY: usize = RANDOMX_ARGON_MEMORY;
pub const ARGON_MEMORY_B2K: usize = RANDOMX_ARGON_MEMORY * ARGON_BLOCK_SIZE / 1024;
pub const DATASET_EXTRA_ITEMS: usize = RANDOMX_DATASET_EXTRA_SIZE / RANDOMX_DATASET_ITEM_SIZE;

// Condition constants
pub const CONDITION_MASK: u32 = (1 << RANDOMX_JUMP_BITS) - 1;
pub const CONDITION_OFFSET: isize = RANDOMX_JUMP_OFFSET;
pub const STORE_L3_CONDITION: u32 = 14;

// Register count
pub const REGISTER_COUNT: usize = 8;