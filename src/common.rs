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

// Common definitions for RandomX implementation
use std::fmt;
use std::error::Error;

/// Size of a RandomX hash in bytes
pub const RANDOMX_HASH_SIZE: usize = 32;

/// Program size in instructions
pub const RANDOMX_PROGRAM_SIZE: usize = 256;

/// Program iterations
pub const RANDOMX_PROGRAM_ITERATIONS: usize = 2048;

/// Default scratchpad memory size
pub const RANDOMX_DEFAULT_SCRATCHPAD_SIZE: usize = 16777216;

/// Argon2 related constants
pub const ARGON_BLOCK_SIZE: usize = 1024;
pub const ARGON_MEMORY: usize = 262144;
pub const RANDOMX_ARGON_ITERATIONS: usize = 3;
pub const RANDOMX_ARGON_LANES: usize = 1;
pub const RANDOMX_ARGON_SALT: &[u8] = b"RandomX\x03";

/// Type alias for RandomX integer registers
pub type IntReg = u64;

/// Type alias for RandomX memory addresses
pub type Addr = u32;

/// FPU register value - two 64-bit parts
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FpuReg {
    pub lo: u64,
    pub hi: u64,
}

impl std::ops::AddAssign for FpuReg {
    fn add_assign(&mut self, other: Self) {
        self.lo = self.lo.wrapping_add(other.lo);
        self.hi = self.hi.wrapping_add(other.hi);
    }
}

impl std::ops::AddAssign<f64> for FpuReg {
    fn add_assign(&mut self, value: f64) {
        // Implementation detail: treat FpuReg as 128-bit binary float
        // This is a simplified implementation that adds a scalar float
        // For complete implementation, would need to consider 128-bit semantics
        let bytes = value.to_bits();
        self.lo = self.lo.wrapping_add(bytes);
    }
}

impl std::ops::SubAssign for FpuReg {
    fn sub_assign(&mut self, other: Self) {
        self.lo = self.lo.wrapping_sub(other.lo);
        self.hi = self.hi.wrapping_sub(other.hi);
    }
}

impl std::ops::SubAssign<f64> for FpuReg {
    fn sub_assign(&mut self, value: f64) {
        let bytes = value.to_bits();
        self.lo = self.lo.wrapping_sub(bytes);
    }
}

impl std::ops::MulAssign for FpuReg {
    fn mul_assign(&mut self, other: Self) {
        // Simplified multiplication - in a real implementation would need to
        // properly handle multiplication of 128-bit values
        self.lo = self.lo.wrapping_mul(other.lo);
        self.hi = self.hi.wrapping_mul(other.hi);
    }
}

impl std::ops::MulAssign<f64> for FpuReg {
    fn mul_assign(&mut self, value: f64) {
        let bytes = value.to_bits();
        self.lo = self.lo.wrapping_mul(bytes);
    }
}

impl std::ops::DivAssign for FpuReg {
    fn div_assign(&mut self, other: Self) {
        // Avoid division by zero
        if other.lo != 0 {
            self.lo = self.lo.wrapping_div(other.lo);
        }
        if other.hi != 0 {
            self.hi = self.hi.wrapping_div(other.hi);
        }
    }
}

impl std::ops::DivAssign<f64> for FpuReg {
    fn div_assign(&mut self, value: f64) {
        if value != 0.0 {
            let bytes = value.to_bits();
            if bytes != 0 {
                self.lo = self.lo.wrapping_div(bytes);
            }
        }
    }
}

impl PartialEq<f64> for FpuReg {
    fn eq(&self, other: &f64) -> bool {
        self.lo == other.to_bits() && self.hi == 0
    }
}

impl PartialOrd<f64> for FpuReg {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        // This is a simplified implementation that only compares the low bits
        // to a float. For a complete implementation, would need to consider 
        // 128-bit semantics.
        let other_bits = other.to_bits();
        match self.hi {
            0 => self.lo.partial_cmp(&other_bits),
            _ => {
                if self.hi & (1 << 63) != 0 {
                    // If high bit is set, treat as negative
                    Some(std::cmp::Ordering::Less)
                } else {
                    // Otherwise it's larger than any 64-bit float
                    Some(std::cmp::Ordering::Greater)
                }
            }
        }
    }
}

// Method to calculate square root, needed for FSQRT instruction
impl FpuReg {
    pub fn sqrt(&self) -> Self {
        // A simplified implementation that takes the sqrt of low bits
        // For a complete 128-bit implementation, would need more work
        let sqrt_lo = ((self.lo as f64).sqrt() as u64);
        FpuReg {
            lo: sqrt_lo,
            hi: 0,
        }
    }
}

/// Errors that can occur during RandomX operations
#[derive(Debug)]
pub enum RandomXError {
    /// Error during cache creation
    CacheCreationError,
    
    /// Error during dataset creation
    DatasetCreationError,
    
    /// Error during VM creation
    VmCreationError,
    
    /// Invalid operation attempted
    InvalidOperation(String),
    
    /// Invalid input data
    InvalidInput(String),
    
    /// Memory allocation or other system failure
    SystemError(String),
    
    /// Thread mutex poisoning error
    MutexPoisoning,
}

impl fmt::Display for RandomXError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandomXError::CacheCreationError => write!(f, "Failed to create RandomX cache"),
            RandomXError::DatasetCreationError => write!(f, "Failed to create RandomX dataset"),
            RandomXError::VmCreationError => write!(f, "Failed to create RandomX VM"),
            RandomXError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            RandomXError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            RandomXError::SystemError(msg) => write!(f, "System error: {}", msg),
            RandomXError::MutexPoisoning => write!(f, "Thread mutex poisoning error"),
        }
    }
}

impl Error for RandomXError {}

/// Result type for RandomX operations
pub type Result<T> = std::result::Result<T, RandomXError>;