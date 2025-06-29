use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, BufWriter, BufReader, BufRead};
use std::path::{Path, PathBuf};
use serde_json::{Value, json};
use std::time::SystemTime;

pub struct InMemoryDB {
    storage: HashMap<String, Value>,
    persistence_file: Option<PathBuf>,
    auto_save: bool,
    backup_enabled: bool,
}

impl InMemoryDB {
    pub fn new() -> Self {
        eprintln!("[DEBUG] Initializing new in-memory database.");
        InMemoryDB {
            storage: HashMap::new(),
            persistence_file: None,
            auto_save: true,
            backup_enabled: false,
        }
    }

    pub fn new_with_persistence<P: AsRef<Path>>(file_path: P) -> io::Result<Self> {
        let path_buf = file_path.as_ref().to_path_buf();
        eprintln!("[DEBUG] Initializing persistent database with file: {}", path_buf.display());

        let mut db = InMemoryDB {
            storage: HashMap::new(),
            persistence_file: Some(path_buf.clone()),
            auto_save: true,
            backup_enabled: true,
        };

        // Ensure parent directory exists before attempting to load
        if let Some(parent) = path_buf.parent() {
            if !parent.exists() {
                eprintln!("[DEBUG] Creating parent directory: {}", parent.display());
                fs::create_dir_all(parent).map_err(|e| {
                    eprintln!("[ERROR] Failed to create parent directory {}: {}", parent.display(), e);
                    e
                })?;
            }
        }

        if let Err(e) = db.load_from_file() {
            eprintln!("[WARN] Could not load existing data from {}: {}", path_buf.display(), e);
            // Create empty file if it doesn't exist
            if !path_buf.exists() {
                eprintln!("[DEBUG] Creating new persistence file: {}", path_buf.display());
                db.save_to_file()?;
            }
        }

        Ok(db)
    }

    pub fn new_persistent(file_name: &str) -> io::Result<Self> {
        let stpers_path = PathBuf::from("stpers").join(file_name);
        eprintln!("[DEBUG] Creating persistent DB at: {}", stpers_path.display());
        Self::new_with_persistence(stpers_path)
    }

    pub fn set_auto_save(&mut self, enabled: bool) {
        eprintln!("[DEBUG] Setting auto-save to: {}", enabled);
        self.auto_save = enabled;
    }

    pub fn set_backup_enabled(&mut self, enabled: bool) {
        eprintln!("[DEBUG] Setting backup enabled to: {}", enabled);
        self.backup_enabled = enabled;
    }

    pub fn insert(&mut self, key: &str, value: Value) -> io::Result<()> {
        eprintln!("[DEBUG] Inserting key: {}", key);
        self.storage.insert(key.to_string(), value);

        if self.auto_save && self.persistence_file.is_some() {
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        eprintln!("[DEBUG] Getting value for key: {}", key);
        self.storage.get(key)
    }

    pub fn delete(&mut self, key: &str) -> io::Result<()> {
        eprintln!("[DEBUG] Deleting key: {}", key);
        self.storage.remove(key);

        if self.auto_save && self.persistence_file.is_some() {
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn update(&mut self, key: &str, value: Value) -> io::Result<bool> {
        eprintln!("[DEBUG] Updating key: {}", key);
        if self.storage.contains_key(key) {
            self.storage.insert(key.to_string(), value);

            if self.auto_save && self.persistence_file.is_some() {
                self.save_to_file()?;
            }

            Ok(true)
        } else {
            eprintln!("[DEBUG] Key not found for update: {}", key);
            Ok(false)
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        let exists = self.storage.contains_key(key);
        eprintln!("[DEBUG] Checking existence for key '{}': {}", key, exists);
        exists
    }

    pub fn keys(&self) -> Vec<String> {
        eprintln!("[DEBUG] Retrieving all keys.");
        self.storage.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        let len = self.storage.len();
        eprintln!("[DEBUG] Current number of entries: {}", len);
        len
    }

    pub fn is_empty(&self) -> bool {
        let empty = self.storage.is_empty();
        eprintln!("[DEBUG] Is storage empty? {}", empty);
        empty
    }

    pub fn clear(&mut self) -> io::Result<()> {
        eprintln!("[DEBUG] Clearing all data.");
        self.storage.clear();

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
        eprintln!("[DEBUG] Creating backup at: {}", backup_path.display());

        fs::copy(path, &backup_path).map_err(|e| {
            eprintln!("[WARN] Failed to create backup: {}", e);
            e
        })?;

        Ok(())
    }

    fn save_to_file(&self) -> io::Result<()> {
        if let Some(ref path) = self.persistence_file {
            eprintln!("[DEBUG] Saving data to file: {}", path.display());

            // Create backup before modifying
            self.create_backup(path)?;

            // Serialize data
            let json_data = serde_json::to_string_pretty(&self.storage)
                .map_err(|e| {
                    eprintln!("[ERROR] Failed to serialize storage to JSON: {}", e);
                    io::Error::new(io::ErrorKind::InvalidData, format!("JSON serialization error: {}", e))
                })?;

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    eprintln!("[DEBUG] Creating parent directory: {}", parent.display());
                    fs::create_dir_all(parent).map_err(|e| {
                        eprintln!("[ERROR] Failed to create directories for {}: {}", parent.display(), e);
                        e
                    })?;
                }
            }

            // Write to temporary file first, then rename (atomic operation)
            let temp_path = path.with_extension("tmp");
            
            {
                let file = File::create(&temp_path).map_err(|e| {
                    eprintln!("[ERROR] Failed to create temporary file {}: {}", temp_path.display(), e);
                    e
                })?;

                let mut writer = BufWriter::new(file);
                writer.write_all(json_data.as_bytes()).map_err(|e| {
                    eprintln!("[ERROR] Failed to write data to temporary file: {}", e);
                    e
                })?;

                writer.flush().map_err(|e| {
                    eprintln!("[ERROR] Failed to flush data to temporary file: {}", e);
                    e
                })?;
            } // BufWriter is dropped here, ensuring all data is written

            // Atomic rename
            fs::rename(&temp_path, path).map_err(|e| {
                eprintln!("[ERROR] Failed to rename {} to {}: {}", temp_path.display(), path.display(), e);
                // Clean up temporary file on failure
                let _ = fs::remove_file(&temp_path);
                e
            })?;

            eprintln!("[DEBUG] Successfully saved data to: {}", path.display());
        }
        Ok(())
    }

    fn load_from_file(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.persistence_file {
            if !path.exists() {
                eprintln!("[DEBUG] No persistence file found at: {}", path.display());
                return Ok(());
            }

            eprintln!("[DEBUG] Loading data from file: {}", path.display());

            // Check if file is readable
            let file = File::open(path).map_err(|e| {
                eprintln!("[ERROR] Failed to open file {}: {}", path.display(), e);
                e
            })?;

            let mut reader = BufReader::new(file);
            let mut content = String::new();
            
            // Read file content
            for line_result in reader.lines() {
                let line = line_result.map_err(|e| {
                    eprintln!("[ERROR] Failed to read line from {}: {}", path.display(), e);
                    e
                })?;
                content.push_str(&line);
                content.push('\n');
            }

            if content.trim().is_empty() {
                eprintln!("[DEBUG] File is empty, initializing with empty storage.");
                self.storage = HashMap::new();
                return Ok(());
            }

            // Parse JSON
            let data: HashMap<String, Value> = serde_json::from_str(&content)
                .map_err(|e| {
                    eprintln!("[ERROR] Failed to parse JSON from {}: {}", path.display(), e);
                    eprintln!("[DEBUG] File content preview: {}", &content[..content.len().min(200)]);
                    io::Error::new(io::ErrorKind::InvalidData, format!("JSON parsing error: {}", e))
                })?;

            self.storage = data;
            eprintln!("[DEBUG] Successfully loaded {} entries from file", self.storage.len());
        }
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        eprintln!("[DEBUG] Manual save triggered.");
        self.save_to_file()
    }

    pub fn reload(&mut self) -> io::Result<()> {
        eprintln!("[DEBUG] Manual reload triggered.");
        self.load_from_file()
    }

    pub fn validate_file_integrity(&self) -> io::Result<bool> {
        if let Some(ref path) = self.persistence_file {
            if !path.exists() {
                return Ok(false);
            }

            eprintln!("[DEBUG] Validating file integrity for: {}", path.display());
            
            let content = fs::read_to_string(path)?;
            if content.trim().is_empty() {
                return Ok(true); // Empty file is valid
            }

            match serde_json::from_str::<HashMap<String, Value>>(&content) {
                Ok(_) => {
                    eprintln!("[DEBUG] File integrity check passed");
                    Ok(true)
                }
                Err(e) => {
                    eprintln!("[ERROR] File integrity check failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            Ok(true) // No persistence file means no integrity issues
        }
    }

    pub fn repair_file(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.persistence_file {
            eprintln!("[DEBUG] Attempting to repair file: {}", path.display());
            
            // Try to find a backup file
            let parent = path.parent().unwrap_or(Path::new("."));
            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
            
            let mut backup_files: Vec<_> = fs::read_dir(parent)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.file_name().to_string_lossy().starts_with(&format!("{}.backup.", file_stem))
                })
                .collect();

            // Sort by modification time (newest first)
            backup_files.sort_by_key(|entry| {
                entry.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH)
            });
            backup_files.reverse();

            for backup_entry in backup_files {
                let backup_path = backup_entry.path();
                eprintln!("[DEBUG] Trying backup file: {}", backup_path.display());
                
                if let Ok(content) = fs::read_to_string(&backup_path) {
                    if let Ok(data) = serde_json::from_str::<HashMap<String, Value>>(&content) {
                        eprintln!("[DEBUG] Successfully restored from backup: {}", backup_path.display());
                        self.storage = data;
                        self.save_to_file()?;
                        return Ok(());
                    }
                }
            }

            eprintln!("[WARN] No valid backup found, initializing with empty storage");
            self.storage = HashMap::new();
            self.save_to_file()?;
        }
        Ok(())
    }
}