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

use crate::common::{IntReg, FpuReg, Addr};
use std::mem;

/// RandomX instruction type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InstructionType {
    Iadd_rs = 0,
    Iadd_m = 1,
    Isub_r = 2,
    Isub_m = 3,
    Imul_r = 4,
    Imul_m = 5,
    Imulh_r = 6,
    Imulh_m = 7,
    Ismulh_r = 8,
    Ismulh_m = 9,
    Imul_rcp = 10,
    Ineg_r = 11,
    Ixor_r = 12,
    Ixor_m = 13,
    Iror_r = 14,
    Irol_r = 15,
    Iswap_r = 16,
    Fswap_r = 17,
    Fadd_r = 18,
    Fadd_m = 19,
    Fsub_r = 20,
    Fsub_m = 21,
    Fscal_r = 22,
    Fmul_r = 23,
    Fdiv_m = 24,
    Fsqrt_r = 25,
    Cbranch = 26,
    Cfround = 27,
    Istore = 28,
    Fstore = 29,
    NOP = 30, // pseudo-instruction
}

/// Instruction execution modifiers
#[repr(u8)]
pub enum ExecutionModifier {
    None = 0,
    Negative = 1,
    Reciprocal = 2,
    RecNeg = 3, // Reciprocal and negative
}

/// Branch condition mask
pub const BRANCH_COND_MASK: u32 = 0x0fffffff;

/// Instruction encoding
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: InstructionType,
    pub dst: u8,
    pub src: u8,
    pub mod_: u8,
    pub imm: i32,
}

impl Instruction {
    /// Creates a new instruction
    pub fn new(opcode: InstructionType, dst: u8, src: u8, mod_: u8, imm: i32) -> Self {
        Self {
            opcode,
            dst,
            src,
            mod_,
            imm,
        }
    }
    
    /// Gets the type of instruction
    pub fn opcode(&self) -> InstructionType {
        self.opcode
    }

    /// Gets the destination register index
    pub fn dst(&self) -> u8 {
        self.dst
    }
    
    /// Gets the source register index
    pub fn src(&self) -> u8 {
        self.src
    }
    
    /// Gets the modifier value
    pub fn mod_(&self) -> u8 {
        self.mod_
    }
    
    /// Gets the immediate value
    pub fn imm(&self) -> i32 {
        self.imm
    }
}

/// Machine state for instruction execution
pub struct MachineState {
    /// Integer registers r0-r7
    pub r: [IntReg; 8],
    
    /// Floating point registers f0-f7
    pub f: [FpuReg; 8],
    
    /// Temporary FPU register
    pub e: FpuReg,
    
    /// Program counter
    pub pc: usize,
    
    /// Program counter checkpoint
    pub checkpoint: usize,
    
    /// Rounding mode
    pub rounding_mode: u32,
    
    /// Scratchpad pointer
    pub scratchpad: *mut u8,
    
    /// Memory size
    pub mem_size: usize,
    
    /// Dataset pointer
    pub dataset: *const u8,
    
    /// Dataset size
    pub dataset_size: usize,
    
    /// Branch mask
    pub branch_mask: u32,
}

/// Execute a single instruction on the given machine state
pub fn execute_instruction(state: &mut MachineState, instr: &Instruction) -> bool {
    use InstructionType::*;

    // Keep track of whether a branch was taken
    let mut branch_taken = false;
    
    match instr.opcode {
        Iadd_rs => {
            let shift = instr.mod_() & 63;
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            state.r[dst] = state.r[dst].wrapping_add(state.r[src].rotate_left(shift));
        }
        
        Iadd_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const IntReg;
                    *ptr
                };
                state.r[dst] = state.r[dst].wrapping_add(value);
            }
        }

        Isub_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            state.r[dst] = state.r[dst].wrapping_sub(state.r[src]);
        }

        Isub_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const IntReg;
                    *ptr
                };
                state.r[dst] = state.r[dst].wrapping_sub(value);
            }
        }

        Imul_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            state.r[dst] = state.r[dst].wrapping_mul(state.r[src]);
        }

        Imul_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const IntReg;
                    *ptr
                };
                state.r[dst] = state.r[dst].wrapping_mul(value);
            }
        }

        Imulh_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            let result = (state.r[dst] as u128).wrapping_mul(state.r[src] as u128) >> 64;
            state.r[dst] = result as u64;
        }

        Imulh_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const IntReg;
                    *ptr
                };
                let result = (state.r[dst] as u128).wrapping_mul(value as u128) >> 64;
                state.r[dst] = result as u64;
            }
        }

        Ismulh_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            let result = ((state.r[dst] as i64 as i128).wrapping_mul(state.r[src] as i64 as i128)) >> 64;
            state.r[dst] = result as u64;
        }

        Ismulh_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const IntReg;
                    *ptr
                };
                let result = ((state.r[dst] as i64 as i128).wrapping_mul(value as i64 as i128)) >> 64;
                state.r[dst] = result as u64;
            }
        }

        Imul_rcp => {
            let dst = instr.dst() as usize;
            if instr.imm() != 0 {
                let divisor = instr.imm() as u64;
                state.r[dst] = imul_rcp(state.r[dst], divisor);
            } else {
                state.r[dst] = 0;
            }
        }

        Ineg_r => {
            let dst = instr.dst() as usize;
            state.r[dst] = (!state.r[dst]).wrapping_add(1);
        }

        Ixor_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            state.r[dst] ^= state.r[src];
        }

        Ixor_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const IntReg;
                    *ptr
                };
                state.r[dst] ^= value;
            }
        }

        Iror_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            let shift = state.r[src] & 63;
            state.r[dst] = state.r[dst].rotate_right(shift as u32);
        }

        Irol_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            let shift = state.r[src] & 63;
            state.r[dst] = state.r[dst].rotate_left(shift as u32);
        }

        Iswap_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            let temp = state.r[dst];
            state.r[dst] = state.r[src];
            state.r[src] = temp;
        }

        Fswap_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            let temp = state.f[dst];
            state.f[dst] = state.f[src];
            state.f[src] = temp;
        }

        Fadd_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            state.f[dst] += state.f[src];
        }

        Fadd_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const FpuReg;
                    *ptr
                };
                state.f[dst] += value;
            }
        }

        Fsub_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            state.f[dst] -= state.f[src];
        }

        Fsub_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const FpuReg;
                    *ptr
                };
                state.f[dst] -= value;
            }
        }

        Fscal_r => {
            // Apply the scaling factor to the destination register
            // The scaling factor is a constant: 0.99999999999999999
            let dst = instr.dst() as usize;
            let scale_factor: f64 = 0.9999999999999999;
            state.f[dst] *= scale_factor;
        }

        Fmul_r => {
            let dst = instr.dst() as usize;
            let src = instr.src() as usize;
            state.f[dst] *= state.f[src];
        }

        Fdiv_m => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let dst = instr.dst() as usize;
            
            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                let value = unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *const FpuReg;
                    *ptr
                };
                
                // Avoid division by zero
                if value != 0.0 {
                    state.f[dst] /= value;
                }
            }
        }

        Fsqrt_r => {
            let dst = instr.dst() as usize;
            // Ensure we're taking square root of a non-negative number
            if state.f[dst] >= 0.0 {
                state.f[dst] = state.f[dst].sqrt();
            }
        }

        Cbranch => {
            let dst = instr.dst() as usize;
            let target = instr.imm() & BRANCH_COND_MASK;
            let condition = ((instr.imm() as u32) >> 28) & 0xf;
            let reg_value = state.r[dst];
            let mask = (1u64 << (condition + 1)) - 1;

            if reg_value & mask == 0 {
                // Take the branch
                state.pc = target as usize;
                branch_taken = true;
            }
        }

        Cfround => {
            let src = instr.src() as usize;
            let imm = instr.imm() & 3;
            state.rounding_mode = ((state.r[src] >> imm) & 3) << 22;
        }

        Istore => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let src = instr.src() as usize;

            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *mut IntReg;
                    *ptr = state.r[src];
                }
            }
        }

        Fstore => {
            // Calculate memory address
            let addr = calculate_address(state, instr);
            let src = instr.src() as usize;

            // Bounds check
            if (addr as usize) < state.mem_size {
                // Safety: We've checked that addr is within bounds
                unsafe {
                    let ptr = state.scratchpad.add(addr as usize) as *mut FpuReg;
                    *ptr = state.f[src];
                }
            }
        }

        NOP => {
            // No operation
        }
    }

    branch_taken
}

/// Calculate memory address for load/store instructions
fn calculate_address(state: &MachineState, instr: &Instruction) -> u32 {
    let addr_reg = instr.src() as usize;
    let addr = (state.r[addr_reg] as u32).wrapping_add(instr.imm() as u32);
    addr & (state.mem_size as u32 - 1)
}

/// Performs an integer multiply by the reciprocal of an integer
fn imul_rcp(a: u64, divisor: u64) -> u64 {
    if a == 0 {
        return 0;
    }

    // Calculate reciprocal
    let k = 64 - divisor.leading_zeros();
    let r = !divisor;
    let shift = k - 1;
    
    // This is simplified for now - in a complete implementation, we would use a lookup table
    let ratio = (r as u128 + 1) << shift;
    let result = ((a as u128 * ratio) >> 64) as u64;
    
    result
}