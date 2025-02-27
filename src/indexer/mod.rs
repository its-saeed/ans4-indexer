use parser::parse_bundle;

use crate::{error::IndexerError, network, utils};

mod models;
mod parser;
pub use models::*;

pub async fn index_bundle(tx_id: &str, output_path: &str) -> Result<(), IndexerError> {
    // Fetch bundle from Arweave
    let bundle = network::fetch_bundle(tx_id).await?;

    // Parse and validate bundle
    let parsed_bundle = parse_bundle(bundle)?;

    // Write parsed bundle to file
    utils::write_to_file(output_path, parsed_bundle)?;

    Ok(())
}
