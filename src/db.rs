use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufWriter, BufReader, BufRead};
use std::path::{Path, PathBuf};
use serde_json::{Value, json};
use std::time::SystemTime;
use crate::hash_index::{HashIndex, hash_value, hash_field_value, calculate_data_hash};

pub struct InMemoryDB {
    storage: HashMap<String, Value>,
    persistence_file: Option<PathBuf>,
    auto_save: bool,
    backup_enabled: bool,
    hash_index: HashIndex,
}

impl InMemoryDB {
    pub fn new() -> Self {
        InMemoryDB {
            storage: HashMap::new(),
            persistence_file: None,
            auto_save: true,
            backup_enabled: false,
            hash_index: HashIndex::new(),
        }
    }

    pub fn new_with_persistence<P: AsRef<Path>>(file_path: P) -> io::Result<Self> {
        let path_buf = file_path.as_ref().to_path_buf();

        let mut db = InMemoryDB {
            storage: HashMap::new(),
            persistence_file: Some(path_buf.clone()),
            auto_save: true,
            backup_enabled: true,
            hash_index: HashIndex::new(),
        };

        if let Some(parent) = path_buf.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if let Err(_) = db.load_from_file() {
            if !path_buf.exists() {
                db.save_to_file()?;
            }
        }

        db.hash_index.load_all_indexes()?;

        Ok(db)
    }

    pub fn new_persistent(file_name: &str) -> io::Result<Self> {
        let stpers_path = PathBuf::from("stpers").join(file_name);
        Self::new_with_persistence(stpers_path)
    }

    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    pub fn set_backup_enabled(&mut self, enabled: bool) {
        self.backup_enabled = enabled;
    }

    pub fn create_index(&mut self, index_name: &str) {
        self.hash_index.create_index(index_name);
        for (key, value) in &self.storage {
            self.hash_index.add_to_index(index_name, key, value);
        }
    }

    pub fn drop_index(&mut self, index_name: &str) {
        self.hash_index.drop_index(index_name);
    }

    pub fn rebuild_index(&mut self, index_name: &str) {
        self.hash_index.rebuild_index(index_name, &self.storage);
    }

    pub fn find_by_value(&self, index_name: &str, value: &Value) -> Vec<String> {
        self.hash_index.find_by_value(index_name, value)
    }

    pub fn find_by_hash(&self, index_name: &str, hash: u64) -> Vec<String> {
        self.hash_index.find_by_hash(index_name, hash)
    }

    pub fn find_by_field(&self, index_name: &str, field_path: &str, search_value: &Value) -> Vec<String> {
        let mut results = Vec::new();
        
        for (key, value) in &self.storage {
            if let Some(field_hash) = hash_field_value(value, field_path) {
                let search_hash = hash_value(search_value);
                if field_hash == search_hash {
                    results.push(key.clone());
                }
            }
        }
        
        results
    }

    pub fn get_index_stats(&self, index_name: &str) -> Option<(usize, usize)> {
        self.hash_index.get_index_stats(index_name)
    }

    pub fn list_indexes(&mut self) -> Vec<String> {
        self.hash_index.list_indexes()
    }

    pub fn verify_data_integrity(&self) -> bool {
        if let Some(ref path) = self.persistence_file {
            if let Some(filename) = path.file_stem() {
                if let Some(filename_str) = filename.to_str() {
                    return self.hash_index.verify_data_integrity(filename_str, &self.storage);
                }
            }
        }
        true
    }

    pub fn insert(&mut self, key: &str, value: Value) -> io::Result<()> {
        for index_name in self.hash_index.list_indexes() {
            self.hash_index.add_to_index(&index_name, key, &value);
        }
        
        self.storage.insert(key.to_string(), value);

        if self.auto_save && self.persistence_file.is_some() {
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.storage.get(key)
    }

    pub fn delete(&mut self, key: &str) -> io::Result<()> {
        if let Some(value) = self.storage.get(key) {
            for index_name in self.hash_index.list_indexes() {
                self.hash_index.remove_from_index(&index_name, key, value);
            }
        }
        
        self.storage.remove(key);

        if self.auto_save && self.persistence_file.is_some() {
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn update(&mut self, key: &str, value: Value) -> io::Result<bool> {
        if self.storage.contains_key(key) {
            if let Some(old_value) = self.storage.get(key) {
                for index_name in self.hash_index.list_indexes() {
                    self.hash_index.remove_from_index(&index_name, key, old_value);
                    self.hash_index.add_to_index(&index_name, key, &value);
                }
            }
            
            self.storage.insert(key.to_string(), value);

            if self.auto_save && self.persistence_file.is_some() {
                self.save_to_file()?;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.storage.contains_key(key)
    }

    pub fn keys(&self) -> Vec<String> {
        self.storage.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.storage.clear();
        
        for index_name in self.hash_index.list_indexes() {
            self.hash_index.clear_index(&index_name);
        }

        if self.auto_save && self.persistence_file.is_some() {
            self.save_to_file()?;
        }

        Ok(())
    }

    fn create_backup(&self, path: &Path) -> io::Result<()> {
        if !self.backup_enabled || !path.exists() {
            return Ok(());
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let backup_path = path.with_extension(format!("backup.{}", timestamp));

        fs::copy(path, &backup_path)?;

        if let Some(filename) = path.file_stem() {
            if let Some(filename_str) = filename.to_str() {
                let hash_file = PathBuf::from("hashes").join(format!("{}.hash", filename_str));
                if hash_file.exists() {
                    let backup_hash_path = PathBuf::from("hashes")
                        .join(format!("{}.backup.{}.hash", filename_str, timestamp));
                    let _ = fs::copy(&hash_file, &backup_hash_path);
                }
            }
        }

        Ok(())
    }

    fn save_to_file(&self) -> io::Result<()> {
        if let Some(ref path) = self.persistence_file {
            self.create_backup(path)?;

            let json_data = serde_json::to_string(&self.storage)
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("JSON serialization error: {}", e))
                })?;

            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            let temp_path = path.with_extension("tmp");
            
            {
                let file = File::create(&temp_path)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(json_data.as_bytes())?;
                writer.flush()?;
            }

            fs::rename(&temp_path, path).map_err(|e| {
                let _ = fs::remove_file(&temp_path);
                e
            })?;

            if let Some(filename) = path.file_stem() {
                if let Some(filename_str) = filename.to_str() {
                    let data_hash = calculate_data_hash(&self.storage);
                    let _ = self.hash_index.save_data_hash(filename_str, &data_hash);
                }
            }
        }
        Ok(())
    }


    fn load_from_file(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.persistence_file {
            if !path.exists() {
                return Ok(());
            }

            let content = fs::read_to_string(path)?;

            if content.trim().is_empty() {
                self.storage = HashMap::new();
                return Ok(());
            }

            let data: HashMap<String, Value> = serde_json::from_str(&content)
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("JSON parsing error: {}", e))
                })?;

            self.storage = data;
            
            for index_name in self.hash_index.list_indexes() {
                self.rebuild_index(&index_name);
            }
        }
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_to_file()
    }

    pub fn reload(&mut self) -> io::Result<()> {
        self.load_from_file()
    }

    pub fn validate_file_integrity(&self) -> io::Result<bool> {
        if let Some(ref path) = self.persistence_file {
            if !path.exists() {
                return Ok(false);
            }
            
            let content = fs::read_to_string(path)?;
            if content.trim().is_empty() {
                return Ok(true);
            }

            serde_json::from_str::<HashMap<String, Value>>(&content)
                .map(|_| true)
                .or(Ok(false))
        } else {
            Ok(true)
        }
    }

    pub fn repair_file(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.persistence_file {
            let parent = path.parent().unwrap_or(Path::new("."));
            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
            
            let mut backup_files: Vec<_> = fs::read_dir(parent)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_name().to_string_lossy().starts_with(&format!("{}.backup.", file_stem))
                })
                .collect();

            backup_files.sort_by_key(|entry| {
                entry.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH)
            });
            backup_files.reverse();

            for backup_entry in backup_files {
                let backup_path = backup_entry.path();
                
                if let Ok(content) = fs::read_to_string(&backup_path) {
                    if let Ok(data) = serde_json::from_str::<HashMap<String, Value>>(&content) {
                        let backup_filename = backup_path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(&file_stem);
                        
                        let hash_dir = PathBuf::from("hashes");
                        let backup_hash_file = hash_dir.join(format!("{}.hash", backup_filename));
                        
                        if backup_hash_file.exists() {
                            if self.hash_index.verify_data_integrity(backup_filename, &data) {
                                self.storage = data;
                                
                                for index_name in self.hash_index.list_indexes() {
                                    self.rebuild_index(&index_name);
                                }
                                
                                self.save_to_file()?;
                                return Ok(());
                            }
                        }
                    }
                }
            }

            self.storage = HashMap::new();
            
            for index_name in self.hash_index.list_indexes() {
                self.hash_index.clear_index(&index_name);
            }
            
            self.save_to_file()?;
        }
        Ok(())
    }
}