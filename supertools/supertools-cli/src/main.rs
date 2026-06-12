use clap::Parser;
use supertools_cli::args;

#[tokio::main]
async fn main() {
    match args::Cli::try_parse() {
        Ok(cli) => {
            println!("Parsed CLI: {:?}", cli);
        }
        Err(err) => {
            err.exit();
        }
    }
}
