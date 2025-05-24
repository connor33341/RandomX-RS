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
use std::mem;
use std::ptr;
use std::slice;
use crate::RANDOMX_DATASET_ITEM_SIZE;

/// RandomX cache structure
pub struct Cache {
    inner: *mut c_void,
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

impl Cache {
    /// Initialize cache with a key
    pub fn new(flags: super::RandomXFlags, key: &[u8]) -> Option<Self> {
        extern "C" {
            fn randomx_alloc_cache(flags: u32) -> *mut c_void;
            fn randomx_init_cache(cache: *mut c_void, key: *const c_void, keySize: usize);
        }
        
        let ptr = unsafe { randomx_alloc_cache(flags.bits()) };
        if ptr.is_null() {
            return None;
        }
        
        unsafe {
            randomx_init_cache(ptr, key.as_ptr() as *const c_void, key.len());
        }
        
        Some(Self { inner: ptr })
    }

    /// Create a Cache from a raw pointer
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self { inner: ptr }
    }
    
    /// Get the raw pointer to the cache
    pub fn as_ptr(&self) -> *const c_void {
        self.inner as *const c_void
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        extern "C" {
            fn randomx_release_cache(cache: *mut c_void);
        }
        
        if !self.inner.is_null() {
            unsafe {
                randomx_release_cache(self.inner);
            }
            self.inner = ptr::null_mut();
        }
    }
}

/// RandomX dataset structure
pub struct Dataset {
    inner: *mut c_void,
}

unsafe impl Send for Dataset {}
unsafe impl Sync for Dataset {}

impl Dataset {
    /// Create a new dataset from a cache
    pub fn new(flags: super::RandomXFlags, cache: &Cache) -> Option<Self> {
        extern "C" {
            fn randomx_alloc_dataset(flags: u32) -> *mut c_void;
            fn randomx_init_dataset(dataset: *mut c_void, cache: *mut c_void, startItem: u64, itemCount: u64);
            fn randomx_dataset_item_count() -> u64;
        }

        let ptr = unsafe { randomx_alloc_dataset(flags.bits()) };
        if ptr.is_null() {
            return None;
        }

        unsafe {
            let item_count = randomx_dataset_item_count();
            randomx_init_dataset(ptr, cache.inner, 0, item_count);
        }

        Some(Self { inner: ptr })
    }
    
    /// Create a dataset from a raw pointer
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self { inner: ptr }
    }
    
    /// Get the raw pointer to the dataset
    pub fn as_ptr(&self) -> *const c_void {
        self.inner as *const c_void
    }
    
    /// Gets a reference to the dataset's memory buffer
    pub fn memory(&self) -> &[u8] {
        unsafe {
            let ptr = randomx_get_dataset_memory(self.inner);
            let count = crate::dataset_item_count() as usize;
            let size = count * RANDOMX_DATASET_ITEM_SIZE;
            slice::from_raw_parts(ptr as *const u8, size)
        }
    }
    
    /// Gets a mutable reference to the dataset's memory buffer
    pub fn memory_mut(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = randomx_get_dataset_memory(self.inner);
            let count = crate::dataset_item_count() as usize;
            let size = count * RANDOMX_DATASET_ITEM_SIZE;
            slice::from_raw_parts_mut(ptr as *mut u8, size)
        }
    }
}

impl Drop for Dataset {
    fn drop(&mut self) {
        extern "C" {
            fn randomx_release_dataset(dataset: *mut c_void);
        }
        
        if !self.inner.is_null() {
            unsafe {
                randomx_release_dataset(self.inner);
            }
            self.inner = ptr::null_mut();
        }
    }
}