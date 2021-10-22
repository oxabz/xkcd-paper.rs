#![deny(warnings)]
mod utils;
mod wallpaper;
mod xkcd;
mod caching;

use crate::utils::ResultExt;
use crate::wallpaper::set_wallpaper;
use crate::XkcdMode::{Last, Nth, Random};
use args::Args;
use getopts::Occur;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgba};
use rand::Rng;
use std::io::Cursor;
use std::str::FromStr;
use thiserror::Error;

const PROGRAM_DESC: &str = "Run this program";
const PROGRAM_NAME: &str = "program";

#[derive(Debug)]
enum XkcdMode {
    Random,
    Last,
    Nth(usize),
}

impl FromStr for XkcdMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "random" => Ok(Random),
            "last" => Ok(Last),
            _ => match s.parse::<usize>() {
                Ok(nth) => Ok(Nth(nth)),
                Err(_) => Err(()),
            },
        }
    }
}

#[derive(Error, Debug)]
enum ArgumentError {
    #[error("Couldn't parse the size or the paddding")]
    SizePaddingParseError,
    #[error("Couldn't parse the colors")]
    ColorParseError,
}

fn setup_args() -> anyhow::Result<Args> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);
    args.flag("h", "help", "Print the usage menu");
    args.option(
        "m",
        "mode",
        "xkcd selection",
        "random/last/<number>",
        Occur::Optional,
        Some("random".to_string()),
    );
    args.option(
        "s",
        "size",
        "screen size (in case you have more than one screen I'd recommend to set it to your biggest)",
        "<width>x<height>",
        Occur::Optional,
        Some("1366x768".to_string()),
    );
    args.option(
        "p",
        "padding",
        "padding around the screen (in case you have more than one screen I'd recommend setting it to half the size difference between your screens + a bit of additional padding)",
        "<horizontal>:<vertical>",
        Occur::Optional,
        Some("20:20".to_string()),
    );
    args.option(
        "f",
        "foreground",
        "foreground color",
        "RRGGBB",
        Occur::Optional,
        Some("4ECDC4".to_string()),
    );
    args.option(
        "b",
        "background",
        "background color",
        "RRGGBB",
        Occur::Optional,
        Some("002A32".to_string()),
    );
    args.parse(std::env::args())?;
    Ok(args)
}

fn get_size(args: &Args) -> anyhow::Result<(usize, usize)> {
    let raw: String = args.value_of("size")?;
    let splited = raw.split('x').collect::<Vec<_>>();
    if let (Some(width), Some(height)) = (splited.get(0), splited.get(1)) {
        Ok((width.parse::<usize>()?, height.parse::<usize>()?))
    } else {
        Err(ArgumentError::SizePaddingParseError.into())
    }
}

fn get_padding(args: &Args) -> anyhow::Result<(usize, usize)> {
    let raw: String = args.value_of("padding")?;
    let splited = raw.split(':').collect::<Vec<_>>();
    if let (Some(horizontal), Some(vertical)) = (splited.get(0), splited.get(1)) {
        Ok((horizontal.parse::<usize>()?, vertical.parse::<usize>()?))
    } else {
        Err(ArgumentError::SizePaddingParseError.into())
    }
}

fn get_colors(args: &Args, param: &str) -> anyhow::Result<Rgba<u8>> {
    let raw: String = args.value_of(param)?;
    let parsed = hex::decode(raw).replace_err(ArgumentError::ColorParseError)?;
    if let (Some(r), Some(g), Some(b)) = (parsed.get(0), parsed.get(1), parsed.get(2)) {
        Ok(Rgba([*r, *g, *b, 255]))
    } else {
        Err(ArgumentError::ColorParseError.into())
    }
}

fn get_args(args: &Args) -> anyhow::Result<(XkcdMode, (usize,usize), (usize,usize), Rgba<u8>, Rgba<u8>)>{
    let mode = args.value_of::<XkcdMode>("mode")?;
    let size = get_size(args)?;
    let padding = get_padding(args)?;
    let fg = get_colors(args , "foreground")?;
    let bg = get_colors(args , "background")?;
    Ok((mode,size,padding,fg,bg))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    //////////////////////////////////////////////////////////////////////
    ///////////////////////// Parsing Args ///////////////////////////////
    //////////////////////////////////////////////////////////////////////

    let args = match setup_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Args error : {}", err);
            std::process::exit(0);
        }
    };

    // Printing the help and exiting
    let help = args.value_of::<bool>("help").unwrap_or(false);
    if help {
        println!("{}", args.full_usage());
        std::process::exit(0);
    }

    let (mode, (width, height), padding, fg, bg) = match get_args(&args) {
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
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
        return;
    }

    //////////////////////////////////////////////////////////////////////
    //////////////////////// loading the xkcd ////////////////////////////
    //////////////////////////////////////////////////////////////////////

    let image = match xkcd::get_xkcd_img(num).await {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Error querying the xkcd comic image url : {}", err);
            return;
        }
    };

    let img = ImageReader::new(Cursor::new(image))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    // Grayscale
    let mut img = img.grayscale();

    // Invert
    img.invert();

    // Resize
    let ratio_width = (width - padding.0) as f32 / img.width() as f32;
    let ratio_height = (height - padding.1) as f32 / img.height() as f32;
    let ratio = ratio_height.min(ratio_width).min(1.0);
    img.resize(
        (img.width() as f32 * ratio) as u32,
        (img.height() as f32 * ratio) as u32,
        FilterType::Triangle,
    );

    //Composition
    let mut canva = DynamicImage::new_rgba8(width as u32, height as u32);
    if let Err(err) = canva.copy_from(
        &img,
        (width as u32 - img.width()) / 2,
        (height as u32 - img.height()) / 2,
    ) {
        eprintln!("Image copy failed : {}", err);
        return;
    }

    //Coloring
    let mut canva = canva.into_rgba8();
    canva.pixels_mut().for_each(|pix| {
        let grey = (pix.0[0] as f32 + pix.0[1] as f32 + pix.0[2] as f32) / 3.0 / 256.0;
        *pix = fg.map2(&bg, |x, y| {
            (x as f32 * grey + y as f32 * (1.0 - grey)) as u8
        });
    });
    let canva = DynamicImage::ImageRgba8(canva);
    let mut image: Vec<u8> = Vec::new();
    canva
        .write_to(&mut image, image::ImageOutputFormat::Png)
        .unwrap();

    //Setting the wallpaper using feh
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
