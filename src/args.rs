use crate::args::XkcdMode::{Random, Last, Nth};
use std::str::FromStr;
use args::Args;
use image::Rgba;
use crate::utils::ResultExt;
use getopts::Occur;

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

pub fn setup_args() -> anyhow::Result<Args> {
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

pub fn get_args(args: &Args) -> anyhow::Result<(XkcdMode, (usize,usize), (usize,usize), Rgba<u8>, Rgba<u8>)>{
    let mode = args.value_of::<XkcdMode>("mode")?;
    let size = get_size(args)?;
    let padding = get_padding(args)?;
    let fg = get_colors(args , "foreground")?;
    let bg = get_colors(args , "background")?;
    Ok((mode,size,padding,fg,bg))
}