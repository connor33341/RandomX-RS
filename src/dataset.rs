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
use std::sync::{Arc, Mutex};
use std::thread;

use crate::RandomXFlags;

/// RandomX cache structure used for light verification mode
pub struct Cache {
    inner: *mut c_void,
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

impl Cache {
    /// Create a new cache with the given flags and key
    pub fn new(flags: RandomXFlags, key: &[u8]) -> Option<Self> {
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
    
    /// Get a raw pointer to the native cache
    pub fn as_ptr(&self) -> *const c_void {
        self.inner as *const c_void
    }
    
    /// Create a Cache from a raw pointer
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self { inner: ptr }
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

/// RandomX dataset structure used for full verification mode
pub struct Dataset {
    inner: *mut c_void,
}

unsafe impl Send for Dataset {}
unsafe impl Sync for Dataset {}

impl Dataset {
    /// Create a new dataset with the given flags and cache
    pub fn new(flags: RandomXFlags, cache: &Cache) -> Option<Self> {
        extern "C" {
            fn randomx_alloc_dataset(flags: u32) -> *mut c_void;
            fn randomx_dataset_item_count() -> usize;
        }
        
        let ptr = unsafe { randomx_alloc_dataset(flags.bits()) };
        if ptr.is_null() {
            return None;
        }
        
        // Initialize the dataset (this can be computationally expensive)
        Self::init_dataset(ptr, cache, 0, unsafe { randomx_dataset_item_count() });
        
        Some(Self { inner: ptr })
    }
    
    /// Create a dataset from a raw pointer
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self { inner: ptr }
    }
    
    /// Get a raw pointer to the native dataset
    pub fn as_ptr(&self) -> *const c_void {
        self.inner as *const c_void
    }
    
    /// Initialize the dataset with multi-threading support
    pub fn init_parallel(flags: RandomXFlags, cache: &Cache, threads: usize) -> Option<Self> {
        extern "C" {
            fn randomx_alloc_dataset(flags: u32) -> *mut c_void;
            fn randomx_dataset_item_count() -> usize;
        }
        
        let ptr = unsafe { randomx_alloc_dataset(flags.bits()) };
        if ptr.is_null() {
            return None;
        }
        
        // Use parallel initialization if more than one thread is specified
        if threads > 1 {
            let count = unsafe { randomx_dataset_item_count() };
            let perThread = count / threads;
            let remainder = count % threads;
            
            let cache_arc = Arc::new(cache);
            let ptr_arc = Arc::new(Mutex::new(ptr));
            
            let mut handles = Vec::new();
            
            for i in 0..threads {
                let t_cache = Arc::clone(&cache_arc);
                let t_ptr = Arc::clone(&ptr_arc);
                
                let startItem = i * perThread;
                let itemCount = if i == threads - 1 {
                    perThread + remainder
                } else {
                    perThread
                };
                
                let handle = thread::spawn(move || {
                    let locked_ptr = *t_ptr.lock().unwrap();
                    Self::init_dataset(locked_ptr, &t_cache, startItem, itemCount);
                });
                
                handles.push(handle);
            }
            
            // Wait for all threads to complete
            for handle in handles {
                handle.join().unwrap();
            }
        } else {
            // Single-threaded initialization
            let count = unsafe { randomx_dataset_item_count() };
            Self::init_dataset(ptr, cache, 0, count);
        }
        
        Some(Self { inner: ptr })
    }
    
    // Initialize a portion of the dataset
    fn init_dataset(dataset: *mut c_void, cache: &Cache, startItem: usize, itemCount: usize) {
        extern "C" {
            fn randomx_init_dataset(dataset: *mut c_void, cache: *const c_void, 
                                   startItem: usize, itemCount: usize);
        }
        
        unsafe {
            randomx_init_dataset(
                dataset, 
                cache.as_ptr(),
                startItem,
                itemCount
            );
        }
    }
    
    /// Get a dataset item
    pub fn get_item(&self, index: usize) -> [u8; 64] {
        extern "C" {
            fn randomx_get_dataset_item(dataset: *const c_void, index: usize) -> *const u8;
        }
        
        let mut item = [0u8; 64];
        
        unsafe {
            let ptr = randomx_get_dataset_item(self.inner, index);
            ptr::copy_nonoverlapping(ptr, item.as_mut_ptr(), 64);
        }
        
        item
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