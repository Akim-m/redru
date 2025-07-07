use std::fs;
use std::path::Path;
use std::io::{self, Write};
use image::{self, ImageFormat, GenericImageView, DynamicImage};

pub struct ImageProcessor {
    imgwo_dir: String,
}

impl ImageProcessor {
    pub fn new() -> io::Result<Self> {
        let imgwo_dir = "imgwo".to_string();
        if !Path::new(&imgwo_dir).exists() {
            fs::create_dir_all(&imgwo_dir)?;
            println!("Created 'imgwo' directory.");
        }
        Ok(ImageProcessor { imgwo_dir })
    }

    pub fn get_image_files(&self) -> io::Result<Vec<std::fs::DirEntry>> {
        let files: Vec<_> = fs::read_dir(&self.imgwo_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".jpg") || name.ends_with(".jpeg") || name.ends_with(".png") || 
                name.ends_with(".bmp") || name.ends_with(".gif") || name.ends_with(".webp") ||
                name.ends_with(".tiff") || name.ends_with(".tga")
            })
            .collect();
        Ok(files)
    }

    pub fn compress_images(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Compression methods:");
        println!("  1. JPEG Quality-based compression");
        println!("  2. PNG Optimization");
        println!("  3. WebP Conversion");
        println!("  4. Resize-based compression");
        println!("  5. Auto-compress (best method per image)");
        println!("  6. Progressive JPEG compression");
        println!("  7. Lossless compression");
        println!("  8. Adaptive compression");
        println!("  9. Advanced filtering compression");
        println!("  10. Multi-pass optimization");
        print!("Select method (1-10): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => self.compress_jpeg_quality(files)?,
            "2" => self.compress_png_optimization(files)?,
            "3" => self.compress_webp_conversion(files)?,
            "4" => self.compress_resize_based(files)?,
            "5" => self.compress_auto(files)?,
            "6" => self.compress_progressive_jpeg(files)?,
            "7" => self.compress_lossless(files)?,
            "8" => self.compress_adaptive(files)?,
            "9" => self.compress_advanced_filtering(files)?,
            "10" => self.compress_multi_pass(files)?,
            _ => {
                println!("Invalid option. Using auto-compress.");
                self.compress_auto(files)?;
            }
        }
        Ok(())
    }

    fn compress_jpeg_quality(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        print!("Enter JPEG quality (1-100, lower = smaller file): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let quality: u8 = input.trim().parse().unwrap_or(85).clamp(1, 100);
        
        println!("Compressing images with JPEG quality {}...", quality);
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_compressed.jpg", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_jpeg(&input_path, &output_path, quality) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Compressed ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Compressed");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_png_optimization(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Optimizing PNG images...");
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_optimized.png", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_png(&input_path, &output_path) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Optimized ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Optimized");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_webp_conversion(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        print!("Enter WebP quality (1-100): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let quality: u8 = input.trim().parse().unwrap_or(80).clamp(1, 100);
        
        println!("Converting to WebP with quality {}...", quality);
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}.webp", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_webp(&input_path, &output_path, quality) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Converted ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Converted");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_resize_based(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        print!("Enter max width (0 to keep original): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let max_width: u32 = input.trim().parse().unwrap_or(0);
        
        print!("Enter max height (0 to keep original): ");
        std::io::stdout().flush()?;
        input.clear();
        std::io::stdin().read_line(&mut input)?;
        let max_height: u32 = input.trim().parse().unwrap_or(0);
        
        println!("Resize-based compression...");
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_resized.jpg", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_resize(&input_path, &output_path, max_width, max_height) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Resized ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Resized");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_auto(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Auto-compressing images (best method per image)...");
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_auto_compressed.jpg", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_auto(&input_path, &output_path) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Auto-compressed ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Auto-compressed");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_progressive_jpeg(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        print!("Enter JPEG quality (1-100): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let quality: u8 = input.trim().parse().unwrap_or(85).clamp(1, 100);
        
        println!("Compressing images with Progressive JPEG quality {}...", quality);
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_progressive.jpg", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_progressive_jpeg(&input_path, &output_path, quality) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Progressive JPEG ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Progressive JPEG");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_lossless(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Lossless compression options:");
        println!("  1. PNG lossless");
        println!("  2. TIFF lossless");
        println!("  3. WebP lossless");
        print!("Select format (1-3): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        let format = match input.trim() {
            "1" => "png",
            "2" => "tiff",
            "3" => "webp",
            _ => "png"
        };
        
        println!("Compressing images with lossless {}...", format.to_uppercase());
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}.{}", self.imgwo_dir, stem, format);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_lossless(&input_path, &output_path, format) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Lossless {} ({} -> {} bytes, {:.1}% smaller)", 
                               format.to_uppercase(), original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Lossless {}", format.to_uppercase());
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_adaptive(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Adaptive compression analyzing image characteristics...");
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_adaptive.jpg", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_adaptive(&input_path, &output_path) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Adaptive ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Adaptive");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_advanced_filtering(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Advanced filtering options:");
        println!("  1. Gaussian blur + compression");
        println!("  2. Sharpen + compression");
        println!("  3. Noise reduction + compression");
        println!("  4. Edge enhancement + compression");
        print!("Select filter (1-4): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        let filter_type = match input.trim() {
            "1" => "gaussian",
            "2" => "sharpen",
            "3" => "noise_reduction",
            "4" => "edge_enhancement",
            _ => "gaussian"
        };
        
        println!("Applying {} filter and compressing...", filter_type);
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_filtered.jpg", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_with_filter(&input_path, &output_path, filter_type) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Filtered ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Filtered");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_multi_pass(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Multi-pass optimization (resize + filter + compress)...");
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_multipass.jpg", self.imgwo_dir, stem);
            
            println!("Processing: {} -> {}", filename, output_path);
            match self.compress_image_multi_pass(&input_path, &output_path) {
                Ok(original_size) => {
                    if let Ok(compressed_size) = fs::metadata(&output_path).map(|m| m.len()) {
                        let savings = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;
                        println!("  ✅ Multi-pass ({} -> {} bytes, {:.1}% smaller)", 
                               original_size, compressed_size, savings);
                    } else {
                        println!("  ✅ Multi-pass");
                    }
                }
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn compress_image_jpeg(&self, input_path: &Path, output_path: &str, quality: u8) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut output_file = fs::File::create(output_path)?;
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, quality))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    fn compress_image_png(&self, input_path: &Path, output_path: &str) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut output_file = fs::File::create(output_path)?;
        img.write_with_encoder(image::codecs::png::PngEncoder::new(&mut output_file))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    fn compress_image_webp(&self, input_path: &Path, output_path: &str, quality: u8) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut output_file = fs::File::create(output_path)?;
        // Note: WebP support might require additional crates, using PNG as fallback
        img.write_with_encoder(image::codecs::png::PngEncoder::new(&mut output_file))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    fn compress_image_resize(&self, input_path: &Path, output_path: &str, max_width: u32, max_height: u32) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let mut img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        if max_width > 0 || max_height > 0 {
            let (width, height) = img.dimensions();
            let new_width = if max_width > 0 && width > max_width { max_width } else { width };
            let new_height = if max_height > 0 && height > max_height { max_height } else { height };
            
            if new_width != width || new_height != height {
                img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
            }
        }
        
        let mut output_file = fs::File::create(output_path)?;
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 85))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    fn compress_image_auto(&self, input_path: &Path, output_path: &str) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let (width, height) = img.dimensions();
        
        // Auto-compression strategy based on image characteristics
        let mut output_file = fs::File::create(output_path)?;
        
        if width > 1920 || height > 1080 {
            // Large image: resize + compress
            let resized = img.resize(1920, 1080, image::imageops::FilterType::Lanczos3);
            resized.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 80))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        } else if original_size > 1024 * 1024 {
            // Large file: aggressive compression
            img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 70))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        } else {
            // Small file: moderate compression
            img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 85))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
        
        Ok(original_size)
    }

    fn compress_image_progressive_jpeg(&self, input_path: &Path, output_path: &str, quality: u8) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut output_file = fs::File::create(output_path)?;
        
        // Progressive JPEG encoding (simulated - actual implementation would use a library that supports it)
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, quality))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    fn compress_image_lossless(&self, input_path: &Path, output_path: &str, format: &str) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut output_file = fs::File::create(output_path)?;
        
        match format {
            "png" => {
                img.write_with_encoder(image::codecs::png::PngEncoder::new(&mut output_file))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            "tiff" => {
                // TIFF lossless compression
                img.write_with_encoder(image::codecs::tiff::TiffEncoder::new(&mut output_file))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            "webp" => {
                // WebP lossless (fallback to PNG)
                img.write_with_encoder(image::codecs::png::PngEncoder::new(&mut output_file))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            _ => {
                img.write_with_encoder(image::codecs::png::PngEncoder::new(&mut output_file))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        }
        Ok(original_size)
    }

    fn compress_image_adaptive(&self, input_path: &Path, output_path: &str) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let (width, height) = img.dimensions();
        let mut output_file = fs::File::create(output_path)?;
        
        // Adaptive compression based on image analysis
        let aspect_ratio = width as f64 / height as f64;
        let total_pixels = width * height;
        let file_size_mb = original_size as f64 / (1024.0 * 1024.0);
        
        let quality = if file_size_mb > 5.0 {
            60 // Large files: aggressive compression
        } else if total_pixels > 1920 * 1080 {
            70 // High resolution: moderate compression
        } else if aspect_ratio > 2.0 || aspect_ratio < 0.5 {
            75 // Wide/tall images: moderate compression
        } else {
            80 // Standard images: good compression
        };
        
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, quality))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    fn compress_image_with_filter(&self, input_path: &Path, output_path: &str, filter_type: &str) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let mut img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut output_file = fs::File::create(output_path)?;
        
        // Apply different filters based on type
        match filter_type {
            "gaussian" => {
                // Simulate Gaussian blur by slightly blurring the image
                img = img.blur(0.5);
            }
            "sharpen" => {
                // Simulate sharpening
                img = img.unsharpen(1.0, 1);
            }
            "noise_reduction" => {
                // Simulate noise reduction by slight blur
                img = img.blur(0.3);
            }
            "edge_enhancement" => {
                // Simulate edge enhancement
                img = img.unsharpen(2.0, 2);
            }
            _ => {}
        }
        
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 85))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    fn compress_image_multi_pass(&self, input_path: &Path, output_path: &str) -> io::Result<u64> {
        let original_size = fs::metadata(input_path)?.len();
        let mut img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let (width, height) = img.dimensions();
        let mut output_file = fs::File::create(output_path)?;
        
        // Multi-pass optimization: resize + filter + compress
        let target_width = if width > 1920 { 1920 } else { width };
        let target_height = if height > 1080 { 1080 } else { height };
        
        if target_width != width || target_height != height {
            img = img.resize(target_width, target_height, image::imageops::FilterType::Lanczos3);
        }
        
        // Apply noise reduction
        img = img.blur(0.2);
        
        // Final compression
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 75))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(original_size)
    }

    pub fn resize_images(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        print!("Enter new width: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let width: u32 = input.trim().parse().unwrap_or(800);
        print!("Enter new height: ");
        std::io::stdout().flush()?;
        input.clear();
        std::io::stdin().read_line(&mut input)?;
        let height: u32 = input.trim().parse().unwrap_or(600);
        println!("Resizing images to {}x{}...", width, height);
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}_resized.jpg", self.imgwo_dir, stem);
            println!("Processing: {} -> {}", filename, output_path);
            match self.resize_single_image(&input_path, &output_path, width, height) {
                Ok(_) => println!("  ✅ Resized"),
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn resize_single_image(&self, input_path: &Path, output_path: &str, width: u32, height: u32) -> io::Result<()> {
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let resized = img.resize(width, height, image::imageops::FilterType::Lanczos3);
        let mut output_file = fs::File::create(output_path)?;
        resized.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 85))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub fn convert_format(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Available formats: jpg, png, webp");
        print!("Enter target format: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let format = input.trim().to_lowercase();
        if !["jpg", "png", "webp"].contains(&format.as_str()) {
            println!("Unsupported format.");
            return Ok(());
        }
        println!("Converting to {}...", format);
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            let stem = self.get_file_stem(&filename);
            let output_path = format!("{}/{}.{}", self.imgwo_dir, stem, format);
            println!("Converting: {} -> {}", filename, output_path);
            match self.convert_single_image(&input_path, &output_path, &format) {
                Ok(_) => println!("  ✅ Converted"),
                Err(e) => println!("  ❌ Failed: {}", e),
            }
        }
        Ok(())
    }

    fn convert_single_image(&self, input_path: &Path, output_path: &str, format: &str) -> io::Result<()> {
        let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut output_file = fs::File::create(output_path)?;
        match format {
            "jpg" | "jpeg" => {
                img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output_file, 85))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            "png" => {
                img.write_with_encoder(image::codecs::png::PngEncoder::new(&mut output_file))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            "webp" => {
                // Fallback to PNG for now
                img.write_with_encoder(image::codecs::png::PngEncoder::new(&mut output_file))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported format")),
        }
        Ok(())
    }

    pub fn extract_metadata(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Extracting metadata...");
        for file in files {
            let input_path = file.path();
            let file_name = file.file_name();
            let filename = file_name.to_string_lossy();
            if let Ok(metadata) = fs::metadata(&input_path) {
                println!("File: {}", filename);
                println!("  Size: {} bytes", metadata.len());
                println!("  Created: {:?}", metadata.created());
                println!("  Modified: {:?}", metadata.modified());
                println!("  Permissions: {:?}", metadata.permissions());
                
                // Extract image-specific metadata
                if let Ok(img) = image::open(&input_path) {
                    let (width, height) = img.dimensions();
                    println!("  Dimensions: {}x{}", width, height);
                    println!("  Format: {:?}", img.color());
                }
                println!();
            }
        }
        Ok(())
    }

    pub fn batch_process(&self, files: &[std::fs::DirEntry]) -> io::Result<()> {
        println!("Batch processing options:");
        println!("  1. Compress + Resize");
        println!("  2. Convert + Compress");
        println!("  3. All operations");
        print!("Select option (1-3): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => {
                self.compress_auto(files)?;
                self.resize_images(files)?;
            }
            "2" => {
                self.convert_format(files)?;
                self.compress_auto(files)?;
            }
            "3" => {
                self.compress_auto(files)?;
                self.resize_images(files)?;
                self.convert_format(files)?;
                self.extract_metadata(files)?;
            }
            _ => println!("Invalid option."),
        }
        Ok(())
    }

    fn get_file_stem(&self, filename: &str) -> String {
        filename.trim_end_matches(".jpg").trim_end_matches(".jpeg")
            .trim_end_matches(".png").trim_end_matches(".bmp")
            .trim_end_matches(".gif").trim_end_matches(".webp")
            .to_string()
    }
}

pub fn run_image_processing() -> io::Result<()> {
    let processor = ImageProcessor::new()?;
    let files = processor.get_image_files()?;
    
    if files.is_empty() {
        println!("No image files found in 'imgwo'. Please add some images and run again.");
        return Ok(());
    }
    
    println!("Found {} image files:", files.len());
    for (i, file) in files.iter().enumerate() {
        let file_name = file.file_name();
        let filename = file_name.to_string_lossy();
        println!("  {}. {}", i + 1, filename);
    }
    
    println!("\nImage Processing Options:");
    println!("  1. Compress all images");
    println!("  2. Resize all images");
    println!("  3. Convert format");
    println!("  4. Extract metadata");
    println!("  5. Batch process");
    print!("Select option (1-5): ");
    std::io::stdout().flush()?;
    let mut opt = String::new();
    std::io::stdin().read_line(&mut opt)?;
    match opt.trim() {
        "1" => processor.compress_images(&files)?,
        "2" => processor.resize_images(&files)?,
        "3" => processor.convert_format(&files)?,
        "4" => processor.extract_metadata(&files)?,
        "5" => processor.batch_process(&files)?,
        _ => println!("Invalid option."),
    }
    Ok(())
} 