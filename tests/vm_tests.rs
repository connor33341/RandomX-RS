use randomx_rs::{
    dataset::Dataset,
    vm::{RandomXVM, VirtualMachine},
    common::{Flag, ResultHash},
    cache::Cache,
};

#[test]
fn test_basic_hash_calculation() {
    // Initialize with a test key
    let key = b"RandomX test key";
    let cache = Cache::new(key).expect("Failed to create cache");
    
    // Create a VM with light mode (no dataset)
    let flags = Flag::DEFAULT;
    let vm = RandomXVM::new(&cache, None, flags).expect("Failed to create VM");
    
    // Calculate a hash
    let input = b"Test input data";
    let hash = vm.calculate(input).expect("Failed to calculate hash");
    
    // Verify we got a valid hash (not checking specific value, just format)
    assert_eq!(hash.len(), 32);
    
    // Basic determinism check - same input should produce same output
    let hash2 = vm.calculate(input).expect("Failed to calculate hash");
    assert_eq!(hash, hash2);
}

#[test]
fn test_batch_hash_calculation() {
    // Initialize with a test key
    let key = b"RandomX batch test key";
    let cache = Cache::new(key).expect("Failed to create cache");
    
    // Create a VM with light mode
    let vm = RandomXVM::new(&cache, None, Flag::DEFAULT).expect("Failed to create VM");
    
    // Test data
    let inputs = vec![
        b"Test input 1".to_vec(),
        b"Test input 2".to_vec(),
        b"Test input 3".to_vec(),
    ];
    
    // Calculate batch of hashes
    let hashes = vm.calculate_batch(&inputs).expect("Failed to calculate batch");
    
    // Verify results
    assert_eq!(hashes.len(), 3);
    
    // Verify deterministic results
    for (i, input) in inputs.iter().enumerate() {
        let single_hash = vm.calculate(input).expect("Failed to calculate individual hash");
        assert_eq!(single_hash, hashes[i]);
    }
}

#[test]
fn test_successive_hash_calculation() {
    // Initialize with a test key
    let key = b"RandomX successive test key";
    let cache = Cache::new(key).expect("Failed to create cache");
    
    // Create a VM with light mode
    let vm = RandomXVM::new(&cache, None, Flag::DEFAULT).expect("Failed to create VM");
    
    // Set up test data
    let first_input = b"First input";
    let next_inputs = vec![
        b"Second input".to_vec(),
        b"Third input".to_vec(),
        b"Fourth input".to_vec(),
    ];
    
    // Calculate successive hashes
    let successive_hashes = vm.calculate_successive(first_input, &next_inputs)
        .expect("Failed to calculate successive hashes");
    
    // Verify we got the expected number of results
    assert_eq!(successive_hashes.len(), next_inputs.len());
    
    // Verify determinism by calculating individually (with state maintenance)
    let first_hash = vm.calculate_first(first_input).expect("Failed to calculate first hash");
    
    let mut manual_successive = Vec::new();
    for input in &next_inputs {
        let next_hash = vm.calculate_next(input).expect("Failed to calculate next hash");
        manual_successive.push(next_hash);
    }
    
    // Compare results
    assert_eq!(successive_hashes, manual_successive);
}

#[test]
fn test_dataset_mode() {
    // Skip this test if we don't want to run slow tests
    if std::env::var("SKIP_SLOW_TESTS").is_ok() {
        return;
    }
    
    // Initialize with a test key
    let key = b"RandomX dataset test key";
    let cache = Cache::new(key).expect("Failed to create cache");
    
    // Create a dataset (this can be slow and memory-intensive)
    let dataset = Dataset::new(&cache, 0).expect("Failed to create dataset");
    
    // Create a VM with full dataset mode
    let flags = Flag::DEFAULT;
    let vm_with_dataset = RandomXVM::new(&cache, Some(&dataset), flags)
        .expect("Failed to create VM with dataset");
    
    // Test data
    let input = b"Test with dataset";
    
    // Calculate hash with dataset
    let hash_with_dataset = vm_with_dataset.calculate(input)
        .expect("Failed to calculate hash with dataset");
    
    // Create a VM in light mode for comparison
    let vm_light = RandomXVM::new(&cache, None, flags)
        .expect("Failed to create VM in light mode");
    
    // Calculate same hash in light mode
    let hash_light = vm_light.calculate(input)
        .expect("Failed to calculate hash in light mode");
    
    // Verify both modes produce the same result
    assert_eq!(hash_with_dataset, hash_light);
}

#[test]
fn test_calculate_hash_basic() {
    // Create a RandomX VM with a simple cache
    let key = b"RandomX test key";
    let input = b"This is a test input";
    let flags = RandomXFlag::DEFAULT;
    
    // Initialize the VM with the key
    let vm = RandomXVM::new(key, flags).expect("Failed to create RandomX VM");
    
    // Calculate a hash
    let hash = vm.calculate_hash(input).expect("Failed to calculate hash");
    
    // Assert that the hash is not empty (actual value will be deterministic but we're just checking basics)
    assert_eq!(hash.len(), 32);
    
    // The same input should produce the same hash
    let second_hash = vm.calculate_hash(input).expect("Failed to calculate hash");
    assert_eq!(hash, second_hash);
}

#[test]
fn test_calculate_batch() {
    // Create a RandomX VM with a simple cache
    let key = b"RandomX batch test key";
    let inputs = vec![
        b"Input 1".to_vec(),
        b"Input 2".to_vec(),
        b"Input 3".to_vec(),
    ];
    let flags = RandomXFlag::DEFAULT;
    
    // Initialize the VM with the key
    let vm = RandomXVM::new(key, flags).expect("Failed to create RandomX VM");
    
    // Calculate hashes in batch
    let hashes = vm.calculate_batch(&inputs).expect("Failed to calculate batch hashes");
    
    // Verify we got the correct number of hashes
    assert_eq!(hashes.len(), inputs.len());
    
    // Verify each hash is the same as if calculated individually
    for (i, input) in inputs.iter().enumerate() {
        let individual_hash = vm.calculate_hash(input).expect("Failed to calculate individual hash");
        assert_eq!(hashes[i], individual_hash);
    }
}

#[test]
fn test_calculate_successive() {
    // Create a RandomX VM with a simple cache
    let key = b"RandomX successive test key";
    let first_input = b"First input";
    let subsequent_inputs = vec![
        b"Second input".to_vec(),
        b"Third input".to_vec(),
        b"Fourth input".to_vec(),
    ];
    let flags = RandomXFlag::DEFAULT;
    
    // Initialize the VM with the key
    let vm = RandomXVM::new(key, flags).expect("Failed to create RandomX VM");
    
    // Calculate successive hashes
    let successive_hashes = vm.calculate_successive(first_input, &subsequent_inputs)
        .expect("Failed to calculate successive hashes");
    
    // Verify we got the correct number of hashes (should be subsequent_inputs.len())
    assert_eq!(successive_hashes.len(), subsequent_inputs.len());
    
    // Verify the successive calculation matches the manual approach
    // First calculate the initial hash to set up the VM state
    let _ = vm.calculate_first(first_input).expect("Failed to calculate first hash");
    
    // Then calculate each subsequent hash and compare with the successive results
    for (i, input) in subsequent_inputs.iter().enumerate() {
        let next_hash = vm.calculate_next(input).expect("Failed to calculate next hash");
        assert_eq!(successive_hashes[i], next_hash);
    }
}

#[test]
fn test_dataset_mode() {
    // Skip if running in CI or environments with limited memory
    if std::env::var("CI").is_ok() {
        return;
    }
    
    // Create a RandomX VM with dataset mode (uses more memory but is faster)
    let key = b"RandomX dataset test key";
    let input = b"Dataset mode test";
    let flags = RandomXFlag::DEFAULT | RandomXFlag::FULL_MEM;
    
    // Initialize the VM with the key and dataset
    let vm = RandomXVM::new(key, flags).expect("Failed to create RandomX VM with dataset");
    
    // Calculate a hash using dataset mode
    let hash = vm.calculate_hash(input).expect("Failed to calculate hash with dataset");
    
    // Just verify we got a hash of the expected length
    assert_eq!(hash.len(), 32);
}