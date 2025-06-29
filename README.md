# Rust In-Memory Database with Persistent Storage and Hash Indexing

A simple yet extensible in-memory key-value database written in Rust with:

✅ JSON-based storage
✅ Persistent disk saving
✅ Automatic backup support
✅ Custom Hash Indexing for efficient lookups
✅ Interactive shell interface

---

## Features

* In-memory key-value store using `HashMap`
* Optional persistent storage to JSON file
* Hash-based indexing for efficient data retrieval
* Backup and restore functionality
* Interactive shell interface to manage the database
* Uses `serde_json` for flexible JSON value storage
* Hash indexing powered by SHA-256
* Simple and extensible design

---

## Requirements

* Rust (Edition 2021 recommended)
* Cargo (Rust package manager)

---

## Getting Started

### 1. Clone the Repository

```
git clone https://github.com/Akim-m/redru
cd your-repo-name
```

### 2. Build the Project

```
cargo build --release
```

### 3. Run the Interactive Shell

```
cargo run
```

---

## Shell Commands

Inside the interactive shell, you can use the following commands:

| Command             | Description                       |
| ------------------- | --------------------------------- |
| `set <key> <value>` | Insert or update a key-value pair |
| `get <key>`         | Retrieve the value for a key      |
| `delete <key>`      | Remove a key-value pair           |
| `list`              | List all keys                     |
| `index <field>`     | Create a hash index on a field    |
| `save`              | Manually save the database        |
| `backup`            | Create a backup of the database   |
| `help`              | Show help menu                    |
| `exit`              | Exit the interactive shell        |

---

## Project Structure

```
├── main.rs         # Entry point and interactive shell
├── db.rs           # Core in-memory database implementation
├── hash_index.rs   # Hash index logic
├── Cargo.toml      # Rust package configuration
└── database.json   # (Generated) Persistent storage file
```

---

## Example Usage

```
> set name "Alice"
> get name
"Alice"

> index name
> save
> backup
> exit
```

---

## Testing

To run tests (if implemented):

```
cargo test
```

---

## Future Improvements

* Field-level indexing for JSON objects
* Support for advanced queries
* Multi-threaded read/write
* Backup scheduling
* Command history in shell

---

## Contributing

Pull requests are welcome. Please open an issue to discuss improvements or report bugs.

---

## License

MIT

---

**Made with Rust**
