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
use std::ptr;
use crate::{RandomXFlags};
use crate::cpu::CPU;

// Platform-specific JIT implementations
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "riscv64")]
mod riscv64;

/// JIT compiler for RandomX
pub struct JitCompiler {
    ptr: *mut c_void,
}

impl JitCompiler {
    /// Create a new JIT compiler
    pub fn new(flags: RandomXFlags, large_pages: bool, numa: bool) -> Option<Self> {
        let ptr = match () {
            #[cfg(target_arch = "x86_64")]
            () => x86_64::create_jit_compiler(flags, large_pages, numa),
            
            #[cfg(target_arch = "aarch64")]
            () => Self::create_aarch64_jit(flags, large_pages, numa),
            
            #[cfg(target_arch = "riscv64")]
            () => Self::create_riscv64_jit(flags, large_pages, numa),
            
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
            () => None,
        };
        
        ptr.map(|p| Self { ptr: p })
    }
    
    /// Get the raw pointer to the JIT compiler
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr
    }
    
    /// Get the raw mutable pointer to the JIT compiler
    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.ptr
    }
    
    // Platform-specific JIT creation functions
    
    #[cfg(target_arch = "aarch64")]
    fn create_aarch64_jit(flags: RandomXFlags, large_pages: bool, numa: bool) -> Option<*mut c_void> {
        extern "C" {
            fn randomx_alloc_jit_compiler_a64(flags: u32, largePages: bool, jitNuma: bool) -> *mut c_void;
        }
        
        let ptr = unsafe {
            randomx_alloc_jit_compiler_a64(flags.bits(), large_pages, numa)
        };
        
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    fn create_riscv64_jit(flags: RandomXFlags, large_pages: bool, numa: bool) -> Option<*mut c_void> {
        extern "C" {
            fn randomx_alloc_jit_compiler_rv64(flags: u32, largePages: bool, jitNuma: bool) -> *mut c_void;
        }
        
        let ptr = unsafe {
            randomx_alloc_jit_compiler_rv64(flags.bits(), large_pages, numa)
        };
        
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
    
    /// Create a VM using this JIT compiler
    pub fn create_vm(&self, cache: *const c_void, dataset: *const c_void) -> Option<*mut c_void> {
        match () {
            #[cfg(target_arch = "x86_64")]
            () => x86_64::create_vm_with_compiler(self.ptr, cache as *mut c_void, dataset as *mut c_void),
            
            #[cfg(target_arch = "aarch64")]
            () => self.create_vm_aarch64(cache, dataset),
            
            #[cfg(target_arch = "riscv64")]
            () => self.create_vm_riscv64(cache, dataset),
            
            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
            () => None,
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    fn create_vm_aarch64(&self, cache: *const c_void, dataset: *const c_void) -> Option<*mut c_void> {
        extern "C" {
            fn randomx_create_vm_a64_compiler(jit: *mut c_void, cache: *mut c_void, dataset: *mut c_void) -> *mut c_void;
        }
        
        let ptr = unsafe {
            randomx_create_vm_a64_compiler(self.ptr, cache as *mut c_void, dataset as *mut c_void)
        };
        
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    fn create_vm_riscv64(&self, cache: *const c_void, dataset: *const c_void) -> Option<*mut c_void> {
        extern "C" {
            fn randomx_create_vm_rv64_compiler(jit: *mut c_void, cache: *mut c_void, dataset: *mut c_void) -> *mut c_void;
        }
        
        let ptr = unsafe {
            randomx_create_vm_rv64_compiler(self.ptr, cache as *mut c_void, dataset as *mut c_void)
        };
        
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
}

impl Drop for JitCompiler {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                match () {
                    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64"))]
                    () => x86_64::free_jit_compiler(self.ptr),
                    
                    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
                    () => {},
                }
            }
        }
    }
}