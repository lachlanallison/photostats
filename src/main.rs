extern crate clap;
use clap::Parser;
use std::{fs};
use std::path::{PathBuf};
use std::time::Instant;
use image;
use thousands::Separable;
use std::cmp::Reverse;

#[derive(Parser,Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Name of the path to walk
    #[arg(short, long, default_value_t = String::from("."))]
    path: String
}

#[derive(Debug)]
struct Img {
    width: u32,
    length: u32,
    total_pixels: u32,
    name: String,
}

#[derive(Debug)]
struct ImgTotals {
    total_pixels: u64,
    filecount: u32,
    photocount: u32,
    largest: String,
    smallest: String,
    widest: String,
    tallest: String,
}

const PRO_XDR_PX: f64 = 20358144.0;
const UHD_PX: f64 = 8294400.0;
const HD_PX: f64 = 2073600.0;

fn print_results (totals: &ImgTotals) {
    let total_pixels_f = totals.total_pixels as f64;
    println!("===== ===== ===== ===== ===== ===== ===== ===== ===== ===== =====\n");
    println!("Total amount of files: {}", totals.filecount.separate_with_commas());
    println!("Total amount of photos: {}", totals.photocount.separate_with_commas());
    println!("Total pixels: {}\n", totals.total_pixels.separate_with_commas());
    println!("How many Pro XDR displays does your photo collection take up: {:?}", total_pixels_f / PRO_XDR_PX);
    println!("How many 4K UHD displays does your photo collection take up: {:?}", total_pixels_f / UHD_PX);
    println!("How many Full HD displays does your photo collection take up: {:?}\n", total_pixels_f / HD_PX);
    println!("Largest photo is: {}", totals.largest);
    println!("Smallest photo is: {}", totals.smallest);
    println!("Widest photo is: {}", totals.widest);
    println!("Tallest photo is: {}\n", totals.tallest);
}

fn analyse(images: &mut Vec<Img>, totals: &mut ImgTotals) {
    images.sort_by_key(|k| (k.total_pixels, Reverse(k.name.clone())));
    totals.smallest = images.first().unwrap().name.clone();
    totals.largest = images.last().unwrap().name.clone();
    
    images.sort_by_key(|k| (k.width, k.name.clone()));
    totals.widest = images.last().unwrap().name.clone();

    images.sort_by_key(|k| (k.length, k.name.clone()));
    totals.tallest = images.last().unwrap().name.clone();
}

fn image_process(image: &PathBuf, totals: &mut ImgTotals) -> Img {

    let filename = image.clone().into_os_string().into_string().expect("Filename is not a string!");

    // println!("is an image: {:?}", image);
    let crnt_img_dim = image::image_dimensions(image);
    
    let crnt_img_dim = match crnt_img_dim {
        Ok(crnt_img_dim) => crnt_img_dim,
        Err(_) => (0, 0),
    };
    let img_details = Img{width: crnt_img_dim.0, length: crnt_img_dim.1, total_pixels: crnt_img_dim.0 * crnt_img_dim.1,  name: filename};
    
    // println!("{:?}", img_details);
    totals.total_pixels += u64::from(img_details.total_pixels);
    totals.photocount += 1;
    // println!("{:?}", totals);
    img_details
}

fn crawl_dir (folder: &PathBuf, images: &mut Vec<Img>, totals: &mut ImgTotals)  {
    if let Ok(entries) = fs::read_dir(&folder) {
        for entry in entries {
            if let Ok(entry) = entry {

                // check if entry is a file
                let metadata = fs::metadata(entry.path()).unwrap();

                if metadata.file_type().is_file() {

                    // skip hidden files
                    if entry.file_name().into_string().unwrap().starts_with(".") {
                        continue
                    }
                    // check if file has an image extension
                    match entry.path().extension().and_then(|ext| ext.to_str().map(|s| s.to_ascii_lowercase())) {
                        Some(ext) if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "tif" | "tiff" | "bmp") => {
                            images.push(image_process(&entry.path(), totals));
                        }
                        _ => (),
                    }
                    
                    totals.filecount += 1;

                } else {
                    // recurse through next folder
                    crawl_dir(&entry.path(), images, totals);
                }
            }
        }
    }
}

fn main() {
    let now = Instant::now();
    println!(r"
         _           _            _        _       
        | |         | |          | |      | |      
   _ __ | |__   ___ | |_ ___  ___| |_ __ _| |_ ___ 
  | '_ \| '_ \ / _ \| __/ _ \/ __| __/ _` | __/ __|
  | |_) | | | | (_) | || (_) \__ \ || (_| | |_\__ \
  | .__/|_| |_|\___/ \__\___/|___/\__\__,_|\__|___/
  | |                                              
  |_|                                              
        ");

    let args = Arguments::parse();

    println!("Scanning {}", args.path);
  
    let dir_path = PathBuf::from(args.path);
    let mut images: Vec<Img> = Vec::new();
    let mut image_totals = ImgTotals{total_pixels: 0, photocount: 0, filecount: 0, largest: String::from(""), smallest: String::from(""), widest: String::from(""), tallest: String::from("")};

    crawl_dir(&dir_path, &mut images, &mut image_totals);

    if images.is_empty() { 
        println!("No photos found, please run or point to a directory with photos.");
    } else {
        analyse(&mut images, &mut image_totals);

        print_results(&image_totals);
    }

    if image_totals.photocount == 0 {
        image_totals.photocount += 1;
    }

    let elapsed = now.elapsed();
    let timeperphoto = elapsed / image_totals.photocount;
    println!("Running photostats took {:.2?}", elapsed);
    println!("Time to process each photo {:.2?}", timeperphoto);

    
}
#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use std::fs;
    use tempfile::Builder;

    fn create_test_image(width: u32, height: u32) -> (PathBuf, String) {
        // Create a temporary file
        let file = Builder::new()
            .suffix(".png")
            .tempfile()
            .expect("Failed to create temp file");
        let file_path = file.path().to_path_buf();
        
        // Keep the file from being deleted when the TempFile is dropped
        file.persist(&file_path).expect("Failed to persist temp file");
        
        let file_name = file_path
            .clone()
            .into_os_string()
            .into_string()
            .expect("Failed to get filename");

        // Create a test image with u8 RGB values (0-255)
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |_, _| {
            Rgb([255u8, 255u8, 255u8])
        });
        
        // Save the image
        img.save(&file_path).expect("Failed to save test image");
        
        // Verify the image exists and can be read
        assert!(file_path.exists(), "Test image file does not exist after creation");
        let dimensions = image::image_dimensions(&file_path)
            .expect("Failed to read test image dimensions");
        assert_eq!(dimensions, (width, height), "Test image dimensions don't match expected values");
        
        (file_path, file_name)
    }

    #[test]
    fn test_image_process() {
        // Create a test image
        let (image_path, image_name) = create_test_image(100, 200);
        
        // Verify the image exists before processing
        assert!(image_path.exists(), "Test image doesn't exist before processing");
        
        let mut totals = ImgTotals {
            total_pixels: 0,
            filecount: 0,
            photocount: 0,
            largest: String::new(),
            smallest: String::new(),
            widest: String::new(),
            tallest: String::new(),
        };

        // Process the image
        let result = image_process(&image_path, &mut totals);

        // Print debug information
        println!("Test image path: {:?}", image_path);
        println!("Test image dimensions from result: {}x{}", result.width, result.length);
        
        // Verify results
        assert_eq!(result.width, 100, "Image width doesn't match expected value");
        assert_eq!(result.length, 200, "Image length doesn't match expected value");
        assert_eq!(result.total_pixels, 20000, "Total pixels don't match expected value");
        assert_eq!(result.name, image_name, "Image name doesn't match expected value");
        
        // Verify totals were updated correctly
        assert_eq!(totals.total_pixels, 20000, "Total pixels in ImgTotals doesn't match expected value");
        assert_eq!(totals.photocount, 1, "Photo count doesn't match expected value");

        // Clean up
        fs::remove_file(image_path).expect("Failed to delete test image");
    }

    #[test]
    fn test_image_process_invalid_image() {
        // Create a path to a non-existent image
        let invalid_path = PathBuf::from("nonexistent.png");
        let mut totals = ImgTotals {
            total_pixels: 0,
            filecount: 0,
            photocount: 0,
            largest: String::new(),
            smallest: String::new(),
            widest: String::new(),
            tallest: String::new(),
        };

        // Process the invalid image
        let result = image_process(&invalid_path, &mut totals);

        // Verify results for invalid image
        assert_eq!(result.width, 0);
        assert_eq!(result.length, 0);
        assert_eq!(result.total_pixels, 0);
        assert_eq!(result.name, "nonexistent.png");
        
        // Verify totals were updated correctly
        assert_eq!(totals.photocount, 1);
        assert_eq!(totals.total_pixels, 0);
    }

    #[test]
    fn test_analyse() {
        // Create a vector of test images with different dimensions
        let mut images = vec![
            Img {
                width: 100,
                length: 200,
                total_pixels: 20000,
                name: "medium.jpg".to_string(),
            },
            Img {
                width: 50,
                length: 50,
                total_pixels: 2500,
                name: "smallest.jpg".to_string(),
            },
            Img {
                width: 300,
                length: 400,
                total_pixels: 120000,
                name: "largest.jpg".to_string(),
            },
            Img {
                width: 500,
                length: 100,
                total_pixels: 50000,
                name: "widest.jpg".to_string(),
            },
            Img {
                width: 200,
                length: 600,
                total_pixels: 120000,
                name: "tallest.jpg".to_string(),
            },
        ];
    
        let mut totals = ImgTotals {
            total_pixels: 0,
            filecount: 0,
            photocount: 0,
            largest: String::new(),
            smallest: String::new(),
            widest: String::new(),
            tallest: String::new(),
        };
    
        // Run the analysis
        analyse(&mut images, &mut totals);
    
        // Verify the results
        assert_eq!(totals.smallest, "smallest.jpg", "Incorrect smallest image");
        // For equal total pixels, will use alphabetical ordering as tiebreaker
        assert_eq!(totals.largest, "largest.jpg", "Incorrect largest image (should be 'largest.jpg' due to alphabetical tiebreaker)");
        assert_eq!(totals.widest, "widest.jpg", "Incorrect widest image");
        assert_eq!(totals.tallest, "tallest.jpg", "Incorrect tallest image");
    
    }
    // Additional test case for ties
    #[test]
    fn test_analyse_with_ties() {
        let mut images = vec![
            Img {
                width: 300,
                length: 400,
                total_pixels: 120000,
                name: "b_large.jpg".to_string(),
            },
            Img {
                width: 400,
                length: 300,
                total_pixels: 120000,
                name: "a_large.jpg".to_string(),
            },
        ];

        let mut totals = ImgTotals {
            total_pixels: 0,
            filecount: 0,
            photocount: 0,
            largest: String::new(),
            smallest: String::new(),
            widest: String::new(),
            tallest: String::new(),
        };

        analyse(&mut images, &mut totals);

        // When total pixels are equal, should use alphabetical order
        assert_eq!(totals.largest, "a_large.jpg", "Tiebreaker should use alphabetical order");

    }
}