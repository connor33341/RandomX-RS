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

use crate::common::{RandomXError, Result};
use crate::RandomXFlags;
use std::ffi::{c_void, c_uint};
use std::ptr;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::thread;

extern "C" {
    fn randomx_alloc_cache(flags: c_uint) -> *mut c_void;
    fn randomx_init_cache(cache: *mut c_void, key: *const c_void, keySize: usize);
    fn randomx_release_cache(cache: *mut c_void);
    fn randomx_alloc_dataset(flags: c_uint) -> *mut c_void;
    fn randomx_dataset_item_count() -> c_uint;
    fn randomx_init_dataset(dataset: *mut c_void, cache: *mut c_void, startItem: c_uint, itemCount: c_uint);
    fn randomx_release_dataset(dataset: *mut c_void);
}

/// RandomX cache structure used for light verification mode
pub struct Cache {
    pub(crate) handle: *mut c_void,
}

impl Cache {
    /// Create a new cache with the given flags and key
    pub fn new(flags: RandomXFlags, key: &[u8]) -> Result<Self> {
        let handle = unsafe { randomx_alloc_cache(flags.bits()) };
        if handle.is_null() {
            return Err(RandomXError::CacheCreationError);
        }
        
        unsafe {
            randomx_init_cache(handle, key.as_ptr() as *const c_void, key.len());
        }
        
        Ok(Cache { handle })
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                randomx_release_cache(self.handle);
            }
        }
    }
}

// Cache is Send and Sync since the underlying C structure is thread-safe
// after initialization
unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

/// RandomX dataset structure used for full verification mode
pub struct Dataset {
    pub(crate) handle: *mut c_void,
    _cache: Option<Arc<Cache>>,  // Keep cache alive as long as the dataset exists
}

impl Dataset {
    /// Create a new dataset with the given flags and cache
    pub fn new(flags: RandomXFlags, cache: Option<Arc<Cache>>) -> Result<Self> {
        let handle = unsafe { randomx_alloc_dataset(flags.bits()) };
        if handle.is_null() {
            return Err(RandomXError::DatasetCreationError);
        }
        
        // Initialize dataset with cache if provided
        if let Some(cache) = &cache {
            let item_count = unsafe { randomx_dataset_item_count() };
            unsafe {
                randomx_init_dataset(handle, cache.handle, 0, item_count);
            }
        }
        
        Ok(Dataset {
            handle,
            _cache: cache,
        })
    }
    
    /// Gets the number of items in a full RandomX dataset
    pub fn item_count() -> u32 {
        unsafe { randomx_dataset_item_count() }
    }
    
    /// Initialize a range of dataset items
    pub fn init_items(&self, cache: &Cache, start_item: u32, item_count: u32) -> Result<()> {
        if self.handle.is_null() {
            return Err(RandomXError::InvalidOperation(
                "Cannot initialize items on an invalid dataset".to_string()
            ));
        }
        
        unsafe {
            randomx_init_dataset(self.handle, cache.handle, start_item, item_count);
        }
        
        Ok(())
    }
}

impl Drop for Dataset {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                randomx_release_dataset(self.handle);
            }
        }
    }
}

// Dataset is Send and Sync since the underlying C structure is thread-safe
// after initialization
unsafe impl Send for Dataset {}
unsafe impl Sync for Dataset {}