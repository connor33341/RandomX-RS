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

use std::ops::{Add, Sub, Mul, Div, BitAnd, BitOr, BitXor, Not};
use std::convert::From;

/// Constants for RandomX algorithm
pub const RANDOMX_HASH_SIZE: usize = 32;
pub const RANDOMX_DATASET_ITEM_SIZE: usize = 64;
pub const RANDOMX_PROGRAM_SIZE: usize = 256;
pub const RANDOMX_PROGRAM_ITERATIONS: usize = 2048;
pub const RANDOMX_SCRATCHPAD_L3: usize = 2097152;
pub const RANDOMX_SCRATCHPAD_L2: usize = 262144;
pub const RANDOMX_SCRATCHPAD_L1: usize = 16384;

/// VM register count constants
pub const REGISTERFILE_SIZE: usize = 8;
pub const REGISTER_COUNT: usize = 8;

/// Representation of a RandomX integer register (r0-r7)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntReg([u64; 2]);

impl IntReg {
    /// Create a new integer register with specified values
    pub fn new(lo: u64, hi: u64) -> Self {
        IntReg([lo, hi])
    }
    
    /// Create a zero-initialized integer register
    pub fn zero() -> Self {
        IntReg([0, 0])
    }
    
    /// Get the low 64 bits
    pub fn lo(&self) -> u64 {
        self.0[0]
    }
    
    /// Get the high 64 bits
    pub fn hi(&self) -> u64 {
        self.0[1]
    }
    
    /// Set the low 64 bits
    pub fn set_lo(&mut self, value: u64) {
        self.0[0] = value;
    }
    
    /// Set the high 64 bits 
    pub fn set_hi(&mut self, value: u64) {
        self.0[1] = value;
    }
}

/// Representation of a RandomX floating-point register (f0-f7)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FpuReg([f64; 2]);

impl FpuReg {
    /// Create a new FPU register with specified values
    pub fn new(lo: f64, hi: f64) -> Self {
        FpuReg([lo, hi])
    }
    
    /// Create a zero-initialized FPU register
    pub fn zero() -> Self {
        FpuReg([0.0, 0.0])
    }
    
    /// Get the low 64 bits as f64
    pub fn lo(&self) -> f64 {
        self.0[0]
    }
    
    /// Get the high 64 bits as f64
    pub fn hi(&self) -> f64 {
        self.0[1]
    }
    
    /// Set the low 64 bits
    pub fn set_lo(&mut self, value: f64) {
        self.0[0] = value;
    }
    
    /// Set the high 64 bits
    pub fn set_hi(&mut self, value: f64) {
        self.0[1] = value;
    }
}

/// Representation of a memory address in RandomX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Addr(pub u64);

impl Addr {
    /// Create a new address with the given value
    pub fn new(value: u64) -> Self {
        Addr(value)
    }
    
    /// Get the raw address value
    pub fn value(&self) -> u64 {
        self.0
    }
    
    /// Apply the scratchpad address mask
    pub fn apply_mask(&self, mask: u64) -> Self {
        Addr(self.0 & mask)
    }
}

impl From<u64> for Addr {
    fn from(value: u64) -> Self {
        Addr(value)
    }
}

impl From<Addr> for u64 {
    fn from(addr: Addr) -> Self {
        addr.0
    }
}

impl Add for Addr {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Addr(self.0.wrapping_add(other.0))
    }
}

impl Sub for Addr {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Addr(self.0.wrapping_sub(other.0))
    }
}

/// Error types for RandomX operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandomXError {
    AllocationError,
    InvalidKey,
    InvalidInput,
    InternalError,
}

/// Result type for RandomX operations
pub type Result<T> = std::result::Result<T, RandomXError>;