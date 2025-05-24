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
use crate::{Cache, Dataset, RandomXFlags, RANDOMX_HASH_SIZE};
use crate::vm_interpreted::InterpretedVirtualMachine;

/// Abstract VM interface
pub trait VirtualMachine {
    /// Initialize the VM with a dataset
    fn initialize_dataset(&mut self, dataset: &Dataset) -> bool;
    
    /// Initialize the VM with a cache
    fn initialize_cache(&mut self, cache: &Cache) -> bool;
    
    /// Set the current randomization info
    fn set_randomization_info(&mut self, info: &[u8]) -> bool;
    
    /// Calculate the hash of the input data
    fn calculate(&mut self, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE];
    
    /// Calculate the hash with another randomization info
    fn calculate_with_info(&mut self, input: &[u8], info: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
        self.set_randomization_info(info);
        self.calculate(input)
    }
}

/// C VM instance wrapper
pub struct RandomXVM {
    inner: *mut c_void,
}

unsafe impl Send for RandomXVM {}
unsafe impl Sync for RandomXVM {}

impl RandomXVM {
    /// Create a new VM with the given flags, cache, and optionally dataset
    pub fn new(flags: RandomXFlags, cache: &Cache, dataset: Option<&Dataset>) -> Option<Self> {
        extern "C" {
            fn randomx_create_vm(flags: u32, cache: *mut c_void, dataset: *mut c_void) -> *mut c_void;
        }
        
        let dataset_ptr = match dataset {
            Some(ds) => ds.as_ptr() as *mut c_void,
            None => ptr::null_mut(),
        };
        
        let ptr = unsafe {
            randomx_create_vm(flags.bits(), cache.as_ptr() as *mut c_void, dataset_ptr)
        };
        
        if ptr.is_null() {
            return None;
        }
        
        Some(Self { inner: ptr })
    }
    
    /// Create a VM from a raw pointer
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self { inner: ptr }
    }
}

impl VirtualMachine for RandomXVM {
    fn initialize_dataset(&mut self, dataset: &Dataset) -> bool {
        extern "C" {
            fn randomx_vm_set_dataset(machine: *mut c_void, dataset: *const c_void) -> bool;
        }
        
        unsafe {
            randomx_vm_set_dataset(self.inner, dataset.as_ptr())
        }
    }
    
    fn initialize_cache(&mut self, cache: &Cache) -> bool {
        extern "C" {
            fn randomx_vm_set_cache(machine: *mut c_void, cache: *const c_void) -> bool;
        }
        
        unsafe {
            randomx_vm_set_cache(self.inner, cache.as_ptr())
        }
    }
    
    fn set_randomization_info(&mut self, info: &[u8]) -> bool {
        extern "C" {
            fn randomx_vm_set_ext_arg(machine: *mut c_void, info: *const u8, infoSize: usize) -> bool;
        }
        
        unsafe {
            randomx_vm_set_ext_arg(self.inner, info.as_ptr(), info.len())
        }
    }
    
    fn calculate(&mut self, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
        extern "C" {
            fn randomx_calculate_hash(machine: *mut c_void, input: *const c_void, 
                                      inputSize: usize, output: *mut u8);
        }
        
        let mut hash = [0u8; RANDOMX_HASH_SIZE];
        
        unsafe {
            randomx_calculate_hash(
                self.inner, 
                input.as_ptr() as *const c_void,
                input.len(),
                hash.as_mut_ptr()
            );
        }
        
        hash
    }
}

impl Drop for RandomXVM {
    fn drop(&mut self) {
        extern "C" {
            fn randomx_destroy_vm(machine: *mut c_void);
        }
        
        if !self.inner.is_null() {
            unsafe {
                randomx_destroy_vm(self.inner);
            }
            self.inner = ptr::null_mut();
        }
    }
}