use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordData {
    pub hashed_password: String,
    pub salt: String,
    pub session_passwords: HashMap<String, String>, // session_name -> hashed_password
}

pub struct PasswordManager {
    password_file: String,
    password_data: Option<PasswordData>,
}

impl PasswordManager {
    pub fn new() -> io::Result<Self> {
        let password_file = "passwords.json".to_string();
        let password_data = if Path::new(&password_file).exists() {
            let content = fs::read_to_string(&password_file)?;
            serde_json::from_str(&content).ok()
        } else {
            None
        };
        
        Ok(PasswordManager {
            password_file,
            password_data,
        })
    }

    pub fn is_master_password_set(&self) -> bool {
        self.password_data.is_some()
    }

    pub fn set_master_password(&mut self) -> io::Result<()> {
        print!("Enter master password: ");
        std::io::stdout().flush()?;
        let mut password = String::new();
        std::io::stdin().read_line(&mut password)?;
        let password = password.trim();

        print!("Confirm master password: ");
        std::io::stdout().flush()?;
        let mut confirm = String::new();
        std::io::stdin().read_line(&mut confirm)?;
        let confirm = confirm.trim();

        if password != confirm {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Passwords don't match"));
        }

        let salt = argon2::password_hash::SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Password hash error: {}", e)))?;

        self.password_data = Some(PasswordData {
            hashed_password: password_hash.to_string(),
            salt: salt.to_string(),
            session_passwords: HashMap::new(),
        });

        self.save_password_data()?;
        println!("✅ Master password set successfully!");
        Ok(())
    }

    pub fn verify_master_password(&self) -> io::Result<bool> {
        if let Some(ref data) = self.password_data {
            print!("Enter master password: ");
            std::io::stdout().flush()?;
            let mut password = String::new();
            std::io::stdin().read_line(&mut password)?;
            let password = password.trim();

            let parsed_hash = PasswordHash::new(&data.hashed_password)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Hash parse error: {}", e)))?;

            match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
                Ok(_) => {
                    println!("✅ Master password verified!");
                    Ok(true)
                }
                Err(_) => {
                    println!("❌ Incorrect master password!");
                    Ok(false)
                }
            }
        } else {
            Ok(true) // No password set, allow access
        }
    }

    pub fn set_session_password(&mut self, session_name: &str) -> io::Result<()> {
        if let Some(ref mut data) = self.password_data {
            print!("Enter password for session '{}': ", session_name);
            std::io::stdout().flush()?;
            let mut password = String::new();
            std::io::stdin().read_line(&mut password)?;
            let password = password.trim();

            print!("Confirm password: ");
            std::io::stdout().flush()?;
            let mut confirm = String::new();
            std::io::stdin().read_line(&mut confirm)?;
            let confirm = confirm.trim();

            if password != confirm {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Passwords don't match"));
            }

            let salt = argon2::password_hash::SaltString::generate(&mut rand::thread_rng());
            let argon2 = Argon2::default();
            let password_hash = argon2.hash_password(password.as_bytes(), &salt)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Password hash error: {}", e)))?;

            data.session_passwords.insert(session_name.to_string(), password_hash.to_string());
            self.save_password_data()?;
            println!("✅ Session password set successfully!");
        }
        Ok(())
    }

    pub fn verify_session_password(&self, session_name: &str) -> io::Result<bool> {
        if let Some(ref data) = self.password_data {
            if let Some(ref hashed_password) = data.session_passwords.get(session_name) {
                print!("Enter password for session '{}': ", session_name);
                std::io::stdout().flush()?;
                let mut password = String::new();
                std::io::stdin().read_line(&mut password)?;
                let password = password.trim();

                let parsed_hash = PasswordHash::new(hashed_password)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Hash parse error: {}", e)))?;

                match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
                    Ok(_) => {
                        println!("✅ Session password verified!");
                        Ok(true)
                    }
                    Err(_) => {
                        println!("❌ Incorrect session password!");
                        Ok(false)
                    }
                }
            } else {
                Ok(true) // No password set for this session
            }
        } else {
            Ok(true) // No master password set
        }
    }

    pub fn remove_session_password(&mut self, session_name: &str) -> io::Result<()> {
        if let Some(ref mut data) = self.password_data {
            if data.session_passwords.remove(session_name).is_some() {
                self.save_password_data()?;
                println!("✅ Session password removed!");
            } else {
                println!("No password found for session '{}'", session_name);
            }
        }
        Ok(())
    }

    pub fn list_protected_sessions(&self) -> Vec<String> {
        if let Some(ref data) = self.password_data {
            data.session_passwords.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn save_password_data(&self) -> io::Result<()> {
        if let Some(ref data) = self.password_data {
            let json = serde_json::to_string_pretty(data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            fs::write(&self.password_file, json)?;
        }
        Ok(())
    }

    pub fn change_master_password(&mut self) -> io::Result<()> {
        if self.verify_master_password()? {
            self.set_master_password()?;
        }
        Ok(())
    }

    pub fn reset_all_passwords(&mut self) -> io::Result<()> {
        print!("Are you sure you want to reset all passwords? (yes/no): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "yes" {
            if Path::new(&self.password_file).exists() {
                fs::remove_file(&self.password_file)?;
            }
            self.password_data = None;
            println!("✅ All passwords reset!");
        } else {
            println!("Password reset cancelled.");
        }
        Ok(())
    }
} 