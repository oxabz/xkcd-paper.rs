use std::env;
use tokio::fs;
use thiserror::Error;
use crate::utils::ResultExt;
use std::path::Path;

const CACHE_PATH: &str = ".cache/xkcd-paper";

#[derive(Error, Debug)]
pub enum CachingError{
    #[error("Couldn't get the $HOME environment variable")]
    NoHomeDir,
    #[error("Couldn't open the file at `{0}`")]
    NotFound(String),
    #[error("Error writing in the cache")]
    WriteError,
    #[error("Couldn't create the directory")]
    DirError
}

fn get_home_dir()->Result<String,CachingError>{
    env::var("HOME").replace_err(CachingError::NoHomeDir)
}

pub async fn cache_xkcd(num: usize, img: &[u8])->Result<(),CachingError>{
    let home = get_home_dir()?;
    let spath = format!("{}/{}/{}.png", home, CACHE_PATH, num);
    let path = Path::new(&spath);
    fs::create_dir_all(path.parent().unwrap()).await.replace_err(CachingError::DirError)?;
    fs::write(path, img).await.map_err(|err|eprintln!("{}", err)).replace_err(CachingError::WriteError)
}

pub async fn get_cached_xkcd(num: usize)->Result<Vec<u8>,CachingError>{
    let home = get_home_dir()?;
    let spath = format!("{}/{}/{}.png", home, CACHE_PATH, num);
    let path = Path::new(&spath);
    fs::read(path).await.replace_err(CachingError::NotFound(spath.clone()))
}