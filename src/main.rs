use ans104_indexer::cli::{handle_index_command, Cli};
use ans104_indexer::error::IndexerError;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), IndexerError> {
    let cli = Cli::from_args();

    handle_index_command(&cli.tx_id, &cli.output).await
}
