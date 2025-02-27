use bytes::Bytes;

use crate::error::IndexerError;

const ARWEAVE_HOST: &str = "https://arweave.net";

pub async fn fetch_bundle(tx_id: &str) -> Result<Bytes, IndexerError> {
    let url = format!("{}/{}", ARWEAVE_HOST, tx_id);
    Ok(reqwest::get(&url).await?.bytes().await?)
}
