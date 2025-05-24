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

use crate::common::{FpuReg, IntReg, RANDOMX_PROGRAM_SIZE, RANDOMX_PROGRAM_ITERATIONS};
use crate::instructions::{Instruction, execute_instruction, MachineState};
use crate::dataset::Dataset;
use crate::blake2;

/// InterpretedVirtualMachine is a Rust implementation of the RandomX interpreted VM.
/// This VM executes RandomX bytecode instructions directly without compilation to native code.
pub struct InterpretedVirtualMachine {
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
    pub fn new(dataset: Option<NonNull<c_void>>, mem_size: usize) -> Result<Self, &'static str> {
        // Allocate scratchpad memory
        let layout = Layout::from_size_align(mem_size, 4096)
            .map_err(|_| "Failed to create memory layout for scratchpad")?;

        let scratchpad_ptr = unsafe {
            let ptr = alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                return Err("Failed to allocate scratchpad memory");
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
        })
    }

    /// Initialize the VM with program data and configuration
    pub fn initialize(&mut self, program: &[Instruction; RANDOMX_PROGRAM_SIZE], config: &[u8]) {
        self.program = *program;

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
                self.state.r[i] = value;
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
                self.state.f[i] = FpuReg { lo, hi };
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
            self.state.e = FpuReg { lo, hi };
        }
        
        self.initialized = true;
    }

    /// Run the VM program
    pub fn run(&mut self) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("VM not initialized");
        }

        // Reset program counter
        self.state.pc = 0;
        
        // Run program iterations
        for _ in 0..self.program_count {
            // Execute program instructions
            while self.state.pc < RANDOMX_PROGRAM_SIZE {
                let instr = self.program[self.state.pc];
                let branch_taken = execute_instruction(&mut self.state, &instr);
                
                if !branch_taken {
                    self.state.pc += 1;
                }
            }
            
            // Reset program counter for next iteration
            self.state.pc = 0;
        }

        Ok(())
    }
    
    /// Get the hash result from the VM state
    pub fn get_result(&self, out: &mut [u8; 32]) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("VM not initialized");
        }

        // Create a buffer for VM state to hash (registers r and f)
        let mut state_buffer = Vec::with_capacity(8 * 8 + 8 * 16);
        
        // Add integer registers to the buffer
        for i in 0..8 {
            state_buffer.extend_from_slice(&self.state.r[i].to_le_bytes());
        }
        
        // Add floating point registers to the buffer
        for i in 0..8 {
            state_buffer.extend_from_slice(&self.state.f[i].lo.to_le_bytes());
            state_buffer.extend_from_slice(&self.state.f[i].hi.to_le_bytes());
        }

        // Use Blake2b to hash the VM state
        blake2::c_compat::hash(&state_buffer, out);
        
        Ok(())
    }
}

impl Drop for InterpretedVirtualMachine {
    fn drop(&mut self) {
        // Free scratchpad memory
        if !self.scratchpad_ptr.as_ptr().is_null() {
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.mem_size, 4096);
                alloc::dealloc(self.scratchpad_ptr.as_ptr(), layout);
            }
        }
    }
}

/// Creates an interpreted virtual machine instance
pub fn create_interpreted_vm(dataset: Option<Dataset>, mem_size: usize) -> Option<InterpretedVirtualMachine> {
    let dataset_ptr = dataset.as_ref().map(|d| unsafe {
        NonNull::new_unchecked(d.as_raw())
    });
    
    match InterpretedVirtualMachine::new(dataset_ptr, mem_size) {
        Ok(vm) => Some(vm),
        Err(_) => None,
    }
}