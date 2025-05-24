use std::env;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Link against the C/C++ RandomX implementation during transition
    println!("cargo:rustc-link-search=native=./src");
    println!("cargo:rustc-link-lib=static=randomx");
    
    // Rebuild when these files change
    println!("cargo:rerun-if-changed=src/randomx.h");
    println!("cargo:rerun-if-changed=src/randomx.cpp");
    println!("cargo:rerun-if-changed=build.rs");
    
    // Platform-specific flags
    let mut cc_config = cc::Build::new();
    cc_config
        .cpp(true)
        .include("src")
        .warnings(false)
        .extra_warnings(false);

    // Configure C/C++ compiler flags based on platform
    if target.contains("msvc") {
        // MSVC-specific flags
        cc_config.flag("/EHsc").flag("/O2").flag("/std:c++14");
    } else {
        // GCC/Clang flags
        cc_config
            .flag("-std=c++14")
            .flag("-O3")
            .flag("-fno-rtti")
            .flag("-Wno-unused-parameter");

        if target.contains("x86_64") {
            cc_config.flag("-maes");
        } else if target.contains("aarch64") {
            cc_config.flag("-march=armv8-a+crypto");
        }
    }
    
    // Detect CPU features for optimal compilation
    #[cfg(feature = "hwloc")]
    {
        if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") || cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=hwloc");
        }
    }

    // Compile C/C++ files
    let mut native_sources = vec![
        "src/aes_hash.cpp",
        "src/allocator.cpp",
        "src/assembly_generator_x86.cpp",
        "src/blake2_generator.cpp",
        "src/bytecode_machine.cpp",
        "src/cpu.cpp",
        "src/dataset.cpp",
        "src/instruction.cpp",
        "src/instructions_portable.cpp",
        "src/randomx.cpp",
        "src/reciprocal.c",
        "src/soft_aes.cpp",
        "src/superscalar.cpp",
        "src/virtual_machine.cpp",
        "src/virtual_memory.c",
        "src/vm_compiled_light.cpp",
        "src/vm_compiled.cpp",
        "src/vm_interpreted_light.cpp",
        "src/vm_interpreted.cpp",
        "src/argon2_core.c",
        "src/argon2_ref.c",
    ];

    // Add platform-specific JIT compiler implementations
    if target.contains("x86_64") {
        native_sources.push("src/jit_compiler_x86.cpp");
        if target.contains("linux") || target.contains("freebsd") || target.contains("dragonfly") {
            cc_config.define("XMRIG_OS_UNIX", "1");
            cc_config.flag("-fPIC");
            
            // For x86-64 Linux, we can use the precompiled assembly file
            println!("cargo:rustc-link-arg=-Wl,--whole-archive");
            cc_config.asm_flag("-f elf64");
            cc_config.file("src/jit_compiler_x86_static.S");
            println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
        } else if target.contains("windows") {
            cc_config.define("XMRIG_OS_WIN", "1");
            
            // For Windows, we need to compile the assembly file separately
            if target.contains("msvc") {
                cc_config.asm_flag("/c");
                cc_config.file("src/jit_compiler_x86_static.asm");
            } else {
                cc_config.asm_flag("-f win64");
                cc_config.file("src/jit_compiler_x86_static.S");
            }
        }
    } else if target.contains("aarch64") {
        native_sources.push("src/jit_compiler_a64.cpp");
        if target.contains("linux") || target.contains("freebsd") || target.contains("dragonfly") {
            cc_config.define("XMRIG_OS_UNIX", "1");
            cc_config.flag("-fPIC");
            cc_config.file("src/jit_compiler_a64_static.S");
        }
    } else if target.contains("riscv64") {
        native_sources.push("src/jit_compiler_rv64.cpp");
        if target.contains("linux") || target.contains("freebsd") || target.contains("dragonfly") {
            cc_config.define("XMRIG_OS_UNIX", "1");
            cc_config.flag("-fPIC");
            cc_config.file("src/jit_compiler_rv64_static.S");
        }
    }
    
    // Add optimized Argon2 implementations if supported
    if target.contains("x86_64") || target.contains("i686") {
        native_sources.push("src/argon2_ssse3.c");
        if !target.contains("msvc") {
            cc_config.flag("-mssse3");
        }
        
        native_sources.push("src/argon2_avx2.c");
        if !target.contains("msvc") {
            cc_config.flag("-mavx2");
        }
    }
    
    // Compile all source files
    cc_config.files(&native_sources);
    cc_config.compile("randomx");
    
    // Generate Rust bindings to C/C++ code
    let bindings = bindgen::Builder::default()
        .header("src/randomx.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}