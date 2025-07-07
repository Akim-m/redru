mod db;
mod hash_index;
mod tests;
mod vector_db;
mod image_processor;
mod password_manager;

use std::io::{self, Write};
use std::fs;
use std::path::Path;
use db::InMemoryDB;
use hash_index::HashIndex;
use vector_db::run_vector_processing;
use image_processor::run_image_processing;
use password_manager::PasswordManager;

fn main() -> io::Result<()> {
    let mut password_manager = PasswordManager::new()?;
    
    // Check if master password is set
    if !password_manager.is_master_password_set() {
        println!("🔐 Welcome to Geng Database Shell!");
        println!("No master password is set. Would you like to set one? (y/n): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
            password_manager.set_master_password()?;
        }
    } else {
        // Verify master password
        if !password_manager.verify_master_password()? {
            println!("❌ Access denied. Exiting.");
            return Ok(());
        }
    }
    
    loop {
        println!("\nSession options:");
        println!("  1. Use existing session");
        println!("  2. Create new session");
        println!("  3. Delete a session");
        println!("  4. Simse (file-to-vector mode)");
        println!("  5. Image (image processing mode)");
        println!("  6. Password management");
        println!("  7. Exit");
        print!("Select option (1-7): ");
        std::io::stdout().flush()?;
        
        let mut opt = String::new();
        std::io::stdin().read_line(&mut opt)?;
        
        match opt.trim() {
            "1" => use_existing_session(&mut password_manager)?,
            "2" => create_new_session(&mut password_manager)?,
            "3" => delete_session(&mut password_manager)?,
            "4" => {
                if password_manager.verify_master_password()? {
                    run_vector_processing()?;
                }
            }
            "5" => {
                if password_manager.verify_master_password()? {
                    run_image_processing()?;
                }
            }
            "6" => password_management_menu(&mut password_manager)?,
            "7" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid option."),
        }
    }
    Ok(())
}

fn use_existing_session(password_manager: &mut PasswordManager) -> io::Result<()> {
    let sessions = get_available_sessions()?;
    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }
    
    println!("Available sessions:");
    for (i, session) in sessions.iter().enumerate() {
        let protected = password_manager.list_protected_sessions().contains(session);
        let status = if protected { "🔒" } else { "🔓" };
        println!("  {}. {} {}", i + 1, status, session);
    }
    
    print!("Select session (1-{}): ", sessions.len());
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if let Ok(index) = input.trim().parse::<usize>() {
        if index > 0 && index <= sessions.len() {
            let session_name = &sessions[index - 1];
            
            // Check if session is password protected
            if password_manager.list_protected_sessions().contains(session_name) {
                if !password_manager.verify_session_password(session_name)? {
                    println!("❌ Access denied to session '{}'", session_name);
                    return Ok(());
                }
            }
            
            run_session(session_name)?;
        } else {
            println!("Invalid session number.");
        }
    } else {
        println!("Invalid input.");
    }
    Ok(())
}

fn create_new_session(password_manager: &mut PasswordManager) -> io::Result<()> {
    print!("Enter session name: ");
    std::io::stdout().flush()?;
    let mut session_name = String::new();
    std::io::stdin().read_line(&mut session_name)?;
    let session_name = session_name.trim();
    
    if session_name.is_empty() {
        println!("Session name cannot be empty.");
        return Ok(());
    }
    
    // Check if session already exists
    let sessions = get_available_sessions()?;
    if sessions.contains(&session_name.to_string()) {
        println!("Session '{}' already exists.", session_name);
        return Ok(());
    }
    
    // Ask if user wants to password protect this session
    print!("Do you want to password protect this session? (y/n): ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
        password_manager.set_session_password(session_name)?;
    }
    
    // Create session directory
    let session_dir = format!("sessions/{}", session_name);
    fs::create_dir_all(&session_dir)?;
    
    // Create initial database file
    let db_file = format!("{}/database.json", session_dir);
    let db = InMemoryDB::new();
    db.save_to_file_with_path(&db_file)?;
    
    println!("✅ Session '{}' created successfully!", session_name);
    Ok(())
}

fn delete_session(password_manager: &mut PasswordManager) -> io::Result<()> {
    let sessions = get_available_sessions()?;
    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }
    
    println!("Available sessions:");
    for (i, session) in sessions.iter().enumerate() {
        let protected = password_manager.list_protected_sessions().contains(session);
        let status = if protected { "🔒" } else { "🔓" };
        println!("  {}. {} {}", i + 1, status, session);
    }
    
    print!("Select session to delete (1-{}): ", sessions.len());
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if let Ok(index) = input.trim().parse::<usize>() {
        if index > 0 && index <= sessions.len() {
            let session_name = &sessions[index - 1];
            
            // Check if session is password protected
            if password_manager.list_protected_sessions().contains(session_name) {
                if !password_manager.verify_session_password(session_name)? {
                    println!("❌ Access denied to session '{}'", session_name);
                    return Ok(());
                }
            }
            
            print!("Are you sure you want to delete session '{}'? (yes/no): ", session_name);
            std::io::stdout().flush()?;
            let mut confirm = String::new();
            std::io::stdin().read_line(&mut confirm)?;
            
            if confirm.trim().to_lowercase() == "yes" {
                let session_dir = format!("sessions/{}", session_name);
                if Path::new(&session_dir).exists() {
                    fs::remove_dir_all(&session_dir)?;
                }
                password_manager.remove_session_password(session_name)?;
                println!("✅ Session '{}' deleted successfully!", session_name);
            } else {
                println!("Session deletion cancelled.");
            }
        } else {
            println!("Invalid session number.");
        }
    } else {
        println!("Invalid input.");
    }
    Ok(())
}

fn password_management_menu(password_manager: &mut PasswordManager) -> io::Result<()> {
    loop {
        println!("\n🔐 Password Management:");
        println!("  1. Set/Change master password");
        println!("  2. Set session password");
        println!("  3. Remove session password");
        println!("  4. List protected sessions");
        println!("  5. Reset all passwords");
        println!("  6. Back to main menu");
        print!("Select option (1-6): ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                if password_manager.is_master_password_set() {
                    password_manager.change_master_password()?;
                } else {
                    password_manager.set_master_password()?;
                }
            }
            "2" => {
                let sessions = get_available_sessions()?;
                if sessions.is_empty() {
                    println!("No sessions found.");
                    continue;
                }
                
                println!("Available sessions:");
                for (i, session) in sessions.iter().enumerate() {
                    let protected = password_manager.list_protected_sessions().contains(session);
                    let status = if protected { "🔒" } else { "🔓" };
                    println!("  {}. {} {}", i + 1, status, session);
                }
                
                print!("Select session (1-{}): ", sessions.len());
                std::io::stdout().flush()?;
                let mut session_input = String::new();
                std::io::stdin().read_line(&mut session_input)?;
                
                if let Ok(index) = session_input.trim().parse::<usize>() {
                    if index > 0 && index <= sessions.len() {
                        let session_name = &sessions[index - 1];
                        password_manager.set_session_password(session_name)?;
                    }
                }
            }
            "3" => {
                let protected_sessions = password_manager.list_protected_sessions();
                if protected_sessions.is_empty() {
                    println!("No protected sessions found.");
                    continue;
                }
                
                println!("Protected sessions:");
                for (i, session) in protected_sessions.iter().enumerate() {
                    println!("  {}. {}", i + 1, session);
                }
                
                print!("Select session (1-{}): ", protected_sessions.len());
                std::io::stdout().flush()?;
                let mut session_input = String::new();
                std::io::stdin().read_line(&mut session_input)?;
                
                if let Ok(index) = session_input.trim().parse::<usize>() {
                    if index > 0 && index <= protected_sessions.len() {
                        let session_name = &protected_sessions[index - 1];
                        password_manager.remove_session_password(session_name)?;
                    }
                }
            }
            "4" => {
                let protected_sessions = password_manager.list_protected_sessions();
                if protected_sessions.is_empty() {
                    println!("No protected sessions found.");
                } else {
                    println!("Protected sessions:");
                    for session in protected_sessions {
                        println!("  🔒 {}", session);
                    }
                }
            }
            "5" => {
                password_manager.reset_all_passwords()?;
            }
            "6" => break,
            _ => println!("Invalid option."),
        }
    }
    Ok(())
}

fn get_available_sessions() -> io::Result<Vec<String>> {
    let sessions_dir = "sessions";
    if !Path::new(sessions_dir).exists() {
        return Ok(Vec::new());
    }
    
    let sessions: Vec<String> = fs::read_dir(sessions_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    
    Ok(sessions)
}

fn run_session(session_name: &str) -> io::Result<()> {
    let db_file = format!("sessions/{}/database.json", session_name);
    let mut db = InMemoryDB::load_from_file_path(&db_file)?;
    let mut hash_index = HashIndex::new();
    
    println!("🔓 Session '{}' loaded. Type 'help' for commands.", session_name);
    
    let mut command_history: Vec<String> = Vec::new();
    let mut history_index = 0;
    
    loop {
        print!("{}> ", session_name);
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        // Add to command history
        command_history.push(input.to_string());
        history_index = command_history.len();
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "help" => {
                println!("Available commands:");
                println!("  add <key> <json_data>     - Add data to database");
                println!("  get <key>                 - Get data by key");
                println!("  delete <key>              - Delete data by key");
                println!("  list                      - List all keys");
                println!("  search <field> <value>    - Search by field value");
                println!("  index <field>             - Create index on field");
                println!("  find <index> <field> <value> - Find using index");
                println!("  partial <index> <field> <substring> - Partial match search");
                println!("  range <index> <field> <min> <max> - Range search");
                println!("  multi <index> <field1> <value1> [field2 value2...] - Multi-field search");
                println!("  values <index> <field>    - List all values for field");
                println!("  save                      - Save database");
                println!("  backup                    - Create backup");
                println!("  restore                   - Restore from backup");
                println!("  repair                    - Repair corrupted database");
                println!("  stats                     - Show database statistics");
                println!("  auto-save <on|off>        - Toggle auto-save");
                println!("  history                   - Show command history");
                println!("  clear                     - Clear screen");
                println!("  test                      - Run database tests");
                println!("  exit                      - Exit session");
            }
            "add" => {
                if parts.len() < 3 {
                    println!("Usage: add <key> <json_data>");
                    continue;
                }
                let key = parts[1];
                let json_data = parts[2..].join(" ");
                match serde_json::from_str(&json_data) {
                    Ok(data) => {
                        db.add(key, data);
                        println!("✅ Data added successfully!");
                    }
                    Err(e) => println!("❌ Invalid JSON: {}", e),
                }
            }
            "get" => {
                if parts.len() != 2 {
                    println!("Usage: get <key>");
                    continue;
                }
                match db.get(parts[1]) {
                    Some(data) => println!("{}", serde_json::to_string_pretty(&data).unwrap()),
                    None => println!("❌ Key not found"),
                }
            }
            "delete" => {
                if parts.len() != 2 {
                    println!("Usage: delete <key>");
                    continue;
                }
                if db.delete_key(parts[1]) {
                    println!("✅ Data deleted successfully!");
                } else {
                    println!("❌ Key not found");
                }
            }
            "list" => {
                let keys = db.list_keys();
                if keys.is_empty() {
                    println!("No data found.");
                } else {
                    println!("Keys:");
                    for key in keys {
                        println!("  {}", key);
                    }
                }
            }
            "search" => {
                if parts.len() < 3 {
                    println!("Usage: search <field> <value>");
                    continue;
                }
                let field = parts[1];
                let value = parts[2..].join(" ");
                let results = db.search_by_field(field, &value);
                if results.is_empty() {
                    println!("No matches found.");
                } else {
                    println!("Found {} matches:", results.len());
                    for key in results {
                        println!("  {}", key);
                    }
                }
            }
            "index" => {
                if parts.len() != 2 {
                    println!("Usage: index <field>");
                    continue;
                }
                hash_index.create_index(parts[1]);
                println!("✅ Index created successfully!");
            }
            "find" => {
                if parts.len() < 4 {
                    println!("Usage: find <index> <field> <value>");
                    continue;
                }
                let index_name = parts[1];
                let field = parts[2];
                let value = parts[3..].join(" ");
                let value_json = serde_json::Value::String(value);
                let results = hash_index.find_by_value(index_name, &value_json);
                if results.is_empty() {
                    println!("No matches found.");
                } else {
                    println!("Found {} matches:", results.len());
                    for key in results {
                        println!("  {}", key);
                    }
                }
            }
            "partial" => {
                if parts.len() < 4 {
                    println!("Usage: partial <index> <field> <substring>");
                    continue;
                }
                let index_name = parts[1];
                let field = parts[2];
                let substring = parts[3..].join(" ");
                let results = hash_index.find_partial(index_name, field, &substring, &db.get_all_data());
                if results.is_empty() {
                    println!("No matches found.");
                } else {
                    println!("Found {} matches:", results.len());
                    for key in results {
                        println!("  {}", key);
                    }
                }
            }
            "range" => {
                if parts.len() != 5 {
                    println!("Usage: range <index> <field> <min> <max>");
                    continue;
                }
                let index_name = parts[1];
                let field = parts[2];
                if let (Ok(min), Ok(max)) = (parts[3].parse::<f64>(), parts[4].parse::<f64>()) {
                    let results = hash_index.find_range(index_name, field, min, max, &db.get_all_data());
                    if results.is_empty() {
                        println!("No matches found.");
                    } else {
                        println!("Found {} matches:", results.len());
                        for key in results {
                            println!("  {}", key);
                        }
                    }
                } else {
                    println!("❌ Invalid min/max values");
                }
            }
            "multi" => {
                if parts.len() < 4 || parts.len() % 2 != 0 {
                    println!("Usage: multi <index> <field1> <value1> [field2 value2...]");
                    continue;
                }
                let index_name = parts[1];
                let mut field_values = Vec::new();
                for i in (2..parts.len()).step_by(2) {
                    if i + 1 < parts.len() {
                        field_values.push((parts[i].to_string(), serde_json::Value::String(parts[i + 1].to_string())));
                    }
                }
                let results = hash_index.find_multi(index_name, &field_values, &db.get_all_data());
                if results.is_empty() {
                    println!("No matches found.");
                } else {
                    println!("Found {} matches:", results.len());
                    for key in results {
                        println!("  {}", key);
                    }
                }
            }
            "values" => {
                if parts.len() != 3 {
                    println!("Usage: values <index> <field>");
                    continue;
                }
                let index_name = parts[1];
                let field = parts[2];
                let values = hash_index.list_field_values(index_name, field, &db.get_all_data());
                if values.is_empty() {
                    println!("No values found.");
                } else {
                    println!("Field values:");
                    for value in values {
                        println!("  {}", value);
                    }
                }
            }
            "save" => {
                match db.save_to_file_with_path(&db_file) {
                    Ok(_) => println!("✅ Database saved successfully!"),
                    Err(e) => println!("❌ Failed to save: {}", e),
                }
            }
            "backup" => {
                match db.create_backup_with_path(&db_file) {
                    Ok(_) => println!("✅ Backup created successfully!"),
                    Err(e) => println!("❌ Failed to create backup: {}", e),
                }
            }
            "restore" => {
                match db.restore_from_backup_path(&db_file) {
                    Ok(_) => println!("✅ Database restored successfully!"),
                    Err(e) => println!("❌ Failed to restore: {}", e),
                }
            }
            "repair" => {
                match db.repair_corrupted_database(&db_file) {
                    Ok(_) => println!("✅ Database repaired successfully!"),
                    Err(e) => println!("❌ Failed to repair: {}", e),
                }
            }
            "stats" => {
                let stats = db.get_statistics();
                println!("Database Statistics:");
                println!("  Total records: {}", stats.total_records);
                println!("  Total size: {} bytes", stats.total_size);
                println!("  Average record size: {:.2} bytes", stats.average_record_size);
                println!("  Last modified: {}", stats.last_modified);
            }
            "auto-save" => {
                if parts.len() != 2 {
                    println!("Usage: auto-save <on|off>");
                    continue;
                }
                match parts[1] {
                    "on" => {
                        db.enable_auto_save();
                        println!("✅ Auto-save enabled!");
                    }
                    "off" => {
                        db.disable_auto_save();
                        println!("✅ Auto-save disabled!");
                    }
                    _ => println!("Usage: auto-save <on|off>"),
                }
            }
            "history" => {
                if command_history.is_empty() {
                    println!("No command history.");
                } else {
                    println!("Command History:");
                    for (i, cmd) in command_history.iter().enumerate() {
                        println!("  {}. {}", i + 1, cmd);
                    }
                }
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
            }
            "test" => {
                println!("Running database tests...");
                match tests::run_tests() {
                    Ok(_) => println!("✅ All tests passed!"),
                    Err(e) => println!("❌ Tests failed: {}", e),
                }
            }
            "exit" => {
                println!("Saving database before exit...");
                db.save_to_file_with_path(&db_file)?;
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command. Type 'help' for available commands.");
            }
        }
    }
    Ok(())
} 