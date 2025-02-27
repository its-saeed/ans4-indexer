use crate::{error::IndexerError, indexer::DataItem};

pub fn write_to_file(output_path: &str, parsed_bundle: Vec<DataItem>) -> Result<(), IndexerError> {
    let contents = serde_json::to_string_pretty(&parsed_bundle)?;
    Ok(std::fs::write(output_path, contents)?)
}
