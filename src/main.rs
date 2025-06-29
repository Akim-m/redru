mod db;
mod hash_index;


use db::InMemoryDB;
use serde_json::{Value, json};
use std::io::{self, Write};
use std::collections::HashMap;

struct DatabaseShell {
    db: InMemoryDB,
    running: bool,
}

impl DatabaseShell {
    fn new() -> io::Result<Self> {
        let db = InMemoryDB::new_persistent("database.json")?;
        Ok(DatabaseShell {
            db,
            running: true,
        })
    }

    fn run(&mut self) -> io::Result<()> {
        self.print_welcome();
        
        while self.running {
            self.print_prompt();
            let input = self.read_input()?;
            self.execute_command(&input)?;
        }
        
        Ok(())
    }

    fn print_welcome(&mut self) {
        println!("🗄️  Interactive Database System with Hash Indexing");
        println!("═══════════════════════════════════════════════════");
        println!("Type 'help' for available commands");
        println!("Database entries: {}", self.db.len());
        println!("Active indexes: {}\n", self.db.list_indexes().len());
    }

    fn print_prompt(&self) {
        print!("db> ");
        io::stdout().flush().unwrap();
    }

    fn read_input(&self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn execute_command(&mut self, input: &str) -> io::Result<()> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0].to_lowercase().as_str() {
            "help" => self.show_help(),
            "set" => self.handle_set(&parts[1..])?,
            "get" => self.handle_get(&parts[1..]),
            "del" | "delete" => self.handle_delete(&parts[1..])?,
            "update" => self.handle_update(&parts[1..])?,
            "exists" => self.handle_exists(&parts[1..]),
            "keys" => self.handle_keys(),
            "count" | "len" => self.handle_count(),
            "clear" => self.handle_clear()?,
            "save" => self.handle_save()?,
            "reload" => self.handle_reload()?,
            "backup" => self.handle_backup(),
            "status" => self.handle_status(),
            "search" => self.handle_search(&parts[1..]),
            "export" => self.handle_export(&parts[1..])?,
            "import" => self.handle_import(&parts[1..])?,
            "create_index" => self.handle_create_index(&parts[1..]),
            "drop_index" => self.handle_drop_index(&parts[1..]),
            "list_indexes" => self.handle_list_indexes(),
            "index_stats" => self.handle_index_stats(&parts[1..]),
            "find_by_value" => self.handle_find_by_value(&parts[1..]),
            "find_by_hash" => self.handle_find_by_hash(&parts[1..]),
            "find_by_field" => self.handle_find_by_field(&parts[1..]),
            "rebuild_index" => self.handle_rebuild_index(&parts[1..]),
            "verify_integrity" => self.handle_verify_integrity(),
            "validate_file" => self.handle_validate_file()?,
            "repair_file" => self.handle_repair_file()?,
            "quit" | "exit" => self.handle_quit(),
            _ => println!("❌ Unknown command: '{}'. Type 'help' for available commands.", parts[0]),
        }
        
        Ok(())
    }

    fn show_help(&self) {
        println!("\n📋 Database Commands:");
        println!("  set <key> <value>     - Store a value");
        println!("  get <key>             - Retrieve a value");
        println!("  update <key> <value>  - Update existing value");
        println!("  delete <key>          - Delete a key-value pair");
        println!("  exists <key>          - Check if key exists");
        println!("  keys                  - List all keys");
        println!("  count                 - Show number of entries");
        println!("  clear                 - Delete all data");
        println!("  search <pattern>      - Search keys containing pattern");
        
        println!("\n🔍 Index Commands:");
        println!("  create_index <name>   - Create a new hash index");
        println!("  drop_index <name>     - Drop an existing index");
        println!("  list_indexes          - Show all indexes");
        println!("  index_stats <name>    - Show index statistics");
        println!("  rebuild_index <name>  - Rebuild an index");
        println!("  find_by_value <index> <value> - Find keys by exact value match");
        println!("  find_by_hash <index> <hash>   - Find keys by hash value");
        println!("  find_by_field <index> <field> <value> - Find by nested field");
        
        println!("\n🔒 Integrity & Security Commands:");
        println!("  verify_integrity      - Verify data integrity using hashes");
        println!("  validate_file         - Validate persistence file integrity");
        println!("  repair_file           - Repair corrupted file from backup");
        
        println!("\n💾 System Commands:");
        println!("  save                  - Manually save to disk");
        println!("  reload                - Reload from disk");
        println!("  export <file>         - Export data to JSON file");
        println!("  import <file>         - Import data from JSON file");
        println!("  status                - Show database status");
        println!("  backup                - Show backup information");
        println!("  help                  - Show this help");
        println!("  quit/exit             - Exit the program\n");
    }

    fn handle_set(&mut self, args: &[&str]) -> io::Result<()> {
        if args.len() < 2 {
            println!("❌ Usage: set <key> <value>");
            return Ok(());
        }

        let key = args[0];
        let value_str = args[1..].join(" ");
        
        let value = self.parse_value(&value_str);
        self.db.insert(key, value)?;
        println!("✅ Set '{}' = '{}'", key, value_str);
        Ok(())
    }

    fn handle_get(&self, args: &[&str]) {
        if args.is_empty() {
            println!("❌ Usage: get <key>");
            return;
        }

        let key = args[0];
        match self.db.get(key) {
            Some(value) => println!("📄 '{}' = {}", key, self.format_value(value)),
            None => println!("❌ Key '{}' not found", key),
        }
    }

    fn handle_delete(&mut self, args: &[&str]) -> io::Result<()> {
        if args.is_empty() {
            println!("❌ Usage: delete <key>");
            return Ok(());
        }

        let key = args[0];
        if self.db.exists(key) {
            self.db.delete(key)?;
            println!("✅ Deleted '{}'", key);
        } else {
            println!("❌ Key '{}' not found", key);
        }
        Ok(())
    }

    fn handle_update(&mut self, args: &[&str]) -> io::Result<()> {
        if args.len() < 2 {
            println!("❌ Usage: update <key> <value>");
            return Ok(());
        }

        let key = args[0];
        let value_str = args[1..].join(" ");
        let value = self.parse_value(&value_str);
        
        if self.db.update(key, value)? {
            println!("✅ Updated '{}' = '{}'", key, value_str);
        } else {
            println!("❌ Key '{}' not found", key);
        }
        Ok(())
    }

    fn handle_exists(&self, args: &[&str]) {
        if args.is_empty() {
            println!("❌ Usage: exists <key>");
            return;
        }

        let key = args[0];
        if self.db.exists(key) {
            println!("✅ Key '{}' exists", key);
        } else {
            println!("❌ Key '{}' does not exist", key);
        }
    }

    fn handle_keys(&self) {
        let keys = self.db.keys();
        if keys.is_empty() {
            println!("📭 No keys found");
        } else {
            println!("🔑 Keys ({}):", keys.len());
            for (i, key) in keys.iter().enumerate() {
                println!("  {}. {}", i + 1, key);
            }
        }
    }

    fn handle_count(&self) {
        println!("📊 Total entries: {}", self.db.len());
    }

    fn handle_clear(&mut self) -> io::Result<()> {
        print!("⚠️  Are you sure you want to delete all data? (y/N): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            self.db.clear()?;
            println!("✅ All data cleared");
        } else {
            println!("❌ Operation cancelled");
        }
        Ok(())
    }

    fn handle_save(&self) -> io::Result<()> {
        self.db.save()?;
        println!("✅ Data saved to disk");
        Ok(())
    }

    fn handle_reload(&mut self) -> io::Result<()> {
        self.db.reload()?;
        println!("✅ Data reloaded from disk");
        Ok(())
    }

    fn handle_backup(&self) {
        println!("💾 Backup system is enabled");
        println!("   Backups are created automatically on save");
        println!("   Hash verification ensures data integrity");
    }

    fn handle_status(&mut self) {
        println!("📊 Database Status:");
        println!("   Entries: {}", self.db.len());
        println!("   Empty: {}", if self.db.is_empty() { "Yes" } else { "No" });
        println!("   Persistence: Enabled");
        println!("   Auto-save: Enabled");
        println!("   Backup: Enabled");
        println!("   Active indexes: {}", self.db.list_indexes().len());
        
        // Check data integrity
        let integrity_status = if self.db.verify_data_integrity() {
            "✅ Valid"
        } else {
            "❌ Corrupted"
        };
        println!("   Data integrity: {}", integrity_status);
        
        if !self.db.list_indexes().is_empty() {
            println!("   Indexes:");
            for index_name in self.db.list_indexes() {
                if let Some((unique_hashes, total_entries)) = self.db.get_index_stats(&index_name) {
                    println!("     {}: {} unique hashes, {} total entries", 
                           index_name, unique_hashes, total_entries);
                }
            }
        }
    }

    fn handle_search(&self, args: &[&str]) {
        if args.is_empty() {
            println!("❌ Usage: search <pattern>");
            return;
        }

        let pattern = args[0].to_lowercase();
        let keys = self.db.keys();
        let matches: Vec<_> = keys.iter()
            .filter(|key| key.to_lowercase().contains(&pattern))
            .collect();

        if matches.is_empty() {
            println!("❌ No keys found matching pattern '{}'", pattern);
        } else {
            println!("🔍 Found {} matches for '{}':", matches.len(), pattern);
            for key in matches {
                if let Some(value) = self.db.get(key) {
                    println!("  {} = {}", key, self.format_value(value));
                }
            }
        }
    }

    fn handle_create_index(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("❌ Usage: create_index <index_name>");
            return;
        }

        let index_name = args[0];
        self.db.create_index(index_name);
        println!("✅ Created index '{}'", index_name);
    }

    fn handle_drop_index(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("❌ Usage: drop_index <index_name>");
            return;
        }

        let index_name = args[0];
        self.db.drop_index(index_name);
        println!("✅ Dropped index '{}'", index_name);
    }

    fn handle_list_indexes(&mut self) {
        let indexes = self.db.list_indexes();
        if indexes.is_empty() {
            println!("📭 No indexes found");
        } else {
            println!("🔍 Active indexes ({}):", indexes.len());
            for (i, index_name) in indexes.iter().enumerate() {
                if let Some((unique_hashes, total_entries)) = self.db.get_index_stats(index_name) {
                    println!("  {}. {} ({} unique hashes, {} entries)", 
                           i + 1, index_name, unique_hashes, total_entries);
                } else {
                    println!("  {}. {}", i + 1, index_name);
                }
            }
        }
    }

    fn handle_index_stats(&self, args: &[&str]) {
        if args.is_empty() {
            println!("❌ Usage: index_stats <index_name>");
            return;
        }

        let index_name = args[0];
        if let Some((unique_hashes, total_entries)) = self.db.get_index_stats(index_name) {
            println!("📊 Index '{}' statistics:", index_name);
            println!("   Unique hash values: {}", unique_hashes);
            println!("   Total indexed entries: {}", total_entries);
            if unique_hashes > 0 {
                let avg_entries = total_entries as f64 / unique_hashes as f64;
                println!("   Average entries per hash: {:.2}", avg_entries);
            }
        } else {
            println!("❌ Index '{}' not found", index_name);
        }
    }

    fn handle_find_by_value(&self, args: &[&str]) {
        if args.len() < 2 {
            println!("❌ Usage: find_by_value <index_name> <value>");
            return;
        }

        let index_name = args[0];
        let value_str = args[1..].join(" ");
        let value = self.parse_value(&value_str);
        
        let results = self.db.find_by_value(index_name, &value);
        if results.is_empty() {
            println!("❌ No entries found for value '{}' in index '{}'", value_str, index_name);
        } else {
            println!("🔍 Found {} matches in index '{}':", results.len(), index_name);
            for key in results {
                if let Some(stored_value) = self.db.get(&key) {
                    println!("  {} = {}", key, self.format_value(stored_value));
                }
            }
        }
    }

    fn handle_find_by_hash(&self, args: &[&str]) {
        if args.len() < 2 {
            println!("❌ Usage: find_by_hash <index_name> <hash_value>");
            return;
        }

        let index_name = args[0];
        let hash_str = args[1];
        
        if let Ok(hash) = hash_str.parse::<u64>() {
            let results = self.db.find_by_hash(index_name, hash);
            if results.is_empty() {
                println!("❌ No entries found for hash {} in index '{}'", hash, index_name);
            } else {
                println!("🔍 Found {} matches for hash {} in index '{}':", results.len(), hash, index_name);
                for key in results {
                    if let Some(value) = self.db.get(&key) {
                        println!("  {} = {}", key, self.format_value(value));
                    }
                }
            }
        } else {
            println!("❌ Invalid hash value: {}", hash_str);
        }
    }

    fn handle_find_by_field(&self, args: &[&str]) {
        if args.len() < 3 {
            println!("❌ Usage: find_by_field <index_name> <field_path> <value>");
            println!("   Example: find_by_field user_index name.first John");
            return;
        }

        let index_name = args[0];
        let field_path = args[1];
        let value_str = args[2..].join(" ");
        let value = self.parse_value(&value_str);
        
        let results = self.db.find_by_field(index_name, field_path, &value);
        if results.is_empty() {
            println!("❌ No entries found for field '{}' = '{}' in index '{}'", 
                   field_path, value_str, index_name);
        } else {
            println!("🔍 Found {} matches for field '{}' = '{}' in index '{}':", 
                   results.len(), field_path, value_str, index_name);
            for key in results {
                if let Some(stored_value) = self.db.get(&key) {
                    println!("  {} = {}", key, self.format_value(stored_value));
                }
            }
        }
    }

    fn handle_rebuild_index(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("❌ Usage: rebuild_index <index_name>");
            return;
        }

        let index_name = args[0];
        self.db.rebuild_index(index_name);
        println!("✅ Rebuilt index '{}'", index_name);
    }

    fn handle_verify_integrity(&self) {
        println!("🔒 Verifying data integrity...");
        
        if self.db.verify_data_integrity() {
            println!("✅ Data integrity verification passed");
            println!("   All data hashes match expected values");
        } else {
            println!("❌ Data integrity verification failed");
            println!("   Data may have been corrupted or tampered with");
            println!("   Consider running 'repair_file' to restore from backup");
        }
    }

    fn handle_validate_file(&self) -> io::Result<()> {
        println!("🔍 Validating persistence file...");
        
        match self.db.validate_file_integrity() {
            Ok(true) => {
                println!("✅ File validation passed");
                println!("   Persistence file is valid and uncorrupted");
            }
            Ok(false) => {
                println!("❌ File validation failed");
                println!("   Persistence file is corrupted or invalid");
                println!("   Consider running 'repair_file' to restore from backup");
            }
            Err(e) => {
                println!("❌ Error during file validation: {}", e);
            }
        }
        
        Ok(())
    }

    fn handle_repair_file(&mut self) -> io::Result<()> {
        println!("🔧 Attempting to repair database from backup...");
        
        match self.db.repair_file() {
            Ok(()) => {
                println!("✅ Database repair completed");
                println!("   Data has been restored from the most recent valid backup");
                println!("   Current entries: {}", self.db.len());
            }
            Err(e) => {
                println!("❌ Database repair failed: {}", e);
                println!("   No valid backups found or all backups are corrupted");
            }
        }
        
        Ok(())
    }

    fn handle_export(&self, args: &[&str]) -> io::Result<()> {
        if args.is_empty() {
            println!("❌ Usage: export <filename>");
            return Ok(());
        }

        let filename = args[0];
        let keys = self.db.keys();
        let mut data = HashMap::new();
        
        for key in keys {
            if let Some(value) = self.db.get(&key) {
                data.insert(key, value.clone());
            }
        }

        let json_data = serde_json::to_string_pretty(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        std::fs::write(filename, json_data)?;
        println!("✅ Exported {} entries to '{}'", data.len(), filename);
        Ok(())
    }

    fn handle_import(&mut self, args: &[&str]) -> io::Result<()> {
        if args.is_empty() {
            println!("❌ Usage: import <filename>");
            return Ok(());
        }

        let filename = args[0];
        let content = std::fs::read_to_string(filename)?;
        let data: HashMap<String, Value> = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut count = 0;
        for (key, value) in data {
            self.db.insert(&key, value)?;
            count += 1;
        }

        println!("✅ Imported {} entries from '{}'", count, filename);
        Ok(())
    }

    fn handle_quit(&mut self) {
        println!("👋 Goodbye!");
        self.running = false;
    }

    fn parse_value(&self, value_str: &str) -> Value {
        if value_str.starts_with('{') || value_str.starts_with('[') {
            serde_json::from_str(value_str).unwrap_or_else(|_| json!(value_str))
        } else if let Ok(num) = value_str.parse::<i64>() {
            json!(num)
        } else if let Ok(float) = value_str.parse::<f64>() {
            json!(float)
        } else if value_str == "true" || value_str == "false" {
            json!(value_str == "true")
        } else {
            json!(value_str)
        }
    }

    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => format!("\"{}\"", s),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(_) | Value::Object(_) => serde_json::to_string_pretty(value).unwrap_or_else(|_| "Invalid JSON".to_string()),
            Value::Null => "null".to_string(),
        }
    }
}

fn main() -> io::Result<()> {


    let mut shell = DatabaseShell::new()?;
    shell.run()
}