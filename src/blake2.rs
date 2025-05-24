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
use std::mem::MaybeUninit;
use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
use crate::instructions::{Instruction, InstructionType};

// FFI declarations for the C++ Blake2b implementation
#[repr(C)]
struct Blake2bState {
    h: [u64; 8],
    t: [u64; 2],
    f: [u64; 2],
    buf: [u8; 256],
    buflen: usize,
    outlen: usize,
    last_node: u8,
}

extern "C" {
    fn blake2b_init(state: *mut Blake2bState, outlen: usize) -> i32;
    fn blake2b_update(state: *mut Blake2bState, input: *const c_void, inlen: usize) -> i32;
    fn blake2b_final(state: *mut Blake2bState, output: *mut c_void, outlen: usize) -> i32;
    fn blake2b(output: *mut c_void, outlen: usize, input: *const c_void, inlen: usize, key: *const c_void, keylen: usize) -> i32;
}

/// Computes a Blake2b hash using the pure Rust implementation
pub fn hash(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut hasher = Blake2b::new(output_len);
    hasher.update(input);
    let result = hasher.finalize();
    result.as_bytes().to_vec()
}

/// Computes a Blake2b hash with a key using the pure Rust implementation
pub fn keyed_hash(input: &[u8], key: &[u8], output_len: usize) -> Vec<u8> {
    let mut hasher = Blake2b::with_key(output_len, key);
    hasher.update(input);
    let result = hasher.finalize();
    result.as_bytes().to_vec()
}

/// Low-level interface that uses the C implementation for better compatibility
pub mod c_compat {
    use super::*;

    /// Initializes a Blake2b state for incremental hashing
    pub fn init(output_len: usize) -> Option<Box<Blake2bState>> {
        let mut state = Box::new(unsafe { MaybeUninit::<Blake2bState>::zeroed().assume_init() });
        let ret = unsafe { blake2b_init(&mut *state, output_len) };
        if ret == 0 {
            Some(state)
        } else {
            None
        }
    }

    /// Updates a Blake2b state with input data
    pub fn update(state: &mut Blake2bState, input: &[u8]) -> bool {
        let ret = unsafe {
            blake2b_update(
                state,
                input.as_ptr() as *const c_void,
                input.len(),
            )
        };
        ret == 0
    }

    /// Finalizes a Blake2b hash and writes the result to the output buffer
    pub fn finalize(state: &mut Blake2bState, output: &mut [u8]) -> bool {
        let ret = unsafe {
            blake2b_final(
                state,
                output.as_mut_ptr() as *mut c_void,
                output.len(),
            )
        };
        ret == 0
    }

    /// Computes a Blake2b hash in a single call
    pub fn hash(input: &[u8], output: &mut [u8]) -> bool {
        let ret = unsafe {
            blake2b(
                output.as_mut_ptr() as *mut c_void,
                output.len(),
                input.as_ptr() as *const c_void,
                input.len(),
                std::ptr::null(),
                0,
            )
        };
        ret == 0
    }

    /// Computes a Blake2b hash with a key in a single call
    pub fn keyed_hash(input: &[u8], key: &[u8], output: &mut [u8]) -> bool {
        let ret = unsafe {
            blake2b(
                output.as_mut_ptr() as *mut c_void,
                output.len(),
                input.as_ptr() as *const c_void,
                input.len(),
                key.as_ptr() as *const c_void,
                key.len(),
            )
        };
        ret == 0
    }
}

/// Generates a RandomX program from an input using Blake2b
pub fn generate_program(input: &[u8], program: &mut [Instruction], config: &mut [u8]) {
    // Generate raw bytes using Blake2b
    let mut hasher = Blake2b::new(program.len() * 8 + config.len());
    hasher.update(input);
    let result = hasher.finalize();
    let bytes = result.as_bytes();
    
    // Fill config with bytes from the hash
    let config_start = 0;
    let config_end = config.len();
    config.copy_from_slice(&bytes[config_start..config_end]);
    
    // Use the remaining bytes to generate instructions
    for i in 0..program.len() {
        let offset = config.len() + i * 8;
        if offset + 8 > bytes.len() {
            // If we run out of bytes, use a secondary hash
            let mut secondary_hasher = Blake2b::new(64);
            secondary_hasher.update(&bytes);
            secondary_hasher.update(&[i as u8]);
            let secondary_result = secondary_hasher.finalize();
            let secondary_bytes = secondary_result.as_bytes();
            
            // Extract instruction components from the secondary hash
            let opcode_byte = secondary_bytes[0] % 30;
            let dst = secondary_bytes[1] & 0x7;
            let src = secondary_bytes[2] & 0x7;
            let mod_byte = secondary_bytes[3];
            let imm = i32::from_le_bytes([
                secondary_bytes[4], 
                secondary_bytes[5], 
                secondary_bytes[6], 
                secondary_bytes[7]
            ]);
            
            // Create instruction
            program[i] = Instruction::new(
                unsafe { std::mem::transmute(opcode_byte) },
                dst,
                src,
                mod_byte,
                imm
            );
        } else {
            // Extract instruction components from the primary hash
            let opcode_byte = bytes[offset] % 30;
            let dst = bytes[offset + 1] & 0x7;
            let src = bytes[offset + 2] & 0x7;
            let mod_byte = bytes[offset + 3];
            let imm = i32::from_le_bytes([
                bytes[offset + 4], 
                bytes[offset + 5], 
                bytes[offset + 6], 
                bytes[offset + 7]
            ]);
            
            // Create instruction
            program[i] = Instruction::new(
                unsafe { std::mem::transmute(opcode_byte) },
                dst,
                src,
                mod_byte,
                imm
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2b() {
        // Test vector from the Blake2b spec
        let input = b"abc";
        let expected = hex::decode("ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923").unwrap();
        
        // Test the Rust implementation
        let hash_result = hash(input, 64);
        assert_eq!(hash_result, expected);
        
        // Test the C-compatible implementation
        let mut output = vec![0; 64];
        assert!(c_compat::hash(input, &mut output));
        assert_eq!(output, expected);
    }
}