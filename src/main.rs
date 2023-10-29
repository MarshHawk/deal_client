//A command-line tool to play Texas Holdem Poker
use clap::Parser;

pub mod deal {
    include!("deal_app.rs");
}
use deal::dealer_client::DealerClient;
use deal::{
    HandRequest,HandResponse,Hand,Board
};
use tonic::Request;

#[derive(Parser)]
#[clap(version = "0.1", author = "Sean Glover", about = "A Poker Game in Rust")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    #[clap(version = "1.0", author = "Sean Glover")]
    Deal {
        #[clap(long)]
        player_count: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match args.command {
        Some(Commands::Deal { player_count  }) => {
            println!("Requesting deal for player count: {}", player_count);
            let mut client = DealerClient::connect("http://127.0.0.1:5003").await?;
            let req = Request::new(HandRequest {
                player_count: player_count as i32,
            });
            println!("deal request: {:#?}", req);
            let hand = client.deal(req).await?.into_inner();
            println!("Requested deal: {:#?}", hand);
        }
        None => println!("No subcommand was used"),
    }
    Ok(())
}