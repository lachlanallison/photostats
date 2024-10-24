extern crate clap;
use clap::Parser;
use std::{fs};
use std::path::{PathBuf};
use std::time::Instant;
use image;
use thousands::Separable;

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
        
        images.sort_by_key(|k| k.total_pixels);
        totals.smallest = images.first().unwrap().name.clone();
        totals.largest = images.last().unwrap().name.clone();
        images.sort_by_key(|k| k.width);
        totals.widest = images.last().unwrap().name.clone();
        images.sort_by_key(|k| k.length);
        totals.tallest = images.last().unwrap().name.clone();
    }

    fn image_process (image: &PathBuf, totals: &mut ImgTotals) -> Img {

        let  filename = image.clone().into_os_string().into_string().expect("Filename is not a string!");

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
}