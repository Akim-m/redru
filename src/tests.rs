use std::fs;
use std::path::Path;
use serde_json::json;
use crate::db::InMemoryDB; // Adjust this import based on your module structure

pub fn run_all_tests() {
    println!("=== Running InMemoryDB Test Suite ===\n");
    
    // Clean up any existing test files
    cleanup_test_files();
    
    // Run all test functions
    test_basic_operations();
    test_persistence_operations();
    test_auto_save_functionality();
    test_backup_functionality();
    test_file_integrity_and_repair();
    test_edge_cases();
    test_error_handling();
    
    // Clean up after tests
    cleanup_test_files();
    
    println!("=== All Tests Completed ===");
}

fn cleanup_test_files() {
    let test_files = [
        "stpers/test_db.json",
        "stpers/test_persistence.json",
        "stpers/test_autosave.json",
        "stpers/test_backup.json",
        "stpers/test_integrity.json",
        "stpers/test_repair.json",
        "stpers/test_edge.json",
        "test_custom_path.json",
    ];
    
    for file in &test_files {
        if Path::new(file).exists() {
            let _ = fs::remove_file(file);
        }
    }
    
    // Remove backup files
    if let Ok(entries) = fs::read_dir("stpers") {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.contains(".backup.") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    
    // Remove stpers directory if empty
    let _ = fs::remove_dir("stpers");
}

fn test_basic_operations() {
    println!("🧪 Testing Basic Operations...");
    
    let mut db = InMemoryDB::new();
    
    // Test initial state
    assert_eq!(db.len(), 0, "New database should be empty");
    assert!(db.is_empty(), "New database should report as empty");
    assert_eq!(db.keys().len(), 0, "New database should have no keys");
    
    // Test insert and get
    db.insert("name", json!("John Doe")).expect("Insert should succeed");
    db.insert("age", json!(30)).expect("Insert should succeed");
    db.insert("active", json!(true)).expect("Insert should succeed");
    
    assert_eq!(db.len(), 3, "Database should have 3 entries");
    assert!(!db.is_empty(), "Database should not be empty");
    
    // Test get
    assert_eq!(db.get("name"), Some(&json!("John Doe")), "Should retrieve correct name");
    assert_eq!(db.get("age"), Some(&json!(30)), "Should retrieve correct age");
    assert_eq!(db.get("active"), Some(&json!(true)), "Should retrieve correct active status");
    assert_eq!(db.get("nonexistent"), None, "Should return None for nonexistent key");
    
    // Test exists
    assert!(db.exists("name"), "Should confirm 'name' exists");
    assert!(db.exists("age"), "Should confirm 'age' exists");
    assert!(!db.exists("nonexistent"), "Should confirm 'nonexistent' doesn't exist");
    
    // Test update
    assert!(db.update("age", json!(31)).expect("Update should succeed"), "Should update existing key");
    assert_eq!(db.get("age"), Some(&json!(31)), "Should retrieve updated age");
    assert!(!db.update("nonexistent", json!("value")).expect("Update should not fail"), "Should return false for nonexistent key");
    
    // Test keys
    let keys = db.keys();
    assert_eq!(keys.len(), 3, "Should have 3 keys");
    assert!(keys.contains(&"name".to_string()), "Keys should contain 'name'");
    assert!(keys.contains(&"age".to_string()), "Keys should contain 'age'");
    assert!(keys.contains(&"active".to_string()), "Keys should contain 'active'");
    
    // Test delete
    db.delete("active").expect("Delete should succeed");
    assert_eq!(db.len(), 2, "Database should have 2 entries after delete");
    assert!(!db.exists("active"), "Deleted key should not exist");
    assert_eq!(db.get("active"), None, "Deleted key should return None");
    
    // Test clear
    db.clear().expect("Clear should succeed");
    assert_eq!(db.len(), 0, "Database should be empty after clear");
    assert!(db.is_empty(), "Database should report as empty after clear");
    assert_eq!(db.keys().len(), 0, "Database should have no keys after clear");
    
    println!("✅ Basic Operations: PASSED\n");
}

fn test_persistence_operations() {
    println!("🧪 Testing Persistence Operations...");
    
    // Test new_persistent constructor
    {
        let mut db = InMemoryDB::new_persistent("test_persistence.json")
            .expect("Should create persistent database");
        
        // Insert some data
        db.insert("persistent_key", json!("persistent_value")).expect("Insert should succeed");
        db.insert("number", json!(42)).expect("Insert should succeed");
        
        assert_eq!(db.len(), 2, "Database should have 2 entries");
    } // db is dropped here, should auto-save
    
    // Load the same database and verify data persisted
    {
        let db = InMemoryDB::new_persistent("test_persistence.json")
            .expect("Should load existing persistent database");
        
        assert_eq!(db.len(), 2, "Loaded database should have 2 entries");
        assert_eq!(db.get("persistent_key"), Some(&json!("persistent_value")), "Should load persistent_key");
        assert_eq!(db.get("number"), Some(&json!(42)), "Should load number");
    }
    
    // Test new_with_persistence with custom path
    {
        let mut db = InMemoryDB::new_with_persistence("test_custom_path.json")
            .expect("Should create database with custom path");
        
        db.insert("custom", json!("path_test")).expect("Insert should succeed");
        assert!(Path::new("test_custom_path.json").exists(), "Custom path file should exist");
    }
    
    // Test manual save and reload
    {
        let mut db = InMemoryDB::new_persistent("test_persistence.json")
            .expect("Should load existing database");
        
        db.insert("manual_test", json!("before_save")).expect("Insert should succeed");
        db.save().expect("Manual save should succeed");
        
        // Modify in memory without auto-save
        db.set_auto_save(false);
        db.insert("temp", json!("not_saved")).expect("Insert should succeed");
        
        // Reload from file
        db.reload().expect("Reload should succeed");
        
        assert!(db.exists("manual_test"), "Should have saved data");
        assert!(!db.exists("temp"), "Should not have unsaved data");
    }
    
    println!("✅ Persistence Operations: PASSED\n");
}

fn test_auto_save_functionality() {
    println!("🧪 Testing Auto-Save Functionality...");
    
    let mut db = InMemoryDB::new_persistent("test_autosave.json")
        .expect("Should create persistent database");
    
    // Test with auto-save enabled (default)
    db.insert("auto_save_test", json!("enabled")).expect("Insert should succeed");
    
    // Verify file was updated
    let file_content = fs::read_to_string("stpers/test_autosave.json")
        .expect("Should read auto-saved file");
    assert!(file_content.contains("auto_save_test"), "File should contain auto-saved data");
    
    // Test with auto-save disabled
    db.set_auto_save(false);
    db.insert("no_auto_save", json!("disabled")).expect("Insert should succeed");
    
    // Reload and verify the non-auto-saved data is not there
    db.reload().expect("Reload should succeed");
    assert!(!db.exists("no_auto_save"), "Non-auto-saved data should not persist");
    assert!(db.exists("auto_save_test"), "Auto-saved data should still exist");
    
    // Re-enable auto-save and test
    db.set_auto_save(true);
    db.insert("re_enabled", json!("auto_save")).expect("Insert should succeed");
    
    db.reload().expect("Reload should succeed");
    assert!(db.exists("re_enabled"), "Re-enabled auto-save should work");
    
    println!("✅ Auto-Save Functionality: PASSED\n");
}

fn test_backup_functionality() {
    println!("🧪 Testing Backup Functionality...");
    
    let mut db = InMemoryDB::new_persistent("test_backup.json")
        .expect("Should create persistent database");
    
    // Enable backups
    db.set_backup_enabled(true);
    
    // Insert initial data
    db.insert("backup_test", json!("initial")).expect("Insert should succeed");
    
    // Wait a moment and insert more data to trigger backup
    std::thread::sleep(std::time::Duration::from_millis(10));
    db.insert("backup_test2", json!("second")).expect("Insert should succeed");
    
    // Check if backup files were created
    let backup_exists = fs::read_dir("stpers")
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry.file_name().to_string_lossy().contains("test_backup.backup.")
            })
        })
        .unwrap_or(false);
    
    assert!(backup_exists, "Backup files should be created when enabled");
    
    // Test with backups disabled
    db.set_backup_enabled(false);
    let initial_backup_count = count_backup_files("test_backup");
    
    db.insert("no_backup", json!("test")).expect("Insert should succeed");
    
    let final_backup_count = count_backup_files("test_backup");
    assert_eq!(initial_backup_count, final_backup_count, "No new backups should be created when disabled");
    
    println!("✅ Backup Functionality: PASSED\n");
}

fn test_file_integrity_and_repair() {
    println!("🧪 Testing File Integrity and Repair...");
    
    // Create a database with valid data
    {
        let mut db = InMemoryDB::new_persistent("test_integrity.json")
            .expect("Should create database");
        
        db.insert("integrity_test", json!("valid_data")).expect("Insert should succeed");
        
        // Test integrity validation on valid file
        assert!(db.validate_file_integrity().expect("Validation should succeed"), 
                "Valid file should pass integrity check");
    }
    
    // Corrupt the file
    fs::write("stpers/test_integrity.json", "invalid json content")
        .expect("Should write corrupted content");
    
    // Test integrity validation on corrupted file
    {
        let db = InMemoryDB::new_persistent("test_integrity.json")
            .expect("Should handle corrupted file gracefully");
        
        assert!(!db.validate_file_integrity().expect("Validation should succeed"), 
                "Corrupted file should fail integrity check");
    }
    
    // Test repair functionality
    {
        let mut db = InMemoryDB::new_persistent("test_repair.json")
            .expect("Should create database for repair test");
        
        // Create some data and backup
        db.set_backup_enabled(true);
        db.insert("repair_test", json!("original")).expect("Insert should succeed");
        
        // Simulate corruption
        fs::write("stpers/test_repair.json", "corrupted content")
            .expect("Should write corrupted content");
        
        // Attempt repair
        db.repair_file().expect("Repair should succeed");
        
        // Verify repair worked (should have empty database since no valid backup for this specific test)
        assert_eq!(db.len(), 0, "Repaired database should be initialized as empty");
        assert!(db.validate_file_integrity().expect("Validation should succeed"), 
                "Repaired file should pass integrity check");
    }
    
    println!("✅ File Integrity and Repair: PASSED\n");
}

fn test_edge_cases() {
    println!("🧪 Testing Edge Cases...");
    
    let mut db = InMemoryDB::new_persistent("test_edge.json")
        .expect("Should create database");
    
    // Test empty string key
    db.insert("", json!("empty_key")).expect("Should handle empty string key");
    assert_eq!(db.get(""), Some(&json!("empty_key")), "Should retrieve value for empty key");
    
    // Test complex JSON values
    let complex_value = json!({
        "nested": {
            "array": [1, 2, 3],
            "object": {
                "key": "value"
            }
        },
        "null_value": null,
        "boolean": true,
        "number": 3.14159
    });
    
    db.insert("complex", complex_value.clone()).expect("Should handle complex JSON");
    assert_eq!(db.get("complex"), Some(&complex_value), "Should retrieve complex JSON correctly");
    
    // Test overwriting existing key with insert
    db.insert("overwrite", json!("original")).expect("Insert should succeed");
    db.insert("overwrite", json!("updated")).expect("Insert should succeed");
    assert_eq!(db.get("overwrite"), Some(&json!("updated")), "Insert should overwrite existing key");
    
    // Test deleting non-existent key
    db.delete("non_existent").expect("Delete should not fail for non-existent key");
    
    // Test updating after deletion
    db.delete("overwrite").expect("Delete should succeed");
    assert!(!db.update("overwrite", json!("after_delete")).expect("Update should not fail"), 
            "Update should return false for deleted key");
    
    // Test very long key
    let long_key = "a".repeat(1000);
    db.insert(&long_key, json!("long_key_value")).expect("Should handle long keys");
    assert_eq!(db.get(&long_key), Some(&json!("long_key_value")), "Should retrieve value for long key");
    
    println!("✅ Edge Cases: PASSED\n");
}

fn test_error_handling() {
    println!("🧪 Testing Error Handling...");
    
    // Test in-memory database (no persistence issues)
    let mut memory_db = InMemoryDB::new();
    memory_db.insert("test", json!("value")).expect("Memory operations should not fail");
    memory_db.delete("test").expect("Memory operations should not fail");
    memory_db.clear().expect("Memory operations should not fail");
    
    // Test persistence with invalid directory (should create directories)
    let result = InMemoryDB::new_with_persistence("deep/nested/path/test.json");
    assert!(result.is_ok(), "Should create nested directories automatically");
    
    // Test empty file handling
    {
        fs::create_dir_all("stpers").ok();
        fs::write("stpers/empty_test.json", "").expect("Should create empty file");
        
        let db = InMemoryDB::new_persistent("empty_test.json")
            .expect("Should handle empty file gracefully");
        
        assert_eq!(db.len(), 0, "Empty file should result in empty database");
    }
    
    // Test file with only whitespace
    {
        fs::write("stpers/whitespace_test.json", "   \n\t  \n  ")
            .expect("Should create whitespace file");
        
        let db = InMemoryDB::new_persistent("whitespace_test.json")
            .expect("Should handle whitespace-only file gracefully");
        
        assert_eq!(db.len(), 0, "Whitespace-only file should result in empty database");
        
        // Clean up
        fs::remove_file("stpers/whitespace_test.json").ok();
        fs::remove_file("stpers/empty_test.json").ok();
    }
    
    println!("✅ Error Handling: PASSED\n");
}

fn count_backup_files(base_name: &str) -> usize {
    fs::read_dir("stpers")
        .map(|entries| {
            entries.flatten()
                .filter(|entry| {
                    entry.file_name().to_string_lossy().contains(&format!("{}.backup.", base_name))
                })
                .count()
        })
        .unwrap_or(0)
}

// Helper function to assert with custom error messages
#[allow(unused)]
macro_rules! assert_eq_msg {
    ($left:expr, $right:expr, $msg:expr) => {
        if $left != $right {
            panic!("{}: expected {:?}, got {:?}", $msg, $right, $left);
        }
    };
}

// Helper function for assertions
#[allow(unused)]
fn assert_eq<T: std::fmt::Debug + PartialEq>(left: T, right: T, message: &str) {
    if left != right {
        panic!("{}: expected {:?}, got {:?}", message, right, left);
    }
}

#[allow(unused)]
fn assert<T>(condition: T, message: &str) 
where 
    T: std::fmt::Debug + PartialEq<bool>
{
    if condition != true {
        panic!("{}", message);
    }
}