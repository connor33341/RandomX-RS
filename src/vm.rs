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
use crate::dataset::{Cache, Dataset};

extern "C" {
    fn randomx_vm_set_cache(machine: *mut c_void, cache: *mut c_void);
    fn randomx_vm_set_dataset(machine: *mut c_void, dataset: *mut c_void);
    fn randomx_destroy_vm(machine: *mut c_void);
}

/// RandomX Virtual Machine wrapper
pub struct VirtualMachine {
    inner: NonNull<c_void>,
}

impl VirtualMachine {
    /// Creates a VirtualMachine wrapper from a raw pointer
    ///
    /// # Safety
    ///
    /// This function should only be called with a valid pointer to a RandomX VM
    /// that was allocated with randomx_create_vm
    pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
        assert!(!ptr.is_null());
        Self {
            inner: NonNull::new_unchecked(ptr),
        }
    }

    /// Gets a raw pointer to the underlying virtual machine
    pub fn as_raw(&self) -> *mut c_void {
        self.inner.as_ptr()
    }

    /// Sets a new cache for the VM
    ///
    /// This should be called when the cache is reinitialized with a new key.
    /// This function is only valid for VMs created without RANDOMX_FLAG_FULL_MEM.
    pub fn set_cache(&mut self, cache: &Cache) {
        unsafe {
            randomx_vm_set_cache(self.inner.as_ptr(), cache.as_raw());
        }
    }

    /// Sets a new dataset for the VM
    ///
    /// This function is only valid for VMs created with RANDOMX_FLAG_FULL_MEM.
    pub fn set_dataset(&mut self, dataset: &Dataset) {
        unsafe {
            randomx_vm_set_dataset(self.inner.as_ptr(), dataset.as_raw());
        }
    }
}

impl Drop for VirtualMachine {
    fn drop(&mut self) {
        unsafe {
            randomx_destroy_vm(self.inner.as_ptr());
        }
    }
}

// Ensure VirtualMachine is Send and Sync
unsafe impl Send for VirtualMachine {}
unsafe impl Sync for VirtualMachine {}