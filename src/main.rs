use image::{GenericImageView, ImageBuffer, Rgba};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha3::{Digest, Sha3_512};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub fn image_to_rgba(path: &str) -> Result<Vec<[u8; 4]>, image::ImageError> {
    let img = image::open(Path::new(path))?;

    let rgba_img = img.to_rgba8();

    let pixels: Vec<[u8; 4]> = rgba_img
        .pixels()
        .map(|p| [p[0], p[1], p[2], p[3]])
        .collect();

    Ok(pixels)
}

pub async fn encrypt_image(path: &str) -> Result<(), image::ImageError> {
    let img = image::open(Path::new(path))?;
    let (width, height) = img.dimensions();
    let pixels = image_to_rgba(path)?;

    let mut output = Vec::new();
    output.push(format!("{}:{}", width, height));

    let pb = ProgressBar::new(pixels.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)")
            .unwrap(),
    );

    let hashed_pixels: Vec<String> = pixels
        .par_iter()
        .map(|pixel| {
            let pixel_hex = format!(
                "0x{:02X}{:02X}{:02X}{:02X}",
                pixel[0], pixel[1], pixel[2], pixel[3]
            );
            let mut hasher = Sha3_512::new();
            hasher.update(&pixel_hex);
            let hash = format!("{:X}", hasher.finalize());
            pb.inc(1);
            hash
        })
        .collect();

    output.extend(hashed_pixels);
    pb.finish();

    let data = output.join(";");
    let mut file = File::create("encrypted.pain").await?;
    file.write_all(data.as_bytes()).await?;
    println!("Encryption complete! See encrypted.pain");
    Ok(())
}

pub async fn decrypt_image(encrypted_path: &str) -> Result<(), image::ImageError> {
    let mut file = File::open(encrypted_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let parts: Vec<String> = contents.split(';').map(|s| s.to_string()).collect();
    let dims: Vec<&str> = parts[0].split(':').collect();
    let width: u32 = dims[0].parse().unwrap();
    let height: u32 = dims[1].parse().unwrap();

    let img = Arc::new(Mutex::new(ImageBuffer::new(width, height)));
    let multi_progress = Arc::new(MultiProgress::new());

    let pixel_hashes: Vec<String> = parts.iter().skip(1).cloned().collect();
    let chunks: Vec<_> = pixel_hashes
        .chunks((pixel_hashes.len() / num_cpus::get()).max(1))
        .map(|chunk| chunk.to_vec())
        .collect();

    let handles: Vec<_> = chunks
        .into_iter()
        .enumerate()
        .map(|(chunk_id, chunk_hashes)| {
            let img = Arc::clone(&img);
            let mp = Arc::clone(&multi_progress);
            let pb = mp.add(ProgressBar::new(256 * 256 * 256 * 256));

            tokio::spawn(async move {
                for (hash_index, hash) in chunk_hashes.iter().enumerate() {
                    'pixel_search: for r in 0..=255 {
                        for g in 0..=255 {
                            for b in 0..=255 {
                                for a in 0..=255 {
                                    pb.inc(1);
                                    let test_pixel =
                                        format!("0x{:02X}{:02X}{:02X}{:02X}", r, g, b, a);
                                    let mut hasher = Sha3_512::new();
                                    hasher.update(&test_pixel);
                                    let test_hash = format!("{:X}", hasher.finalize());

                                    if test_hash == **hash {
                                        let pixel_index =
                                            chunk_id * chunk_hashes.len() + hash_index;
                                        let x = pixel_index as u32 % width;
                                        let y = pixel_index as u32 / width;
                                        let mut img = img.lock().unwrap();
                                        img.put_pixel(
                                            x,
                                            y,
                                            Rgba([r as u8, g as u8, b as u8, a as u8]),
                                        );
                                        break 'pixel_search;
                                    }
                                }
                            }
                        }
                    }
                }
                pb.finish();
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    let img = Arc::try_unwrap(img).unwrap().into_inner().unwrap();
    img.save("decrypted.png")?;
    println!("Decryption complete! See decrypted.png");
    Ok(())
}
async fn test_pixel_image() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageBuffer::new(10, 10);
    for x in 0..10 {
        for y in 0..10 {
            img.put_pixel(x, y, Rgba([0_u8, 0_u8, 255_u8, 255_u8]));
        }
    }

    img.save("pixels.png").unwrap();

    encrypt_image("pixels.png").await.unwrap();
    println!("Image encrypted to encrypted.pain");

    let mut file = File::open("encrypted.pain").await.unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();
    println!("Encrypted content: {}", contents);

    decrypt_image("encrypted.pain").await.unwrap();
    println!("Image decrypted to decrypted.png");

    Ok(())
}

fn validate_pain_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .map_or(false, |ext| ext == "pain")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Choose action:");
    println!("1. Encrypt image");
    println!("2. Decrypt image");
    println!("3. Run test");
    println!("Enter number (1-3):");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => {
            println!("Enter image path:");
            let mut path = String::new();
            std::io::stdin().read_line(&mut path)?;

            encrypt_image(path.trim()).await?;
        }
        "2" => {
            println!("Enter encrypted file path:");

            let mut path = String::new();
            std::io::stdin().read_line(&mut path)?;
            let path = path.trim();

            if !validate_pain_file(path) {
                println!("Error: File must have .pain extension!");
                return Ok(());
            }

            decrypt_image(path.trim()).await?;
        }
        "3" => {
            println!("Running test...");
            test_pixel_image().await?;
        }
        _ => println!("Invalid option!"),
    }

    Ok(())
}
