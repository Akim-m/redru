use crate::db::InMemoryDB;
use serde_json::{json, Value};
use std::io;

pub fn run_tests() -> io::Result<()> {
    test_basic_operations()?;
    test_persistence()?;
    test_indexing()?;
    test_search()?;
    test_integrity()?;
    test_backup_repair()?;
    test_import_export()?;
    Ok(())
}

fn test_basic_operations() -> io::Result<()> {
    let mut db = InMemoryDB::new();
    
    db.insert("key1", json!("value1"))?;
    assert!(db.exists("key1"));
    assert_eq!(db.get("key1"), Some(&json!("value1")));
    
    db.update("key1", json!("updated"))?;
    assert_eq!(db.get("key1"), Some(&json!("updated")));
    
    db.delete("key1")?;
    assert!(!db.exists("key1"));
    
    assert!(db.is_empty());
    db.insert("key2", json!({"nested": {"value": 42}}))?;
    assert_eq!(db.len(), 1);
    
    db.clear()?;
    assert!(db.is_empty());
    
    Ok(())
}

fn test_persistence() -> io::Result<()> {
    let file_path = "test_db.json";
    let _ = std::fs::remove_file(file_path);
    
    {
        let mut db = InMemoryDB::new_with_persistence(file_path)?;
        db.insert("persistent1", json!("data1"))?;
        db.insert("persistent2", json!({"a": 1, "b": 2}))?;
        db.save()?;
    }
    
    {
        let db = InMemoryDB::new_with_persistence(file_path)?;
        assert!(db.exists("persistent1"));
        assert!(db.exists("persistent2"));
        assert_eq!(db.len(), 2);
    }
    
    std::fs::remove_file(file_path)?;
    Ok(())
}

fn test_indexing() -> io::Result<()> {
    let mut db = InMemoryDB::new();
    
    db.create_index("test_index");
    db.insert("user1", json!({"name": "Alice", "age": 30}))?;
    db.insert("user2", json!({"name": "Bob", "age": 25}))?;
    db.insert("user3", json!({"name": "Alice", "age": 35}))?;
    
    let alice_results = db.find_by_value("test_index", &json!({"name": "Alice", "age": 30}));
    assert_eq!(alice_results.len(), 1);
    assert!(alice_results.contains(&"user1".to_string()));
    
    let age_results = db.find_by_field("test_index", "age", &json!(25));
    assert_eq!(age_results.len(), 1);
    assert!(age_results.contains(&"user2".to_string()));
    
    let name_results = db.find_by_field("test_index", "name", &json!("Alice"));
    assert_eq!(name_results.len(), 2);
    assert!(name_results.contains(&"user1".to_string()));
    assert!(name_results.contains(&"user3".to_string()));
    
    let stats = db.get_index_stats("test_index").unwrap();
    assert_eq!(stats.0, 3);
    assert_eq!(stats.1, 3);
    
    db.drop_index("test_index");
    assert!(db.list_indexes().is_empty());
    
    Ok(())
}

fn test_search() -> io::Result<()> {
    let mut db = InMemoryDB::new();
    
    db.insert("apple", json!("fruit"))?;
    db.insert("banana", json!("fruit"))?;
    db.insert("carrot", json!("vegetable"))?;
    db.insert("appetizer", json!("food"))?;
    
    let results = db.keys();
    assert_eq!(results.len(), 4);
    
    Ok(())
}

fn test_integrity() -> io::Result<()> {
    let mut db = InMemoryDB::new_with_persistence("integrity_db.json")?;
    db.insert("data1", json!("important"))?;
    db.insert("data2", json!("critical"))?;
    db.save()?;
    
    assert!(db.verify_data_integrity());
    assert!(db.validate_file_integrity()?);
    
    std::fs::write("integrity_db.json", "corrupted data")?;
    assert!(!db.validate_file_integrity()?);
    
    db.repair_file()?;
    assert!(db.validate_file_integrity()?);
    
    std::fs::remove_file("integrity_db.json")?;
    Ok(())
}

fn test_backup_repair() -> io::Result<()> {
    let file_path = "backup_test.json";
    let _ = std::fs::remove_file(file_path);
    
    {
        let mut db = InMemoryDB::new_with_persistence(file_path)?;
        db.set_backup_enabled(true);
        db.insert("backup1", json!("data1"))?;
        db.save()?;
    }
    
    {
        let mut db = InMemoryDB::new_with_persistence(file_path)?;
        db.insert("backup2", json!("data2"))?;
        db.save()?;
    }
    
    std::fs::remove_file(file_path)?;
    
    let mut db = InMemoryDB::new_with_persistence(file_path)?;
    db.repair_file()?;
    assert!(db.exists("backup1"));
    assert!(!db.exists("backup2"));
    
    std::fs::remove_file(file_path)?;
    Ok(())
}

fn test_import_export() -> io::Result<()> {
    let export_file = "export_test.json";
    let mut db = InMemoryDB::new();
    
    db.insert("export1", json!("data1"))?;
    db.insert("export2", json!({"key": "value"}))?;
    
    let keys = db.keys();
    assert_eq!(keys.len(), 2);
    
    let _ = std::fs::remove_file(export_file);
    
    let mut db2 = InMemoryDB::new();
    assert!(db2.is_empty());
    
    std::fs::remove_file(export_file)?;
    Ok(())
}