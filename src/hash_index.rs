use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::io::{self, Write, BufWriter, BufReader, BufRead};
use std::path::{Path, PathBuf};
use serde_json::Value;
use sha2::{Sha256, Digest};

pub struct HashIndex {
    indexes: HashMap<String, HashMap<u64, Vec<String>>>,
    index_dir: PathBuf,
    hash_dir: PathBuf,
}

impl HashIndex {
    pub fn new() -> Self {
        let index_dir = PathBuf::from("Indefx");
        let hash_dir = PathBuf::from("hashes");
        
        if !index_dir.exists() {
            let _ = fs::create_dir_all(&index_dir);
        }
        if !hash_dir.exists() {
            let _ = fs::create_dir_all(&hash_dir);
        }
        
        HashIndex {
            indexes: HashMap::new(),
            index_dir,
            hash_dir,
        }
    }

    pub fn create_index(&mut self, index_name: &str) {
        self.indexes.insert(index_name.to_string(), HashMap::new());
        self.save_index(index_name).unwrap_or(());
    }

    pub fn drop_index(&mut self, index_name: &str) {
        self.indexes.remove(index_name);
        let index_file = self.index_dir.join(format!("{}.json", index_name));
        let hash_file = self.hash_dir.join(format!("{}.hash", index_name));
        let _ = fs::remove_file(index_file);
        let _ = fs::remove_file(hash_file);
    }

    pub fn add_to_index(&mut self, index_name: &str, key: &str, value: &Value) {
        if let Some(index) = self.indexes.get_mut(index_name) {
            let hash = hash_value(value);
            index.entry(hash).or_insert_with(Vec::new).push(key.to_string());
            self.save_index(index_name).unwrap_or(());
        }
    }

    pub fn remove_from_index(&mut self, index_name: &str, key: &str, value: &Value) {
        if let Some(index) = self.indexes.get_mut(index_name) {
            let hash = hash_value(value);
            if let Some(keys) = index.get_mut(&hash) {
                keys.retain(|k| k != key);
                if keys.is_empty() {
                    index.remove(&hash);
                }
            }
            self.save_index(index_name).unwrap_or(());
        }
    }

    pub fn find_by_value(&self, index_name: &str, value: &Value) -> Vec<String> {
        if let Some(index) = self.indexes.get(index_name) {
            let hash = hash_value(value);
            index.get(&hash).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn find_by_hash(&self, index_name: &str, hash: u64) -> Vec<String> {
        if let Some(index) = self.indexes.get(index_name) {
            index.get(&hash).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn get_all_hashes(&self, index_name: &str) -> Vec<u64> {
        if let Some(index) = self.indexes.get(index_name) {
            index.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn rebuild_index(&mut self, index_name: &str, storage: &HashMap<String, Value>) {
        if let Some(index) = self.indexes.get_mut(index_name) {
            index.clear();
            for (key, value) in storage {
                let hash = hash_value(value);
                index.entry(hash).or_insert_with(Vec::new).push(key.clone());
            }
            self.save_index(index_name).unwrap_or(());
        }
    }

    pub fn clear_index(&mut self, index_name: &str) {
        if let Some(index) = self.indexes.get_mut(index_name) {
            index.clear();
            self.save_index(index_name).unwrap_or(());
        }
    }

    pub fn index_exists(&self, index_name: &str) -> bool {
        self.indexes.contains_key(index_name)
    }

    pub fn get_index_stats(&self, index_name: &str) -> Option<(usize, usize)> {
        if let Some(index) = self.indexes.get(index_name) {
            let unique_hashes = index.len();
            let total_entries = index.values().map(|v| v.len()).sum();
            Some((unique_hashes, total_entries))
        } else {
            None
        }
    }

    pub fn list_indexes(&mut self) -> Vec<String> {
        let mut indexes = self.indexes.keys().cloned().collect::<Vec<_>>();
        
        if let Ok(entries) = fs::read_dir(&self.index_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        let index_name = name.trim_end_matches(".json").to_string();
                        if !indexes.contains(&index_name) {
                            if self.load_index(&index_name).is_ok() {
                                indexes.push(index_name);
                            }
                        }
                    }
                }
            }
        }
        
        indexes
    }

    pub fn verify_index_integrity(&self, index_name: &str) -> bool {
        if let Some(index) = self.indexes.get(index_name) {
            let index_file = self.index_dir.join(format!("{}.json", index_name));
            let hash_file = self.hash_dir.join(format!("{}.hash", index_name));
            
            if !index_file.exists() || !hash_file.exists() {
                return false;
            }
            
            if let Ok(stored_hash) = fs::read_to_string(&hash_file) {
                let current_hash = self.calculate_index_hash(index);
                return stored_hash.trim() == current_hash;
            }
        }
        false
    }

    pub fn create_data_hash(&self, data: &HashMap<String, Value>) -> String {
        let mut hasher = Sha256::new();
        let mut keys: Vec<_> = data.keys().collect();
        keys.sort();

        for key in keys {
            hasher.update(key.as_bytes());
            if let Ok(value_bytes) = serde_json::to_vec(&data[key]) {
                hasher.update(&value_bytes);
            }
        }

        format!("{:x}", hasher.finalize())
    }

    pub fn save_data_hash(&self, filename: &str, hash: &str) -> io::Result<()> {
        let hash_file = self.hash_dir.join(format!("{}.hash", filename));
        fs::write(hash_file, hash)?;
        Ok(())
    }

    pub fn verify_data_integrity(&self, filename: &str, data: &HashMap<String, Value>) -> bool {
        let hash_file = self.hash_dir.join(format!("{}.hash", filename));
        
        if let Ok(stored_hash) = fs::read_to_string(&hash_file) {
            let current_hash = self.create_data_hash(data);
            stored_hash.trim() == current_hash
        } else {
            false
        }
    }

    fn calculate_index_hash(&self, index: &HashMap<u64, Vec<String>>) -> String {
        let json_data = serde_json::to_string(index).unwrap_or_default();
        calculate_sha256(&json_data)
    }

    fn save_index(&self, index_name: &str) -> io::Result<()> {
        if let Some(index) = self.indexes.get(index_name) {
            let index_file = self.index_dir.join(format!("{}.json", index_name));
            let hash_file = self.hash_dir.join(format!("{}.hash", index_name));
            let json_data = serde_json::to_string_pretty(index)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
            let temp_file = index_file.with_extension("tmp");
            
            {
                let file = File::create(&temp_file)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(json_data.as_bytes())?;
                writer.flush()?;
            }
            
            fs::rename(&temp_file, &index_file).map_err(|e| {
                let _ = fs::remove_file(&temp_file);
                e
            })?;

            let hash = self.calculate_index_hash(index);
            fs::write(hash_file, hash)?;
        }
        Ok(())
    }

    fn load_index(&mut self, index_name: &str) -> io::Result<()> {
        let index_file = self.index_dir.join(format!("{}.json", index_name));
        
        if !index_file.exists() {
            return Ok(());
        }

        let file = File::open(&index_file)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        
        for line_result in reader.lines() {
            let line = line_result?;
            content.push_str(&line);
            content.push('\n');
        }

        if content.trim().is_empty() {
            self.indexes.insert(index_name.to_string(), HashMap::new());
            return Ok(());
        }

        let index_data: HashMap<u64, Vec<String>> = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.indexes.insert(index_name.to_string(), index_data);
        Ok(())
    }

    pub fn load_all_indexes(&mut self) -> io::Result<()> {
        if !self.index_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.index_dir)?;
        for entry in entries {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let index_name = name.trim_end_matches(".json");
                    self.load_index(index_name)?;
                }
            }
        }
        Ok(())
    }

    /// Find keys where a field contains a substring (case-insensitive, for String fields)
    pub fn find_partial(&self, index_name: &str, field: &str, substring: &str, storage: &HashMap<String, Value>) -> Vec<String> {
        let mut results = Vec::new();
        let substring = substring.to_lowercase();
        for (key, value) in storage {
            if let Some(field_value) = crate::hash_index::extract_field_value(value, field) {
                if let Some(s) = field_value.as_str() {
                    if s.to_lowercase().contains(&substring) {
                        results.push(key.clone());
                    }
                }
            }
        }
        results
    }

    /// Find keys where a numeric field is within a range (inclusive)
    pub fn find_range(&self, index_name: &str, field: &str, min: f64, max: f64, storage: &HashMap<String, Value>) -> Vec<String> {
        let mut results = Vec::new();
        for (key, value) in storage {
            if let Some(field_value) = crate::hash_index::extract_field_value(value, field) {
                if let Some(n) = field_value.as_f64() {
                    if n >= min && n <= max {
                        results.push(key.clone());
                    }
                }
            }
        }
        results
    }

    /// Find keys where multiple fields match specified values (all must match)
    pub fn find_multi(&self, index_name: &str, field_values: &[(String, Value)], storage: &HashMap<String, Value>) -> Vec<String> {
        let mut results = Vec::new();
        'outer: for (key, value) in storage {
            for (field, expected) in field_values {
                if let Some(field_value) = crate::hash_index::extract_field_value(value, field) {
                    if field_value != expected {
                        continue 'outer;
                    }
                } else {
                    continue 'outer;
                }
            }
            results.push(key.clone());
        }
        results
    }

    /// List all unique values for a given field in an index
    pub fn list_field_values(&self, index_name: &str, field: &str, storage: &HashMap<String, Value>) -> Vec<Value> {
        let mut values = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for value in storage.values() {
            if let Some(field_value) = crate::hash_index::extract_field_value(value, field) {
                if seen.insert(field_value.clone()) {
                    values.push(field_value.clone());
                }
            }
        }
        values
    }
}

pub fn hash_value(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash_json_value(value, &mut hasher);
    hasher.finish()
}

fn hash_json_value(value: &Value, hasher: &mut DefaultHasher) {
    match value {
        Value::Null => 0u8.hash(hasher),
        Value::Bool(b) => {
            1u8.hash(hasher);
            b.hash(hasher);
        }
        Value::Number(n) => {
            2u8.hash(hasher);
            if let Some(i) = n.as_i64() {
                i.hash(hasher);
            } else if let Some(u) = n.as_u64() {
                u.hash(hasher);
            } else if let Some(f) = n.as_f64() {
                f.to_bits().hash(hasher);
            }
        }
        Value::String(s) => {
            3u8.hash(hasher);
            s.hash(hasher);
        }
        Value::Array(arr) => {
            4u8.hash(hasher);
            arr.len().hash(hasher);
            for item in arr {
                hash_json_value(item, hasher);
            }
        }
        Value::Object(obj) => {
            5u8.hash(hasher);
            obj.len().hash(hasher);
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();
            for key in keys {
                key.hash(hasher);
                hash_json_value(&obj[key], hasher);
            }
        }
    }
}

pub fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub fn hash_field_value(value: &Value, field_path: &str) -> Option<u64> {
    if let Some(field_value) = extract_field_value(value, field_path) {
        Some(hash_value(field_value))
    } else {
        None
    }
}

fn extract_field_value<'a>(value: &'a Value, field_path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = field_path.split('.').collect();
    let mut current = value;
    
    for part in parts {
        match current {
            Value::Object(obj) => {
                current = obj.get(part)?;
            }
            Value::Array(arr) => {
                if let Ok(index) = part.parse::<usize>() {
                    current = arr.get(index)?;
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    
    Some(current)
}

pub fn calculate_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn calculate_data_hash(data: &HashMap<String, Value>) -> String {
    let json_data = serde_json::to_string(data).unwrap_or_default();
    calculate_sha256(&json_data)
}

pub fn verify_data_hash(data: &HashMap<String, Value>, expected_hash: &str) -> bool {
    let current_hash = calculate_data_hash(data);
    current_hash == expected_hash
}