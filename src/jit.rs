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

use cfg_if::cfg_if;
use std::sync::Arc;

use crate::common::RANDOMX_PROGRAM_SIZE;

// Define constants based on architecture
#[cfg(all(target_arch = "x86_64", not(target_os = "macos")))]
pub const HAVE_COMPILER: bool = true;

#[cfg(all(target_arch = "aarch64", not(target_os = "macos")))]
pub const HAVE_COMPILER: bool = true;

#[cfg(all(target_arch = "riscv64", not(target_os = "macos")))]
pub const HAVE_COMPILER: bool = true;

#[cfg(not(any(
    all(target_arch = "x86_64", not(target_os = "macos")),
    all(target_arch = "aarch64", not(target_os = "macos")),
    all(target_arch = "riscv64", not(target_os = "macos"))
)))]
pub const HAVE_COMPILER: bool = false;

// Platform-specific JIT compiler implementations
cfg_if! {
    if #[cfg(all(target_arch = "x86_64", not(target_os = "macos")))] {
        mod x86_64;
        pub use x86_64::JitCompilerImpl;
    } else if #[cfg(all(target_arch = "aarch64", not(target_os = "macos")))] {
        mod aarch64;
        pub use aarch64::JitCompilerImpl;
    } else if #[cfg(all(target_arch = "riscv64", not(target_os = "macos")))] {
        mod riscv64;
        pub use riscv64::JitCompilerImpl;
    } else {
        mod fallback;
        pub use fallback::JitCompilerImpl;
    }
}

/// Code buffer used by the JIT compiler
pub struct CodeBuffer {
    code: Box<[u8]>,
    code_pos: usize,
    rcp_count: i32,
}

impl CodeBuffer {
    /// Creates a new code buffer with the specified size
    pub fn new(size: usize) -> Self {
        Self {
            code: vec![0; size].into_boxed_slice(),
            code_pos: 0,
            rcp_count: 0,
        }
    }

    /// Emits bytes to the code buffer
    pub fn emit(&mut self, src: &[u8]) {
        let end = self.code_pos + src.len();
        if end <= self.code.len() {
            self.code[self.code_pos..end].copy_from_slice(src);
            self.code_pos = end;
        } else {
            panic!("Code buffer overflow");
        }
    }

    /// Emits a single value to the code buffer
    pub fn emit_value<T: Copy>(&mut self, src: T) {
        let size = std::mem::size_of::<T>();
        let end = self.code_pos + size;
        if end <= self.code.len() {
            // Safety: We're ensuring proper bounds and T is Copy (plain data)
            unsafe {
                let src_ptr = &src as *const T as *const u8;
                let dst_ptr = self.code.as_mut_ptr().add(self.code_pos);
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, size);
            }
            self.code_pos = end;
        } else {
            panic!("Code buffer overflow");
        }
    }

    /// Emits bytes at a specific position in the code buffer
    pub fn emit_at(&mut self, pos: usize, src: &[u8]) {
        let end = pos + src.len();
        if end <= self.code.len() {
            self.code[pos..end].copy_from_slice(src);
        } else {
            panic!("Code buffer overflow");
        }
    }

    /// Emits a single value at a specific position in the code buffer
    pub fn emit_value_at<T: Copy>(&mut self, pos: usize, src: T) {
        let size = std::mem::size_of::<T>();
        let end = pos + size;
        if end <= self.code.len() {
            // Safety: We're ensuring proper bounds and T is Copy (plain data)
            unsafe {
                let src_ptr = &src as *const T as *const u8;
                let dst_ptr = self.code.as_mut_ptr().add(pos);
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, size);
            }
        } else {
            panic!("Code buffer overflow");
        }
    }

    /// Gets the current position in the code buffer
    pub fn position(&self) -> usize {
        self.code_pos
    }

    /// Gets a reference to the code buffer
    pub fn code(&self) -> &[u8] {
        &self.code[..self.code_pos]
    }

    /// Gets a mutable reference to the code buffer
    pub fn code_mut(&mut self) -> &mut [u8] {
        &mut self.code[..self.code_pos]
    }
}

/// Compiler state used during JIT compilation
pub struct CompilerState {
    pub code_buffer: CodeBuffer,
    pub instruction_offsets: [i32; RANDOMX_PROGRAM_SIZE],
    pub register_usage: [i32; 8], // 8 registers in RandomX
}

impl CompilerState {
    /// Creates a new compiler state with the specified code buffer size
    pub fn new(code_size: usize) -> Self {
        Self {
            code_buffer: CodeBuffer::new(code_size),
            instruction_offsets: [0; RANDOMX_PROGRAM_SIZE],
            register_usage: [0; 8],
        }
    }
}

/// JIT compiler trait that all implementations must support
pub trait JitCompiler {
    /// Create a new JIT compiler instance
    fn new() -> Self where Self: Sized;
    
    /// Generate compiled program
    fn generate_program(&self, program_data: &[u8], configuration: &[u8]);
    
    /// Get a pointer to the compiled program function
    fn get_program_func(&self) -> unsafe extern "C" fn();
    
    /// Generate code for the superscalar hash function
    fn generate_superscalar_hash(&self, program: &[u8], reciprocal_cache: &[u8]);
    
    /// Enable writing to the code buffer
    fn enable_writing(&self);
    
    /// Enable execution of the code buffer
    fn enable_execution(&self);
    
    /// Enable both writing and execution of the code buffer
    fn enable_all(&self);
}

// For now, we'll use a placeholder implementation that simply forwards
// calls to the native implementation via FFI

/// Primary JIT compiler implementation
/// This is either a native Rust implementation or a bridge to the C/C++ code
#[derive(Clone)]
pub struct JitCompilerX86 {
    _private: (),
}

/// Fallback JIT compiler implementation when no native JIT is available
#[derive(Clone)]
pub struct JitCompilerFallback {
    _private: (),
}

// Type alias for the appropriate JIT compiler implementation
cfg_if! {
    if #[cfg(all(target_arch = "x86_64", not(target_os = "macos")))] {
        pub type DefaultJitCompiler = JitCompilerX86;
    } else {
        pub type DefaultJitCompiler = JitCompilerFallback;
    }
}

#[cfg(not(all(target_arch = "x86_64", not(target_os = "macos"))))]
mod fallback {
    use super::*;
    
    pub struct JitCompilerImpl {}
    
    impl JitCompiler for JitCompilerImpl {
        fn new() -> Self {
            Self {}
        }
        
        fn generate_program(&self, _program_data: &[u8], _configuration: &[u8]) {
            // No-op in fallback mode
        }
        
        fn get_program_func(&self) -> unsafe extern "C" fn() {
            unsafe extern "C" fn dummy() {
                // This should never be called directly
                panic!("Attempted to call JIT code in fallback mode");
            }
            dummy
        }
        
        fn generate_superscalar_hash(&self, _program: &[u8], _reciprocal_cache: &[u8]) {
            // No-op in fallback mode
        }
        
        fn enable_writing(&self) {
            // No-op in fallback mode
        }
        
        fn enable_execution(&self) {
            // No-op in fallback mode
        }
        
        fn enable_all(&self) {
            // No-op in fallback mode
        }
    }
}