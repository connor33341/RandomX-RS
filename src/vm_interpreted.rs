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

use std::alloc::{self, Layout};
use std::ptr::NonNull;
use std::ffi::c_void;
use std::cell::UnsafeCell;
use std::sync::Mutex;

use crate::common::{FpuReg, IntReg, RANDOMX_PROGRAM_SIZE, RANDOMX_PROGRAM_ITERATIONS, RandomXError, Result, RANDOMX_HASH_SIZE};
use crate::instructions::{Instruction, execute_instruction, MachineState};
use crate::dataset::Dataset;
use crate::blake2;
use crate::vm::VirtualMachine;

/// InterpretedVirtualMachine is a Rust implementation of the RandomX interpreted VM.
/// This VM executes RandomX bytecode instructions directly without compilation to native code.
pub struct InterpretedVirtualMachine {
    // Use interior mutability with UnsafeCell and Mutex for thread safety
    inner: UnsafeCell<InterpretedVMInner>,
    mutex: Mutex<()>,
}

// Inner structure containing the actual VM state
struct InterpretedVMInner {
    // Machine state
    state: MachineState,
    
    // Program buffer
    program: [Instruction; RANDOMX_PROGRAM_SIZE],
    
    // Configuration
    dataset: Option<NonNull<c_void>>,
    mem_size: usize,
    scratchpad_ptr: NonNull<u8>,
    program_count: usize,
    
    // Execution is ready
    initialized: bool,
}

impl InterpretedVirtualMachine {
    /// Create a new interpreted virtual machine
    pub fn new(dataset: Option<NonNull<c_void>>, mem_size: usize) -> Result<Self> {
        // Allocate scratchpad memory
        let layout = Layout::from_size_align(mem_size, 4096)
            .map_err(|_| RandomXError::SystemError("Failed to create memory layout for scratchpad".to_string()))?;

        let scratchpad_ptr = unsafe {
            let ptr = alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                return Err(RandomXError::SystemError("Failed to allocate scratchpad memory".to_string()));
            }
            NonNull::new_unchecked(ptr)
        };
        
        // Initialize machine state with default values
        let state = MachineState {
            r: [0; 8],
            f: [FpuReg { lo: 0, hi: 0 }; 8],
            e: FpuReg { lo: 0, hi: 0 },
            pc: 0,
            checkpoint: 0,
            rounding_mode: 0,
            scratchpad: scratchpad_ptr.as_ptr(),
            mem_size,
            dataset: dataset.map_or(std::ptr::null(), |ptr| ptr.as_ptr() as *const u8),
            dataset_size: 0, // Will be initialized later
            branch_mask: 0xFFFFFFFF,
        };

        Ok(Self {
            inner: UnsafeCell::new(InterpretedVMInner {
                state,
                program: [Instruction::new(
                    crate::instructions::InstructionType::NOP,
                    0, 0, 0, 0
                ); RANDOMX_PROGRAM_SIZE],
                dataset,
                mem_size,
                scratchpad_ptr,
                program_count: RANDOMX_PROGRAM_ITERATIONS,
                initialized: false,
            }),
            mutex: Mutex::new(()),
        })
    }

    /// Initialize the VM with program data and configuration
    pub fn initialize(&mut self, program: &[Instruction; RANDOMX_PROGRAM_SIZE], config: &[u8]) {
        let _lock = self.mutex.lock().unwrap();
        let inner = unsafe { &mut *self.inner.get() };
        inner.program = *program;

        // Set initial register values from config
        unsafe {
            // First 128 bytes are integer registers (8 registers * 8 bytes each)
            for i in 0..8 {
                let offset = i * 8;
                let value = u64::from_le_bytes([
                    config[offset],
                    config[offset + 1],
                    config[offset + 2],
                    config[offset + 3],
                    config[offset + 4],
                    config[offset + 5],
                    config[offset + 6],
                    config[offset + 7],
                ]);
                inner.state.r[i] = value;
            }
            
            // Next 128 bytes are for FPU registers (8 registers * 16 bytes each)
            for i in 0..8 {
                let offset = 64 + i * 16;
                let lo = u64::from_le_bytes([
                    config[offset],
                    config[offset + 1],
                    config[offset + 2],
                    config[offset + 3],
                    config[offset + 4],
                    config[offset + 5],
                    config[offset + 6],
                    config[offset + 7],
                ]);
                let hi = u64::from_le_bytes([
                    config[offset + 8],
                    config[offset + 9],
                    config[offset + 10],
                    config[offset + 11],
                    config[offset + 12],
                    config[offset + 13],
                    config[offset + 14],
                    config[offset + 15],
                ]);
                inner.state.f[i] = FpuReg { lo, hi };
            }
            
            // Remaining 16 bytes for the 'e' register
            let offset = 192;
            let lo = u64::from_le_bytes([
                config[offset],
                config[offset + 1],
                config[offset + 2],
                config[offset + 3],
                config[offset + 4],
                config[offset + 5],
                config[offset + 6],
                config[offset + 7],
            ]);
            let hi = u64::from_le_bytes([
                config[offset + 8],
                config[offset + 9],
                config[offset + 10],
                config[offset + 11],
                config[offset + 12],
                config[offset + 13],
                config[offset + 14],
                config[offset + 15],
            ]);
            inner.state.e = FpuReg { lo, hi };
        }
        
        inner.initialized = true;
    }

    /// Run the VM program
    pub fn run(&mut self) -> Result<()> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;
        let inner = unsafe { &mut *self.inner.get() };
        if !inner.initialized {
            return Err(RandomXError::InvalidOperation("VM not initialized".to_string()));
        }

        // Reset program counter
        inner.state.pc = 0;
        
        // Run program iterations
        for _ in 0..inner.program_count {
            // Execute program instructions
            while inner.state.pc < RANDOMX_PROGRAM_SIZE {
                let instr = inner.program[inner.state.pc];
                let branch_taken = execute_instruction(&mut inner.state, &instr);
                
                if !branch_taken {
                    inner.state.pc += 1;
                }
            }
            
            // Reset program counter for next iteration
            inner.state.pc = 0;
        }

        Ok(())
    }
    
    /// Get the hash result from the VM state
    pub fn get_result(&self, out: &mut [u8; 32]) -> Result<()> {
        let inner = unsafe { &*self.inner.get() };
        if !inner.initialized {
            return Err(RandomXError::InvalidOperation("VM not initialized".to_string()));
        }

        // Create a buffer for VM state to hash (registers r and f)
        let mut state_buffer = Vec::with_capacity(8 * 8 + 8 * 16);
        
        // Add integer registers to the buffer
        for i in 0..8 {
            state_buffer.extend_from_slice(&inner.state.r[i].to_le_bytes());
        }
        
        // Add floating point registers to the buffer
        for i in 0..8 {
            state_buffer.extend_from_slice(&inner.state.f[i].lo.to_le_bytes());
            state_buffer.extend_from_slice(&inner.state.f[i].hi.to_le_bytes());
        }

        // Use Blake2b to hash the VM state
        blake2::c_compat::hash(&state_buffer, out);
        
        Ok(())
    }
}

impl Drop for InterpretedVirtualMachine {
    fn drop(&mut self) {
        // Free scratchpad memory
        let inner = unsafe { &mut *self.inner.get() };
        if !inner.scratchpad_ptr.as_ptr().is_null() {
            unsafe {
                let layout = Layout::from_size_align_unchecked(inner.mem_size, 4096);
                alloc::dealloc(inner.scratchpad_ptr.as_ptr(), layout);
            }
        }
    }
}

impl VirtualMachine for InterpretedVirtualMachine {
    fn calculate(&self, input: &[u8]) -> Result<[u8; RANDOMX_HASH_SIZE]> {
        let _lock = self.mutex.lock().map_err(|_| RandomXError::MutexPoisoning)?;
        
        // Generate program from input
        let mut program = [Instruction::new(
            crate::instructions::InstructionType::NOP,
            0, 0, 0, 0
        ); RANDOMX_PROGRAM_SIZE];
        
        // Hash the input to generate program and config
        let mut config = [0u8; 256];
        
        // Use Blake2b to hash input and generate program/config
        blake2::generate_program(input, &mut program, &mut config);
        
        // Initialize VM
        unsafe {
            (*self.inner.get()).program = program;
            
            // Set initial register values from config
            // First 128 bytes are integer registers (8 registers * 8 bytes each)
            for i in 0..8 {
                let offset = i * 8;
                let value = u64::from_le_bytes([
                    config[offset],
                    config[offset + 1],
                    config[offset + 2],
                    config[offset + 3],
                    config[offset + 4],
                    config[offset + 5],
                    config[offset + 6],
                    config[offset + 7],
                ]);
                (*self.inner.get()).state.r[i] = value;
            }
            
            // Next 128 bytes are for FPU registers (8 registers * 16 bytes each)
            for i in 0..8 {
                let offset = 64 + i * 16;
                let lo = u64::from_le_bytes([
                    config[offset],
                    config[offset + 1],
                    config[offset + 2],
                    config[offset + 3],
                    config[offset + 4],
                    config[offset + 5],
                    config[offset + 6],
                    config[offset + 7],
                ]);
                let hi = u64::from_le_bytes([
                    config[offset + 8],
                    config[offset + 9],
                    config[offset + 10],
                    config[offset + 11],
                    config[offset + 12],
                    config[offset + 13],
                    config[offset + 14],
                    config[offset + 15],
                ]);
                (*self.inner.get()).state.f[i] = FpuReg { lo, hi };
            }
            
            // Remaining 16 bytes for the 'e' register
            let offset = 192;
            let lo = u64::from_le_bytes([
                config[offset],
                config[offset + 1],
                config[offset + 2],
                config[offset + 3],
                config[offset + 4],
                config[offset + 5],
                config[offset + 6],
                config[offset + 7],
            ]);
            let hi = u64::from_le_bytes([
                config[offset + 8],
                config[offset + 9],
                config[offset + 10],
                config[offset + 11],
                config[offset + 12],
                config[offset + 13],
                config[offset + 14],
                config[offset + 15],
            ]);
            (*self.inner.get()).state.e = FpuReg { lo, hi };
            
            (*self.inner.get()).initialized = true;
        }
        
        // Run VM
        unsafe {
            let inner = &mut *self.inner.get();
            
            // Reset program counter
            inner.state.pc = 0;
            
            // Run program iterations
            for _ in 0..inner.program_count {
                // Execute program instructions
                while inner.state.pc < RANDOMX_PROGRAM_SIZE {
                    let instr = inner.program[inner.state.pc];
                    let branch_taken = execute_instruction(&mut inner.state, &instr);
                    
                    if !branch_taken {
                        inner.state.pc += 1;
                    }
                }
                
                // Reset program counter for next iteration
                inner.state.pc = 0;
            }
        }
        
        // Get result
        let mut hash = [0u8; RANDOMX_HASH_SIZE];
        
        unsafe {
            let inner = &*self.inner.get();
            
            // Create a buffer for VM state to hash (registers r and f)
            let mut state_buffer = Vec::with_capacity(8 * 8 + 8 * 16);
            
            // Add integer registers to the buffer
            for i in 0..8 {
                state_buffer.extend_from_slice(&inner.state.r[i].to_le_bytes());
            }
            
            // Add floating point registers to the buffer
            for i in 0..8 {
                state_buffer.extend_from_slice(&inner.state.f[i].lo.to_le_bytes());
                state_buffer.extend_from_slice(&inner.state.f[i].hi.to_le_bytes());
            }
            
            // Use Blake2b to hash the VM state
            blake2::c_compat::hash(&state_buffer, &mut hash);
        }
        
        Ok(hash)
    }
    
    fn calculate_with_info(&self, input: &[u8], info: &[u8]) -> Result<[u8; RANDOMX_HASH_SIZE]> {
        // For the interpreted VM, we'll combine input and info to create a new input
        let mut combined = Vec::with_capacity(input.len() + info.len());
        combined.extend_from_slice(input);
        combined.extend_from_slice(info);
        
        self.calculate(&combined)
    }
    
    fn calculate_successive(&self, first_input: &[u8], next_inputs: &[&[u8]]) -> Result<Vec<[u8; RANDOMX_HASH_SIZE]>> {
        // For the interpreted VM implementation, we'll just calculate each hash individually
        // A more optimized implementation could reuse some computation
        let mut results = Vec::with_capacity(1 + next_inputs.len());
        
        // Calculate first hash
        results.push(self.calculate(first_input)?);
        
        // Calculate subsequent hashes
        for input in next_inputs {
            results.push(self.calculate(input)?);
        }
        
        Ok(results)
    }
}

/// Creates an interpreted virtual machine instance
pub fn create_interpreted_vm(dataset: Option<&Dataset>, mem_size: usize) -> Option<InterpretedVirtualMachine> {
    let dataset_ptr = dataset.map(|d| unsafe {
        NonNull::new_unchecked(d.as_ptr() as *mut c_void)
    });
    
    match InterpretedVirtualMachine::new(dataset_ptr, mem_size) {
        Ok(vm) => Some(vm),
        Err(_) => None,
    }
}