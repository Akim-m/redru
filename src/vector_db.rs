use std::fs;
use std::io::{self, Write, Read};
use serde_json::Value;

pub struct VectorDB {
    vectors: Vec<Vec<f64>>,
    file_path: String,
}

impl VectorDB {
    pub fn new(file_path: &str) -> io::Result<Self> {
        let vectors: Vec<Vec<f64>> = if let Ok(data) = fs::read_to_string(file_path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(VectorDB {
            vectors,
            file_path: file_path.to_string(),
        })
    }

    pub fn add_vector(&mut self, vector: Vec<f64>) -> io::Result<()> {
        if !vector.is_empty() {
            self.vectors.push(vector);
            self.save()?;
        }
        Ok(())
    }

    pub fn query_similar(&self, query: &Vec<f64>, cosine: bool) -> Vec<(usize, f64)> {
        let mut results: Vec<(usize, f64)> = self.vectors.iter().enumerate()
            .filter_map(|(i, v)| {
                if v.len() == query.len() {
                    let dist = if cosine {
                        1.0 - Self::cosine_similarity(v, query)
                    } else {
                        Self::euclidean_distance(v, query)
                    };
                    Some((i, dist))
                } else {
                    None
                }
            })
            .collect();
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results
    }

    pub fn batch_query(&self, queries: &[Vec<f64>], cosine: bool) -> Vec<Vec<(usize, f64)>> {
        queries.iter().map(|q| self.query_similar(q, cosine)).collect()
    }

    pub fn delete_vector(&mut self, index: usize) -> io::Result<()> {
        if index < self.vectors.len() {
            self.vectors.remove(index);
            self.save()?;
        }
        Ok(())
    }

    pub fn list_vectors(&self) -> &Vec<Vec<f64>> {
        &self.vectors
    }

    pub fn save_as_binary(&self, bin_path: &str) -> io::Result<()> {
        let mut file = fs::File::create(bin_path)?;
        for v in &self.vectors {
            let len = v.len() as u64;
            file.write_all(&len.to_le_bytes())?;
            for f in v {
                file.write_all(&f.to_le_bytes())?;
            }
        }
        Ok(())
    }

    pub fn load_from_binary(&mut self, bin_path: &str) -> io::Result<()> {
        let mut file = fs::File::open(bin_path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mut idx = 0;
        let mut loaded = Vec::new();
        while idx + 8 <= buf.len() {
            let len = u64::from_le_bytes(buf[idx..idx+8].try_into().unwrap()) as usize;
            idx += 8;
            let mut v = Vec::new();
            for _ in 0..len {
                if idx + 8 > buf.len() { break; }
                let f = f64::from_le_bytes(buf[idx..idx+8].try_into().unwrap());
                v.push(f);
                idx += 8;
            }
            loaded.push(v);
        }
        self.vectors = loaded;
        self.save()?;
        Ok(())
    }

    fn save(&self) -> io::Result<()> {
        fs::write(&self.file_path, serde_json::to_string_pretty(&self.vectors).unwrap())?;
        Ok(())
    }

    fn euclidean_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
    }

    fn cosine_similarity(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
    }
}

pub fn run_simse() -> io::Result<()> {
    use std::io::{Read, Write};
    let sils_dir = "sils";
    if !std::path::Path::new(sils_dir).exists() {
        fs::create_dir_all(sils_dir)?;
    }
    println!("Drop a file into the 'sils' directory and press Enter when ready...");
    let mut _dummy = String::new();
    std::io::stdin().read_line(&mut _dummy)?;
    let files: Vec<_> = fs::read_dir(sils_dir)?.filter_map(|e| e.ok()).collect();
    if files.is_empty() {
        println!("No file found in 'sils'. Exiting simse mode.");
        return Ok(());
    }
    let file_path = files[0].path();
    println!("Reading file: {}", file_path.display());
    let mut content = String::new();
    fs::File::open(&file_path)?.read_to_string(&mut content)?;
    let vectors: Vec<Vec<f64>> = content
        .lines()
        .map(|line| line.split(',').filter_map(|s| s.trim().parse::<f64>().ok()).collect())
        .collect();
    println!("File converted to vectors:");
    for vec in &vectors {
        println!("{:?}", vec);
    }
    // Save vectors to sils/vectors.json
    let vectors_path = format!("{}/vectors.json", sils_dir);
    let json = serde_json::to_string_pretty(&vectors).unwrap();
    fs::write(&vectors_path, json)?;
    println!("Vectors saved to {}", vectors_path);
    // Vector DB CLI
    vector_db_cli(&vectors_path)?;
    Ok(())
}

fn vector_db_cli(vectors_path: &str) -> io::Result<()> {
    let mut db = VectorDB::new(vectors_path)?;
    loop {
        println!("\nVector DB Options:");
        println!("  1. Add new vector");
        println!("  2. Query similar vectors (Euclidean)");
        println!("  3. Query similar vectors (Cosine)");
        println!("  4. Batch query (Euclidean)");
        println!("  5. List all vectors");
        println!("  6. Delete a vector");
        println!("  7. Save/load as binary");
        println!("  8. Exit");
        print!("Select option (1-8): ");
        std::io::stdout().flush()?;
        let mut opt = String::new();
        std::io::stdin().read_line(&mut opt)?;
        let opt = opt.trim();
        match opt {
            "1" => {
                print!("Enter vector as comma-separated numbers: ");
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let vec: Vec<f64> = input.trim().split(',').filter_map(|s| s.trim().parse().ok()).collect();
                if db.add_vector(vec).is_ok() {
                    println!("Vector added.");
                } else {
                    println!("Invalid vector.");
                }
            }
            "2" => {
                query_vector(&db, false)?;
            }
            "3" => {
                query_vector(&db, true)?;
            }
            "4" => {
                print!("Enter batch of query vectors (one per line, end with empty line):\n");
                std::io::stdout().flush()?;
                let mut batch = Vec::new();
                loop {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    let line = input.trim();
                    if line.is_empty() { break; }
                    let vec: Vec<f64> = line.split(',').filter_map(|s| s.trim().parse().ok()).collect();
                    if !vec.is_empty() { batch.push(vec); }
                }
                let results = db.batch_query(&batch, false);
                for (i, result) in results.iter().enumerate() {
                    println!("\nQuery {}:", i+1);
                    print_top_matches(&db, &batch[i], result);
                }
            }
            "5" => {
                for (i, v) in db.list_vectors().iter().enumerate() {
                    println!("  {}: {:?}", i, v);
                }
            }
            "6" => {
                print!("Enter index of vector to delete: ");
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if let Ok(idx) = input.trim().parse::<usize>() {
                    if db.delete_vector(idx).is_ok() {
                        println!("Vector deleted.");
                    } else {
                        println!("Invalid index.");
                    }
                } else {
                    println!("Invalid input.");
                }
            }
            "7" => {
                println!("  a. Save as binary");
                println!("  b. Load from binary");
                print!("Select (a/b): ");
                std::io::stdout().flush()?;
                let mut sub = String::new();
                std::io::stdin().read_line(&mut sub)?;
                match sub.trim() {
                    "a" => {
                        let bin_path = format!("{}.bin", vectors_path.trim_end_matches(".json"));
                        if db.save_as_binary(&bin_path).is_ok() {
                            println!("Saved to {}", bin_path);
                        }
                    }
                    "b" => {
                        let bin_path = format!("{}.bin", vectors_path.trim_end_matches(".json"));
                        if db.load_from_binary(&bin_path).is_ok() {
                            println!("Loaded from {}", bin_path);
                        }
                    }
                    _ => println!("Invalid option."),
                }
            }
            "8" => break,
            _ => println!("Invalid option."),
        }
    }
    Ok(())
}

fn query_vector(db: &VectorDB, cosine: bool) -> io::Result<()> {
    print!("Enter query vector as comma-separated numbers: ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let query: Vec<f64> = input.trim().split(',').filter_map(|s| s.trim().parse().ok()).collect();
    if query.is_empty() {
        println!("Invalid query vector.");
        return Ok(());
    }
    let results = db.query_similar(&query, cosine);
    print_top_matches(db, &query, &results);
    Ok(())
}

fn print_top_matches(db: &VectorDB, query: &Vec<f64>, results: &[(usize, f64)]) {
    println!("Top 5 closest vectors:");
    for (i, dist) in results.iter().take(5) {
        let vectors = db.list_vectors();
        if *i < vectors.len() {
            println!("  idx {}: {:?} (distance: {:.4})", i, vectors[*i], dist);
        }
    }
}

pub fn run_vector_processing() -> io::Result<()> {
    run_simse()
} 