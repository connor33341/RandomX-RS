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

use std::ffi::c_void;
use crate::RandomXFlags;
use crate::common::{ARGON_BLOCK_SIZE, ARGON_MEMORY, RANDOMX_ARGON_ITERATIONS, RANDOMX_ARGON_LANES, RANDOMX_ARGON_SALT};

// Constants from the Argon2 algorithm
pub const ARGON2_BLOCK_SIZE: usize = 1024;
pub const ARGON2_QWORDS_IN_BLOCK: usize = ARGON2_BLOCK_SIZE / 8;

pub const ARGON2_ADDRESSES_IN_BLOCK: usize = 128;
pub const ARGON2_SYNC_POINTS: u32 = 4;

// Argon2 error codes
#[derive(Debug, PartialEq)]
pub enum Argon2Error {
    Ok = 0,
    OutputPtrNull = -1,
    OutputTooShort = -2,
    OutputTooLong = -3,
    PwdTooShort = -4,
    PwdTooLong = -5,
    SaltTooShort = -6,
    SaltTooLong = -7,
    AdTooShort = -8,
    AdTooLong = -9,
    SecretTooShort = -10,
    SecretTooLong = -11,
    TimeTooSmall = -12,
    TimeTooLarge = -13,
    MemoryTooLittle = -14,
    MemoryTooMuch = -15,
    LanesTooFew = -16,
    LanesTooMany = -17,
    PwdPtrMismatch = -18,
    SaltPtrMismatch = -19,
    SecretPtrMismatch = -20,
    AdPtrMismatch = -21,
    MemoryAllocationError = -22,
    FreeMemoryCbkNull = -23,
    AllocateMemoryCbkNull = -24,
    IncorrectParameter = -25,
    IncorrectType = -26,
    OutPtrMismatch = -27,
    ThreadsTooFew = -28,
    ThreadsTooMany = -29,
    MissingArgs = -30,
    EncodingFail = -31,
    DecodingFail = -32,
    ThreadFail = -33,
    DecodingLengthFail = -34,
    VerifyMismatch = -35,
}

// Argon2 type variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Argon2Type {
    Argon2D = 0,
    Argon2I = 1,
    Argon2Id = 2,
}

// Version of the algorithm
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Argon2Version {
    Version10 = 0x10,
    Version13 = 0x13,
}

// Flag bits - modify behavior of the algorithm
pub const ARGON2_FLAG_CLEAR_PASSWORD: u32 = 1 << 0;
pub const ARGON2_FLAG_CLEAR_SECRET: u32 = 1 << 1;
pub const ARGON2_DEFAULT_FLAGS: u32 = 0;

// FFI declarations for the C implementation

// Basic hash function - uses internal memory allocation
extern "C" {
    fn argon2_hash(
        t_cost: u32,           // Number of iterations
        m_cost: u32,           // Memory usage in kibibytes
        parallelism: u32,      // Number of threads and lanes
        pwd: *const c_void,    // Password
        pwdlen: usize,         // Password length
        salt: *const c_void,   // Salt
        saltlen: usize,        // Salt length
        hash: *mut c_void,     // Output hash
        hashlen: usize,        // Output hash length
        encoded: *mut u8,      // Encoded hash (unused in RandomX)
        encodedlen: usize,     // Encoded hash length
        type_: i32,            // Argon2 variant
        version: u32,          // Version number
    ) -> i32;

    // Implementation of fill_segment for various optimizations
    fn randomx_argon2_fill_segment_ref(
        instance: *const c_void, 
        position: *const c_void
    );

    fn randomx_argon2_fill_segment_ssse3(
        instance: *const c_void,
        position: *const c_void
    );

    fn randomx_argon2_fill_segment_avx2(
        instance: *const c_void,
        position: *const c_void
    );

    // Get implementation pointer based on flags
    fn randomx_argon2_impl_ssse3() -> Option<unsafe extern "C" fn(*const c_void, *const c_void)>;
    fn randomx_argon2_impl_avx2() -> Option<unsafe extern "C" fn(*const c_void, *const c_void)>;
}

/// Provides an implementation of the Argon2 hash function
pub struct Argon2;

impl Argon2 {
    /// Hash a password with Argon2
    /// 
    /// # Arguments
    /// 
    /// * `password` - The password to hash
    /// * `salt` - The salt to use
    /// * `parallelism` - Number of threads and lanes
    /// * `t_cost` - Number of iterations
    /// * `m_cost` - Memory usage in kibibytes
    /// * `hash_len` - Output hash length
    /// * `argon2_type` - The Argon2 variant to use
    /// * `version` - Version of the algorithm to use
    /// 
    /// # Returns
    /// 
    /// The computed hash or an error
    pub fn hash(
        password: &[u8],
        salt: &[u8],
        parallelism: u32,
        t_cost: u32,
        m_cost: u32,
        hash_len: usize,
        argon2_type: Argon2Type,
        version: Argon2Version,
    ) -> Result<Vec<u8>, Argon2Error> {
        let mut hash = vec![0u8; hash_len];
        
        // We don't need encoded output for RandomX
        let encoded: *mut u8 = std::ptr::null_mut();
        let encoded_len: usize = 0;
        
        let result = unsafe {
            argon2_hash(
                t_cost,
                m_cost,
                parallelism,
                password.as_ptr() as *const c_void,
                password.len(),
                salt.as_ptr() as *const c_void,
                salt.len(),
                hash.as_mut_ptr() as *mut c_void,
                hash_len,
                encoded,
                encoded_len,
                argon2_type as i32,
                version as u32,
            )
        };
        
        if result == 0 {
            Ok(hash)
        } else {
            Err(Self::convert_error_code(result))
        }
    }
    
    /// Get the appropriate implementation of Argon2 based on flags
    pub fn select_impl(flags: RandomXFlags) -> Option<unsafe extern "C" fn(*const c_void, *const c_void)> {
        unsafe {
            if flags.contains(RandomXFlags::ARGON2_AVX2) {
                randomx_argon2_impl_avx2()
            } else if flags.contains(RandomXFlags::ARGON2_SSSE3) {
                randomx_argon2_impl_ssse3()
            } else {
                Some(randomx_argon2_fill_segment_ref)
            }
        }
    }
    
    /// Convert a C error code to a Rust enum
    fn convert_error_code(code: i32) -> Argon2Error {
        match code {
            0 => Argon2Error::Ok,
            -1 => Argon2Error::OutputPtrNull,
            -2 => Argon2Error::OutputTooShort,
            -3 => Argon2Error::OutputTooLong,
            -4 => Argon2Error::PwdTooShort,
            -5 => Argon2Error::PwdTooLong,
            -6 => Argon2Error::SaltTooShort,
            -7 => Argon2Error::SaltTooLong,
            -8 => Argon2Error::AdTooShort,
            -9 => Argon2Error::AdTooLong,
            -10 => Argon2Error::SecretTooShort,
            -11 => Argon2Error::SecretTooLong,
            -12 => Argon2Error::TimeTooSmall,
            -13 => Argon2Error::TimeTooLarge,
            -14 => Argon2Error::MemoryTooLittle,
            -15 => Argon2Error::MemoryTooMuch,
            -16 => Argon2Error::LanesTooFew,
            -17 => Argon2Error::LanesTooMany,
            -18 => Argon2Error::PwdPtrMismatch,
            -19 => Argon2Error::SaltPtrMismatch,
            -20 => Argon2Error::SecretPtrMismatch,
            -21 => Argon2Error::AdPtrMismatch,
            -22 => Argon2Error::MemoryAllocationError,
            -23 => Argon2Error::FreeMemoryCbkNull,
            -24 => Argon2Error::AllocateMemoryCbkNull,
            -25 => Argon2Error::IncorrectParameter,
            -26 => Argon2Error::IncorrectType,
            -27 => Argon2Error::OutPtrMismatch,
            -28 => Argon2Error::ThreadsTooFew,
            -29 => Argon2Error::ThreadsTooMany,
            -30 => Argon2Error::MissingArgs,
            -31 => Argon2Error::EncodingFail,
            -32 => Argon2Error::DecodingFail,
            -33 => Argon2Error::ThreadFail,
            -34 => Argon2Error::DecodingLengthFail,
            -35 => Argon2Error::VerifyMismatch,
            _ => Argon2Error::IncorrectParameter,
        }
    }
}

/// Helper function to create an Argon2d hash with RandomX parameters
pub fn argon2d_randomx(input: &[u8]) -> Result<Vec<u8>, Argon2Error> {
    Argon2::hash(
        input,
        RANDOMX_ARGON_SALT,
        RANDOMX_ARGON_LANES as u32,
        RANDOMX_ARGON_ITERATIONS as u32,
        ARGON_MEMORY as u32,
        ARGON_BLOCK_SIZE,
        Argon2Type::Argon2D,
        Argon2Version::Version13,
    )
}