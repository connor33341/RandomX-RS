/*
Copyright (c) 2023-2025, connor33341 (Rust implementation)

All rights reserved.
*/

#[cfg(test)]
mod vm_tests {
    use crate::vm::{RandomXVM, VirtualMachine};
    use crate::dataset::{Cache, Dataset};
    use crate::RandomXFlags;
    use crate::common::RANDOMX_HASH_SIZE;

    const TEST_KEY: &[u8] = b"RandomX test key";

    // Test vectors for validation
    const TEST_INPUT_1: &[u8] = b"Test input 1";
    const TEST_INPUT_2: &[u8] = b"Test input 2";
    const TEST_INPUT_3: &[u8] = b"Test input 3";

    #[test]
    fn test_vm_creation() {
        let cache = Cache::new(TEST_KEY, RandomXFlags::default()).expect("Failed to create cache");
        let vm = RandomXVM::new(RandomXFlags::default(), Some(&cache), None);
        assert!(vm.is_ok(), "VM creation failed");
    }
    
    #[test]
    fn test_basic_hash_calculation() {
        let cache = Cache::new(TEST_KEY, RandomXFlags::default()).expect("Failed to create cache");
        let mut vm = RandomXVM::new(RandomXFlags::default(), Some(&cache), None).expect("Failed to create VM");
        
        let hash = vm.calculate(TEST_INPUT_1).expect("Failed to calculate hash");
        
        // Verify hash is non-zero and correct size
        assert_eq!(hash.len(), RANDOMX_HASH_SIZE);
        assert!(hash.iter().any(|&b| b != 0), "Hash contains all zeros");
        
        // Calculate the same hash again to verify consistency
        let hash2 = vm.calculate(TEST_INPUT_1).expect("Failed to calculate hash");
        assert_eq!(hash, hash2, "Hash calculation not deterministic");
        
        // Calculate a different hash to verify it changes with input
        let hash3 = vm.calculate(TEST_INPUT_2).expect("Failed to calculate hash");
        assert_ne!(hash, hash3, "Different inputs produce the same hash");
    }
    
    #[test]
    fn test_batch_calculation() {
        let cache = Cache::new(TEST_KEY, RandomXFlags::default()).expect("Failed to create cache");
        let mut vm = RandomXVM::new(RandomXFlags::default(), Some(&cache), None).expect("Failed to create VM");
        
        // Create batch of inputs
        let inputs = vec![TEST_INPUT_1, TEST_INPUT_2, TEST_INPUT_3];
        
        // Calculate hashes in batch
        let hashes = vm.calculate_batch(&inputs).expect("Failed to calculate batch hashes");
        
        // Verify correct number of hashes
        assert_eq!(hashes.len(), inputs.len());
        
        // Verify each hash against individual calculation
        for (i, input) in inputs.iter().enumerate() {
            let individual_hash = vm.calculate(input).expect("Failed to calculate individual hash");
            assert_eq!(hashes[i], individual_hash, "Batch hash doesn't match individual hash");
        }
    }
    
    #[test]
    fn test_successive_calculation() {
        let cache = Cache::new(TEST_KEY, RandomXFlags::default()).expect("Failed to create cache");
        let mut vm = RandomXVM::new(RandomXFlags::default(), Some(&cache), None).expect("Failed to create VM");
        
        // Create successive inputs
        let first_input = TEST_INPUT_1;
        let next_inputs = vec![TEST_INPUT_2, TEST_INPUT_3];
        
        // Calculate hashes in succession
        let successive_hashes = vm.calculate_successive(first_input, &next_inputs).expect("Failed to calculate successive hashes");
        
        // Verify correct number of hashes returned
        assert_eq!(successive_hashes.len(), next_inputs.len());
        
        // Verify hashes are not all the same
        if next_inputs.len() > 1 {
            assert_ne!(
                successive_hashes[0], 
                successive_hashes[1], 
                "Successive hash calculations produced identical hashes"
            );
        }
        
        // Validate that successive calculation produces different results than independent calculations
        // Because the VM state is influenced by the previous calculation
        let mut independent_vm = RandomXVM::new(RandomXFlags::default(), Some(&cache), None).expect("Failed to create VM");
        let independent_hash = independent_vm.calculate(TEST_INPUT_2).expect("Failed to calculate hash");
        
        // The successive hash for the same input should be different due to VM state influence
        assert_ne!(
            successive_hashes[0], 
            independent_hash, 
            "Successive hash calculation not influenced by previous state"
        );
    }
    
    #[test]
    fn test_dataset_mode() {
        // Skip this test if running in a CI environment where it might be too slow
        if std::env::var("CI").is_ok() {
            return;
        }

        let flags = RandomXFlags::default() | RandomXFlags::FLAG_FULL_MEM;
        let cache = Cache::new(TEST_KEY, flags).expect("Failed to create cache");
        
        // Create a dataset (this might take some time)
        let dataset = Dataset::new(&cache, flags).expect("Failed to create dataset");
        
        // Create VM in full mode with dataset
        let mut vm = RandomXVM::new(flags, None, Some(&dataset)).expect("Failed to create VM");
        
        // Test calculation works with dataset
        let hash = vm.calculate(TEST_INPUT_1).expect("Failed to calculate hash with dataset");
        assert_eq!(hash.len(), RANDOMX_HASH_SIZE);
        assert!(hash.iter().any(|&b| b != 0), "Hash contains all zeros");
    }
}