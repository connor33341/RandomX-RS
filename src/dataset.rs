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
use std::ptr::NonNull;
use std::slice;
use crate::RANDOMX_DATASET_ITEM_SIZE;

extern "C" {
    fn randomx_init_cache(cache: *mut c_void, key: *const c_void, key_size: libc::size_t);
    fn randomx_release_cache(cache: *mut c_void);
    
    fn randomx_init_dataset(dataset: *mut c_void, cache: *mut c_void, start_item: libc::c_ulong, item_count: libc::c_ulong);
    fn randomx_get_dataset_memory(dataset: *mut c_void) -> *mut c_void;
    fn randomx_release_dataset(dataset: *mut c_void);
}

/// RandomX Cache wrapper
pub struct Cache {
    inner: NonNull<c_void>,
}

/// RandomX Dataset wrapper
pub struct Dataset {
    inner: NonNull<c_void>,
}

impl Cache {
    /// Creates a Cache wrapper from a raw pointer
    /// 
    /// # Safety
    /// 
    /// This function should only be called with a valid pointer to a RandomX cache
    /// that was allocated with randomx_alloc_cache
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        assert!(!ptr.is_null());
        Self {
            inner: NonNull::new_unchecked(ptr),
        }
    }
    
    /// Gets a raw pointer to the underlying cache
    pub fn as_raw(&self) -> *mut c_void {
        self.inner.as_ptr()
    }
    
    /// Initializes the cache with the given key
    pub fn init(&mut self, key: &[u8]) {
        unsafe {
            randomx_init_cache(
                self.inner.as_ptr(),
                key.as_ptr() as *const c_void,
                key.len(),
            );
        }
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        unsafe {
            randomx_release_cache(self.inner.as_ptr());
        }
    }
}

impl Dataset {
    /// Creates a Dataset wrapper from a raw pointer
    /// 
    /// # Safety
    /// 
    /// This function should only be called with a valid pointer to a RandomX dataset
    /// that was allocated with randomx_alloc_dataset
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        assert!(!ptr.is_null());
        Self {
            inner: NonNull::new_unchecked(ptr),
        }
    }
    
    /// Gets a raw pointer to the underlying dataset
    pub fn as_raw(&self) -> *mut c_void {
        self.inner.as_ptr()
    }
    
    /// Initializes a range of items in the dataset using the provided cache
    pub fn init(&mut self, cache: &Cache, start_item: u64, item_count: u64) {
        unsafe {
            randomx_init_dataset(
                self.inner.as_ptr(),
                cache.as_raw(),
                start_item,
                item_count,
            );
        }
    }
    
    /// Gets a reference to the dataset's memory buffer
    pub fn memory(&self) -> &[u8] {
        unsafe {
            let ptr = randomx_get_dataset_memory(self.inner.as_ptr());
            let count = crate::dataset_item_count() as usize;
            let size = count * RANDOMX_DATASET_ITEM_SIZE;
            slice::from_raw_parts(ptr as *const u8, size)
        }
    }
    
    /// Gets a mutable reference to the dataset's memory buffer
    pub fn memory_mut(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = randomx_get_dataset_memory(self.inner.as_ptr());
            let count = crate::dataset_item_count() as usize;
            let size = count * RANDOMX_DATASET_ITEM_SIZE;
            slice::from_raw_parts_mut(ptr as *mut u8, size)
        }
    }
}

impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe {
            randomx_release_dataset(self.inner.as_ptr());
        }
    }
}

// Ensure these types are Send and Sync
unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}
unsafe impl Send for Dataset {}
unsafe impl Sync for Dataset {}