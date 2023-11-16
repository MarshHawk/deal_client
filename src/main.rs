use std::env;

//A command-line tool to play Texas Holdem Poker
use clap::Parser;

pub mod deal {
    include!("deal_app.rs");
}
use deal::dealer_client::DealerClient;
use deal::{HandRequest, HandResponse};
use inquire::{InquireError, Select};
use rusoto_core::credential::EnvironmentProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_dynamodb::DynamoDbClient;
use tonic::Request;

mod table {
    pub mod model {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct Table {
            pub id: String,
            pub(crate) player_ids: Vec<String>,
        }

        impl Table {
            async fn id(&self) -> &str {
                &self.id
            }

            async fn player_ids(&self) -> &[String] {
                &self.player_ids
            }
        }
    }

    pub mod repository {
        use rusoto_dynamodb::{
            AttributeValue, DynamoDb, DynamoDbClient, PutItemInput, ScanInput, UpdateItemInput,
        };
        use serde_dynamodb::from_hashmap;
        use uuid::Uuid;

        pub struct TableRepository {
            client: DynamoDbClient,
            table_name: String,
        }

        impl TableRepository {
            pub fn new(client: DynamoDbClient, table_name: String) -> Self {
                Self { client, table_name }
            }

            pub async fn create_table(&self) -> Result<super::model::Table, String> {
                let table = super::model::Table {
                    id: Uuid::new_v4().to_string(),
                    player_ids: Vec::new(),
                };
                let item = serde_dynamodb::to_hashmap(&table).map_err(|e| e.to_string())?;
                let input = PutItemInput {
                    table_name: self.table_name.clone(),
                    item,
                    ..Default::default()
                };
                self.client
                    .put_item(input)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(table)
            }

            pub async fn add_player_to_table(
                &self,
                table_id: &str,
                player_id: &str,
            ) -> Result<(), String> {
                let key = [(
                    "id".to_string(),
                    AttributeValue {
                        s: Some(table_id.to_string()),
                        ..Default::default()
                    },
                )]
                .iter()
                .cloned()
                .collect();
                let update_expression = "SET player_ids = list_append(player_ids, :player_id)";
                let condition_expression = "size(player_ids) < :max_players";
                let vec = vec![AttributeValue {
                    s: Some(table_id.to_string()),
                    ..Default::default()
                }];
                let expression_attribute_values = [
                    (
                        ":player_id".to_string(),
                        AttributeValue {
                            l: Some(vec),
                            ..Default::default()
                        },
                    ),
                    (
                        ":max_players".to_string(),
                        AttributeValue {
                            n: Some("10".to_string()),
                            ..Default::default()
                        },
                    ),
                ]
                .iter()
                .cloned()
                .collect();
                let input = UpdateItemInput {
                    table_name: self.table_name.clone(),
                    key,
                    update_expression: Some(update_expression.to_string()),
                    condition_expression: Some(condition_expression.to_string()),
                    expression_attribute_values: Some(expression_attribute_values),
                    ..Default::default()
                };
                self.client
                    .update_item(input)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(())
            }

            pub async fn get_tables_with_space(&self) -> Result<Vec<super::model::Table>, String> {
                let expression_attribute_values = [(
                    ":max_players".to_string(),
                    AttributeValue {
                        n: Some("10".to_string()),
                        ..Default::default()
                    },
                )]
                .iter()
                .cloned()
                .collect();
                let input = ScanInput {
                    table_name: self.table_name.clone(),
                    filter_expression: Some("size(player_ids) < :max_players".to_string()),
                    expression_attribute_values: Some(expression_attribute_values),
                    ..Default::default()
                };
                let result = self.client.scan(input).await.map_err(|e| e.to_string())?;
                let items = result.items.ok_or("No items found".to_string())?;
                let tables = items
                    .iter()
                    .map(|item| {
                        let table: super::model::Table = from_hashmap(item.clone()).unwrap();
                        table
                    })
                    .collect();
                Ok(tables)
            }
        }
    }

    pub mod service {
        pub struct TableService {
            table_repository: super::repository::TableRepository,
        }

        impl TableService {
            pub fn new(table_repository: super::repository::TableRepository) -> Self {
                Self { table_repository }
            }
            pub async fn create_table(&mut self) -> Result<super::model::Table, String> {
                self.table_repository.create_table().await
            }

            pub async fn add_player_to_table(
                &mut self,
                table_id: &str,
                player_id: &str,
            ) -> Result<(), String> {
                self.table_repository
                    .add_player_to_table(table_id, player_id)
                    .await
            }

            pub async fn get_tables_with_space(&self) -> Result<Vec<super::model::Table>, String> {
                self.table_repository.get_tables_with_space().await
            }
        }
    }
}

mod player {
    pub mod model {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct Player {
            pub id: String,
            pub stack: Option<f64>,
            pub cards: Vec<String>,
            pub score: i32,
            pub description: String,
        }

        impl Player {
            async fn id(&self) -> &str {
                &self.id
            }

            async fn stack(&self) -> Option<f64> {
                self.stack
            }

            async fn cards(&self) -> &[String] {
                &self.cards
            }

            async fn score(&self) -> i32 {
                self.score
            }

            async fn description(&self) -> &str {
                &self.description
            }
        }
    }
}

mod hand {
    pub mod model {
        use crate::player::model::Player;


        pub struct Hand {
            id: String,
            table_id: String,
            players: Vec<Player>,
            cards: Cards,
            player_events: Vec<PlayerEvent>,
            street_events: Vec<StreetEvent>,
        }

        struct Cards {
            flop: Vec<String>,
            turn: String,
            river: String,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct PlayerEvent {
            pub player_id: String,
            pub action: PlayerAction,
            pub amount: f64,
            pub street_type: StreetType,
            pub current_stack: Option<f64>,
            pub current_pot: Option<f64>,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum PlayerAction {
            Bet,
            Check,
            Fold,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum StreetType {
            Preflop,
            Flop,
            Turn,
            River,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct StreetEvent {
            street_type: StreetType,
            current_active_players: Vec<ActivePlayer>,
            pot: f64,
            cycle_count: u32,
            should_increment_cycle: bool,
        }

        #[derive(Clone, Debug, PartialEq)]
        struct ActivePlayer {
            id: String,
            bet: f64,
            stack: f64,
            is_inactive: Option<bool>,
        }
    }
}

#[derive(Parser)]
#[clap(
    version = "0.1",
    author = "Sean Glover",
    about = "A Poker Game in Rust"
)]
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
    Play {
        player_id: String,
        #[clap(long, short, action)]
        start: bool,
    },
}

fn get_table_actions() -> Vec<&'static str> {
    vec!["Join Table", "Create Table"]
}

fn get_dynamodb_local_client() -> DynamoDbClient {
    env::set_var("AWS_ACCESS_KEY_ID", "123");
    env::set_var("AWS_SECRET_ACCESS_KEY", "xyz");
    // Create custom Region
    let region = Region::Custom {
        name: "us-east-2".to_owned(),
        endpoint: "http://localhost:8000".to_owned(),
    };

    DynamoDbClient::new_with(
        HttpClient::new().unwrap(),
        EnvironmentProvider::default(),
        region,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match args.command {
        Some(Commands::Deal { player_count }) => {
            println!("Requesting deal for player count: {}", player_count);
            let mut client = DealerClient::connect("http://127.0.0.1:5003").await?;
            let req = Request::new(HandRequest {
                player_count: player_count as i32,
            });
            println!("deal request: {:#?}", req);
            let hand: HandResponse = client.deal(req).await?.into_inner();
            println!("Requested deal: {:#?}", hand);
        }
        Some(Commands::Play { player_id, start }) => {
            println!("Requesting play for: {}", player_id);
            println!("Game will start: {}", start);
            let client = get_dynamodb_local_client();
            const TABLES_TABLE_NAME: &str = "tables";
            let table_repository =
                table::repository::TableRepository::new(client, TABLES_TABLE_NAME.to_string());
            let mut table_service = table::service::TableService::new(table_repository);
            let result = table_service.create_table().await;
            println!("Create table result: {:?}", result);
            let tables = table_service.get_tables_with_space().await;
            println!("Tables: {:?}", tables);

            let table_action_options: Vec<&str> = get_table_actions();

            let table_action_ans: Result<&str, InquireError> =
                Select::new("Please choose:", table_action_options).prompt();

            match table_action_ans {
                Ok("Join Table") => {
                    let tables = table_service.get_tables_with_space().await?;
                    let table_names: Vec<&str> =
                        tables.iter().map(|table| table.id.as_str()).collect();
                    let table_name_ans: Result<&str, InquireError> =
                        Select::new("Please choose a table to join:", table_names).prompt();
                    let table_name = table_name_ans.unwrap();
                    let result = table_service
                        .add_player_to_table(table_name, &player_id)
                        .await;
                    println!("Add player to table result: {:?}", result);
                }
                Ok("Create Table") => {
                    let result = table_service.create_table().await;
                    match result {
                        Ok(table) => {
                            println!("Table created with id: {}", table.id);
                            let result = table_service
                                .add_player_to_table(&table.id, &player_id)
                                .await;
                            match result {
                                Ok(_) => println!("Player added to table successfully"),
                                Err(e) => println!("Error adding player to table: {:?}", e),
                            }
                        }
                        Err(e) => println!("Error creating table: {:?}", e),
                    }
                }
                _ => {}
            }
        }
        None => println!("No subcommand was used"),
    }

    Ok(())
}
