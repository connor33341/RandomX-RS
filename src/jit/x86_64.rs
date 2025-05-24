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

extern "C" {
    fn randomx_alloc_jit_compiler_x86(flags: u32, largePages: bool, jitNuma: bool) -> *mut c_void;
    fn randomx_free_jit_compiler(jit: *mut c_void);
    fn randomx_create_vm_x86_compiler(jit: *mut c_void, cache: *mut c_void, dataset: *mut c_void) -> *mut c_void;
}

/// Create a JIT compiler for x86_64 architecture
pub fn create_jit_compiler(flags: RandomXFlags, large_pages: bool, numa: bool) -> Option<*mut c_void> {
    let ptr = unsafe {
        randomx_alloc_jit_compiler_x86(flags.bits(), large_pages, numa)
    };
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

/// Free a JIT compiler
pub fn free_jit_compiler(jit: *mut c_void) {
    unsafe {
        randomx_free_jit_compiler(jit);
    }
}

/// Create a VM with JIT compiler
pub fn create_vm_with_compiler(jit: *mut c_void, cache: *mut c_void, dataset: *mut c_void) -> Option<*mut c_void> {
    let ptr = unsafe {
        randomx_create_vm_x86_compiler(jit, cache, dataset)
    };
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}