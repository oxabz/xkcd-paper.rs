use std::env;
use tokio::fs;
use thiserror::Error;
use crate::utils::ResultExt;
use std::path::Path;

const CACHE_PATH: &str = ".cache/xkcd-paper";

#[derive(Error, Debug)]
enum CachingError{
    NoHomeDir,
    NotFound,
    WriteError
}

fn get_home_dir()->Result<String,CachingError>{
    env::var("HOME").replace_err(CachingError::NoHomeDir)
}

async fn cache_xkcd(num: usize, img: &[u8])->Result<(),CachingError>{
    let home = get_home_dir()?;
    let path = Path::new(&format!("{}/{}/{}", home, CACHE_PATH, num));
    fs::write(path, img).await.replace_err(CachingError::WriteError)
}

async fn get_cached_xkcd(num: usize)->Result<Vec<u8>,CachingError>{
    let home = get_home_dir()?;
    let path = Path::new(&format!("{}/{}/{}", home, CACHE_PATH, num));
    fs::read(path).await.replace_err(CachingError::NotFound)
}