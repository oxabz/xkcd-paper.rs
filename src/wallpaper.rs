use crate::utils::ResultExt;
use std::process::Stdio;
use std::path::Path;
use tokio::process::{Command, Child};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WallpaperError{
    #[error("Couldnt find feh check if it's in your PATH")]
    FehNotFound,
    #[error("Error running whereis")]
    WhereIsError,
    #[error("Error running feh")]
    FehError,
    #[error("Error piping the picture")]
    PipeError
}

async fn check_feh()-> Result<(),WallpaperError>{
    let mut child = Command::new("whereis")
        .stdout(Stdio::piped())
        .args(vec!["feh"])
        .spawn().replace_err(WallpaperError::WhereIsError)?;
    let mut res = String::new();
    child.stdout.take().unwrap().read_to_string(&mut res).await;
    child.wait().await.replace_err(WallpaperError::WhereIsError)?;
    // Note to self : beurk
    if res.len() > 10 {
        Ok(())
    }else {
        Err(WallpaperError::FehNotFound)
    }
}

pub async fn set_wallpaper(image:&Vec<u8>) -> Result<Child, WallpaperError>{
    check_feh().await?;
    let mut child = Command::new("feh")
        .stdin(Stdio::piped())
        .args(vec!["--bg-center","-"])
        .spawn().replace_err(WallpaperError::FehError)?;
    child.stdin.take().unwrap().write_all(image).await.replace_err(WallpaperError::PipeError)?;
    Ok(child)
}