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

// RandomX Virtual Machine wrapper implementation
use crate::common::{RandomXError, Result, RANDOMX_HASH_SIZE};
use crate::dataset::{Cache, Dataset};
use crate::RandomXFlags;

use std::ptr::{null, null_mut};
use std::ffi::c_void;
use std::cell::UnsafeCell;
use std::sync::{Mutex, Arc};

/// Trait for RandomX virtual machine implementations
pub trait VirtualMachine {
    /// Calculate a RandomX hash for the given input
    fn calculate(&self, input: &[u8]) -> Result<[u8; RANDOMX_HASH_SIZE]>;
    
    /// Calculate a RandomX hash with customization info
    fn calculate_with_info(&self, input: &[u8], info: &[u8]) -> Result<[u8; RANDOMX_HASH_SIZE]>;
    
    /// Calculate hashes for multiple inputs (batch processing)
    fn calculate_batch(&self, inputs: &[&[u8]]) -> Result<Vec<[u8; RANDOMX_HASH_SIZE]>> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            results.push(self.calculate(input)?);
        }
        Ok(results)
    }
    
    /// Calculate hashes for successive inputs with internal state reuse for better performance
    fn calculate_successive(&self, first_input: &[u8], next_inputs: &[&[u8]]) -> Result<Vec<[u8; RANDOMX_HASH_SIZE]>>;
}

/// Wrapper around the C RandomX VM implementation
pub struct RandomXVM {
    vm_ptr: UnsafeCell<*mut c_void>,
    mutex: Mutex<()>, // For interior mutability and thread safety
}

// Raw C FFI declarations
extern "C" {
    fn randomx_create_vm(flags: u32, cache: *const c_void, dataset: *const c_void) -> *mut c_void;
    fn randomx_vm_set_cache(machine: *mut c_void, cache: *const c_void);
    fn randomx_vm_set_dataset(machine: *mut c_void, dataset: *const c_void);
    fn randomx_destroy_vm(machine: *mut c_void);
    fn randomx_calculate_hash(
        machine: *mut c_void,
        input: *const u8,
        input_size: usize,
        output: *mut u8,
    );
    fn randomx_calculate_hash_first(
        machine: *mut c_void,
        input: *const u8,
        input_size: usize,
    );
    fn randomx_calculate_hash_next(
        machine: *mut c_void,
        input: *const u8,
        input_size: usize,
        output: *mut u8,
    );
}

impl RandomXVM {
    /// Creates a new RandomX virtual machine
    ///
    /// # Arguments
    /// * `flags` - RandomX flags that affect VM behavior
    /// * `cache` - Optional cache for light-mode verification
    /// * `dataset` - Optional dataset for full verification mode
    ///
    /// # Returns
    /// A Result containing the VM or an error
    pub fn new(
        flags: RandomXFlags,
        cache: Option<&Cache>,
        dataset: Option<&Dataset>,
    ) -> Result<Self> {
        let cache_ptr = match cache {
            Some(c) => c.as_ptr(),
            None => null(),
        };

        let dataset_ptr = match dataset {
            Some(d) => d.as_ptr(),
            None => null(),
        };

        let vm_ptr = unsafe { randomx_create_vm(flags.bits(), cache_ptr, dataset_ptr) };

        if vm_ptr.is_null() {
            return Err(RandomXError::VmCreationError);
        }

        Ok(Self {
            vm_ptr: UnsafeCell::new(vm_ptr),
            mutex: Mutex::new(()),
        })
    }

    /// Sets the cache to be used by this virtual machine
    ///
    /// # Arguments
    /// * `cache` - The cache to use
    ///
    /// # Returns
    /// A Result indicating success or an error
    pub fn set_cache(&self, cache: &Cache) -> Result<()> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;

        unsafe {
            randomx_vm_set_cache(*self.vm_ptr.get(), cache.as_ptr());
        }

        Ok(())
    }

    /// Sets the dataset to be used by this virtual machine
    ///
    /// # Arguments
    /// * `dataset` - The dataset to use
    ///
    /// # Returns
    /// A Result indicating success or an error
    pub fn set_dataset(&self, dataset: &Dataset) -> Result<()> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;

        unsafe {
            randomx_vm_set_dataset(*self.vm_ptr.get(), dataset.as_ptr());
        }

        Ok(())
    }
    
    /// Returns whether this VM is still valid
    pub fn is_valid(&self) -> bool {
        unsafe { !(*self.vm_ptr.get()).is_null() }
    }
    
    /// Calculate hash for the first input in a series
    /// This is used internally for successive calculations
    fn calculate_first(&self, input: &[u8]) -> Result<()> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;

        unsafe {
            randomx_calculate_hash_first(
                *self.vm_ptr.get(),
                input.as_ptr(),
                input.len(),
            );
        }

        Ok(())
    }
    
    /// Calculate hash for next inputs after the first in a series
    fn calculate_next(&self, input: &[u8]) -> Result<[u8; RANDOMX_HASH_SIZE]> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;
        let mut hash = [0u8; RANDOMX_HASH_SIZE];
        
        unsafe {
            randomx_calculate_hash_next(
                *self.vm_ptr.get(),
                input.as_ptr(),
                input.len(),
                hash.as_mut_ptr(),
            );
        }

        Ok(hash)
    }
}

impl VirtualMachine for RandomXVM {
    fn calculate(&self, input: &[u8]) -> Result<[u8; RANDOMX_HASH_SIZE]> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;
        
        unsafe {
            if (*self.vm_ptr.get()).is_null() {
                return Err(RandomXError::InvalidOperation(
                    "Virtual machine not initialized".to_string(),
                ));
            }
            
            let mut hash = [0u8; RANDOMX_HASH_SIZE];

            randomx_calculate_hash(
                *self.vm_ptr.get(),
                input.as_ptr(),
                input.len(),
                hash.as_mut_ptr(),
            );

            Ok(hash)
        }
    }

    fn calculate_with_info(&self, input: &[u8], info: &[u8]) -> Result<[u8; RANDOMX_HASH_SIZE]> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;
        
        unsafe {
            if (*self.vm_ptr.get()).is_null() {
                return Err(RandomXError::InvalidOperation(
                    "Virtual machine not initialized".to_string(),
                ));
            }
            
            // Since randomx_calculate_hash_with_info doesn't appear to be available in the C API,
            // we'll implement our own version by combining the input and info
            let combined_input = [input, info].concat();
            let mut hash = [0u8; RANDOMX_HASH_SIZE];

            randomx_calculate_hash(
                *self.vm_ptr.get(),
                combined_input.as_ptr(),
                combined_input.len(),
                hash.as_mut_ptr(),
            );

            Ok(hash)
        }
    }
    
    fn calculate_successive(&self, first_input: &[u8], next_inputs: &[&[u8]]) -> Result<Vec<[u8; RANDOMX_HASH_SIZE]>> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;
        
        unsafe {
            if (*self.vm_ptr.get()).is_null() {
                return Err(RandomXError::InvalidOperation(
                    "Virtual machine not initialized".to_string(),
                ));
            }

            // Calculate the first hash and initialize VM state
            randomx_calculate_hash_first(
                *self.vm_ptr.get(),
                first_input.as_ptr(),
                first_input.len(),
            );
            
            // Calculate subsequent hashes reusing VM state
            let mut results = Vec::with_capacity(next_inputs.len());
            for input in next_inputs {
                let mut hash = [0u8; RANDOMX_HASH_SIZE];
                randomx_calculate_hash_next(
                    *self.vm_ptr.get(),
                    input.as_ptr(),
                    input.len(),
                    hash.as_mut_ptr(),
                );
                results.push(hash);
            }
            
            Ok(results)
        }
    }
}

impl Drop for RandomXVM {
    fn drop(&mut self) {
        // Handle mutex lock failure gracefully - we can't use ? in drop
        let lock_result = self.mutex.lock();
        if let Ok(_lock) = lock_result {
            unsafe {
                if !(*self.vm_ptr.get()).is_null() {
                    randomx_destroy_vm(*self.vm_ptr.get());
                    *self.vm_ptr.get() = null_mut();
                }
            }
        }
    }
}

// Implementing Send and Sync for RandomXVM is safe because we ensure proper memory management
// and thread-safety through the native C library's guarantees
unsafe impl Send for RandomXVM {}
unsafe impl Sync for RandomXVM {}