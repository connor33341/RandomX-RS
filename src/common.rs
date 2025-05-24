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
        }
    }
}

impl Error for RandomXError {}

/// Result type for RandomX operations
pub type Result<T> = std::result::Result<T, RandomXError>;