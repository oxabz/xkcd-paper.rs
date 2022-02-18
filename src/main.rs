#![deny(warnings)]
mod utils;
mod wallpaper;
mod xkcd;
mod caching;
mod args;

use crate::wallpaper::set_wallpaper;
use crate::args::XkcdMode;
use crate::xkcd::get_xkcd_img;
use crate::args::{setup_args, get_args};
use crate::caching::{get_cached_xkcd, cache_xkcd};
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use rand::Rng;
use std::io::Cursor;
use std::process::exit;
use futures::future::TryFutureExt;
use rayon::prelude::*;
use itertools::{Chunk, Itertools};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    //////////////////////////////////////////////////////////////////////
    ///////////////////////// Parsing Args ///////////////////////////////
    //////////////////////////////////////////////////////////////////////

    let args = match setup_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Args error : {}", err);
            exit(1);
        }
    };

    // Printing the help and exiting
    let help = args.value_of::<bool>("help").unwrap_or(false);
    if help {
        println!("{}", args.full_usage());
        exit(0);
    }

    let (mode, (width, height), padding, fg, bg) = match get_args(&args) {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
        Ok(val) => val,
    };


    //////////////////////////////////////////////////////////////////////
    ///////////////////////// picking an xkcd ////////////////////////////
    //////////////////////////////////////////////////////////////////////

    let last = match xkcd::get_last_xkcd().await {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Error querying the last xkcd comic : {}", err);
            return;
        }
    };

    let mut rng = rand::thread_rng();

    let num = match mode {
        XkcdMode::Random => rng.gen_range(1..last),
        XkcdMode::Last => last,
        XkcdMode::Nth(n) => n,
    };

    if num > last && num < 1 {
        eprintln!("{} is not a valid xkcd number", num);
        exit(1);
    }



    //////////////////////////////////////////////////////////////////////
    //////////////////////// loading the xkcd ////////////////////////////
    //////////////////////////////////////////////////////////////////////

    let image = match get_cached_xkcd(num).or_else(|err| {

        eprintln!("{}", err);
        get_xkcd_img(num)
    }).await {
        Ok(img) => {
            if let Err(err) = cache_xkcd(num, img.as_slice()).await {
                eprintln!("{}", err)
            }
            img
        }
        Err(err)=>{
            eprintln!("Could find the corresponding xkcd comic in cache or at xkcd.com : {}", err);
            exit(1);
        }
    };



    //////////////////////////////////////////////////////////////////////
    /////////////////////// processing the image /////////////////////////
    //////////////////////////////////////////////////////////////////////


    let mut img = ImageReader::new(Cursor::new(image))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    // Invert
    img.invert();

    // Resize
    img.resize(
        (width - padding.0) as f32 as u32,
        (height - padding.1) as u32,
        FilterType::Triangle,
    );

    //Composition
    let mut canvas = DynamicImage::new_rgba8(width as u32, height as u32);
    if let Err(err) = canvas.copy_from(
        &img,
        (width as u32 - img.width()) / 2,
        (height as u32 - img.height()) / 2,
    ) {
        eprintln!("Image copy failed : {}", err);
        exit(1);
    }



    // Coloring
    let ffg = [ fg.0[0] as f32, fg.0[1] as f32, fg.0[2] as f32, fg.0[3] as f32 ];
    let fbg = [ bg.0[0] as f32, bg.0[1] as f32, bg.0[2] as f32, bg.0[3] as f32 ];

    let raw = canvas.into_rgba8().into_vec();
    let colored_raw : Vec<_> = raw.into_iter()
        .chunks(4)
        .into_iter()
        .map(Chunk::collect::<Vec<_>>)
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|pixel|{
            let avg = (pixel[0] as i32 + pixel[1] as i32 + pixel[2] as i32)/3;
            let delta = ((pixel[0] as i32 - avg).abs() + (pixel[1] as i32 - avg).abs() + (pixel[2] as i32 - avg).abs())as f32 / 510.;
            let inv_pixel = pixel.iter().map(|x|255-x).collect::<Vec<_>>();
            if delta > 1. {
                println!("delta : {}",&delta);
            }
            let fg = pixel[0] as f32/255.0;
            let bg = 1.0-fg;
            vec![
                ((1.0 - delta)*(ffg[0] * fg + fbg[0] * bg) +  delta * inv_pixel[0] as f32) as u8,
                ((1.0 - delta)*(ffg[1] * fg + fbg[1] * bg) +  delta * inv_pixel[1] as f32) as u8,
                ((1.0 - delta)*(ffg[2] * fg + fbg[2] * bg) +  delta * inv_pixel[2] as f32) as u8,
                ((1.0 - delta)*(ffg[3] * fg + fbg[3] * bg) +  delta * inv_pixel[3] as f32) as u8,
            ]
        }).flatten().collect();

    // Converting back to png
    let canvas = ImageBuffer::from_vec(width as u32, height as u32, colored_raw).unwrap();
    let canvas = DynamicImage::ImageRgba8(canvas);
    let mut image: Vec<u8> = Vec::new();
    canvas
        .write_to(&mut image, image::ImageOutputFormat::Png)
        .unwrap();


    //////////////////////////////////////////////////////////////////////
    ////////////// Setting the wallpaper using feh ///////////////////////
    //////////////////////////////////////////////////////////////////////

    let mut child = match set_wallpaper(&image).await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error setting the wallpaper : {}", err);
            return;
        }
    };
    match child.wait().await {
        Ok(code) => {
            if !code.success() {
                eprintln!(
                    "Error setting the wallpaper : feh exited with code {}",
                    code.code().unwrap()
                );
            };
        }
        Err(_) => {
            eprintln!("Error setting the wallpaper");
        }
    };
}
