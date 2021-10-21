use crate::utils::ResultExt;
use reqwest::get;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
struct Xkcd {
    num: usize,
    img: String,
}

#[derive(Error, Debug)]
pub enum XkcdError {
    #[error("Couldn't reach xkcd.com")]
    CouldntReach,
    #[error("Failed to parse the json response of xkcd.com")]
    JsonParseError,
    #[error("Failed to read the bytes of the xkcd.com picture")]
    BytesParseError,
}

pub async fn get_last_xkcd() -> Result<usize, XkcdError> {
    let response = get("https://xkcd.com/info.0.json")
        .await
        .replace_err(XkcdError::CouldntReach)?;
    let xkcd: Xkcd = response
        .json()
        .await
        .replace_err(XkcdError::JsonParseError)?;
    Ok(xkcd.num)
}

async fn get_xkcd_url(n: usize) -> Result<String, XkcdError> {
    let response = get(&*format!("https://xkcd.com/{}/info.0.json", n))
        .await
        .replace_err(XkcdError::CouldntReach)?;
    let xkcd: Xkcd = response
        .json()
        .await
        .replace_err(XkcdError::JsonParseError)?;
    Ok(xkcd.img)
}

pub async fn get_xkcd_img(n: usize) -> Result<Vec<u8>, XkcdError> {
    let url = get_xkcd_url(n).await?;
    let response = get(url).await.replace_err(XkcdError::CouldntReach)?;
    let bytes = response
        .bytes()
        .await
        .map(|x| x.to_vec())
        .replace_err(XkcdError::BytesParseError)?;
    Ok(bytes)
}
