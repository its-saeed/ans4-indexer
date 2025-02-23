use crate::{error::IndexerError, indexer::index_bundle};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "arweave_ans104_indexer",
    about = "ANS-104 Bundle Indexer for Arweave"
)]
pub struct Cli {
    #[structopt(help = "The transaction ID of the ANS-104 bundle")]
    pub tx_id: String,

    #[structopt(short, long, help = "Output file to write the parsed array")]
    pub output: String,
}

pub async fn handle_index_command(tx_id: &str, output_path: &str) -> Result<(), IndexerError> {
    println!("Indexing bundle with transaction ID: {}", tx_id);
    index_bundle(tx_id, output_path).await?;
    println!("Bundle indexed successfully and saved to: {}", output_path);
    Ok(())
}
