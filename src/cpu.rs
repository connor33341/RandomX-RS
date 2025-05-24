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

use lazy_static::lazy_static;

lazy_static! {
    /// The CPU features available on the current system
    pub static ref CPU: Cpu = Cpu::new();
}

/// CPU feature detection
pub struct Cpu {
    has_aes: bool,
    has_ssse3: bool,
    has_avx2: bool,
    has_avx512f: bool,
    has_pclmul: bool,
    has_neon: bool,
    has_asimd: bool,
}

impl Cpu {
    /// Creates a new CPU instance with detected features
    pub fn new() -> Self {
        Self {
            has_aes: Cpu::detect_aes(),
            has_ssse3: Cpu::detect_ssse3(),
            has_avx2: Cpu::detect_avx2(),
            has_avx512f: Cpu::detect_avx512f(),
            has_pclmul: Cpu::detect_pclmul(),
            has_neon: Cpu::detect_neon(),
            has_asimd: Cpu::detect_asimd(),
        }
    }

    /// Check if the CPU supports AES instructions
    pub fn has_aes(&self) -> bool {
        self.has_aes
    }

    /// Check if the CPU supports SSSE3 instructions
    pub fn has_ssse3(&self) -> bool {
        self.has_ssse3
    }

    /// Check if the CPU supports AVX2 instructions
    pub fn has_avx2(&self) -> bool {
        self.has_avx2
    }

    /// Check if the CPU supports AVX512F instructions
    pub fn has_avx512f(&self) -> bool {
        self.has_avx512f
    }

    /// Check if the CPU supports PCLMUL instructions
    pub fn has_pclmul(&self) -> bool {
        self.has_pclmul
    }

    /// Check if the CPU supports NEON instructions (ARM)
    pub fn has_neon(&self) -> bool {
        self.has_neon
    }

    /// Check if the CPU supports ASIMD instructions (ARM)
    pub fn has_asimd(&self) -> bool {
        self.has_asimd
    }

    // Feature detection methods
    // For now, we'll use platform-specific code for each feature
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn detect_aes() -> bool {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        
        unsafe {
            let cpuid = __cpuid(1);
            // AES: ECX bit 25
            (cpuid.ecx & (1 << 25)) != 0
        }
    }
    
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    fn detect_aes() -> bool {
        if cfg!(target_arch = "aarch64") {
            // On ARM, AES is part of the Crypto Extension
            // For a proper implementation, we should use getauxval(AT_HWCAP)
            // But for now, we'll assume false unless we implement the detection
            false
        } else {
            false
        }
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn detect_ssse3() -> bool {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        
        unsafe {
            let cpuid = __cpuid(1);
            // SSSE3: ECX bit 9
            (cpuid.ecx & (1 << 9)) != 0
        }
    }
    
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    fn detect_ssse3() -> bool {
        false
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn detect_avx2() -> bool {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        
        unsafe {
            let (max_level, _) = __get_cpuid_max(0);
            if max_level >= 7 {
                let cpuid = __cpuid_count(7, 0);
                // AVX2: EBX bit 5
                (cpuid.ebx & (1 << 5)) != 0
            } else {
                false
            }
        }
    }
    
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    fn detect_avx2() -> bool {
        false
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn detect_avx512f() -> bool {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        
        unsafe {
            let (max_level, _) = __get_cpuid_max(0);
            if max_level >= 7 {
                let cpuid = __cpuid_count(7, 0);
                // AVX512F: EBX bit 16
                (cpuid.ebx & (1 << 16)) != 0
            } else {
                false
            }
        }
    }
    
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    fn detect_avx512f() -> bool {
        false
    }
    
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn detect_pclmul() -> bool {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        
        unsafe {
            let cpuid = __cpuid(1);
            // PCLMUL: ECX bit 1
            (cpuid.ecx & (1 << 1)) != 0
        }
    }
    
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    fn detect_pclmul() -> bool {
        false
    }
    
    #[cfg(target_arch = "aarch64")]
    fn detect_neon() -> bool {
        // NEON is always available on AArch64
        true
    }
    
    #[cfg(not(target_arch = "aarch64"))]
    fn detect_neon() -> bool {
        false
    }
    
    #[cfg(target_arch = "aarch64")]
    fn detect_asimd() -> bool {
        // Advanced SIMD (ASIMD) is always available on AArch64
        true
    }
    
    #[cfg(not(target_arch = "aarch64"))]
    fn detect_asimd() -> bool {
        false
    }
}