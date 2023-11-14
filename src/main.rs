use std::env;

//A command-line tool to play Texas Holdem Poker
use clap::Parser;

pub mod deal {
    include!("deal_app.rs");
}
use deal::dealer_client::DealerClient;
use deal::{HandRequest, HandResponse};
use inquire::error::InquireResult;
use inquire::Select;
use rusoto_core::credential::EnvironmentProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, ListTablesInput};

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
            AttributeValue, DynamoDb, DynamoDbClient, PutItemInput, ScanInput,
            UpdateItemInput,
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
                let update_expression =
            "SET player_ids = list_append(if_not_exists(player_ids, :empty_list), :player_id)";
                let condition_expression = "size(player_ids) < :max_players";
                let expression_attribute_values = [
                    (
                        ":player_id".to_string(),
                        AttributeValue {
                            ss: Some(vec![player_id.to_string()]),
                            ..Default::default()
                        },
                    ),
                    (
                        ":empty_list".to_string(),
                        AttributeValue {
                            ss: Some(vec![]),
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

//#[derive(Parser)]
//#[clap(version = "0.1", author = "Sean Glover", about = "A Poker Game in Rust")]
//struct Options {
//    #[clap(subcommand)]
//    command: Command,
//}
//
//#[derive(Parser)]
//struct DealOptions {
//    #[clap(long)]
//    player_count: u32,
//}
//
//
//#[derive(Parser)]
//struct PlayOptions {
//    name: String,
//}
//
//
//#[derive(Parser)]
//enum Command {
//    Deal(DealOptions),
//}

//async fn connect_to_table_service() -> Result<TableServiceClient<Channel>, Box<dyn std::error::Error>> {
//    let channel = Channel::from_static("http://localhost:5004")
//        .connect()
//        .await?;
//    let client = TableServiceClient::new(channel);
//    Ok(client)
//}

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
async fn main() -> InquireResult<()> {
    let client = get_dynamodb_local_client();
    const TABLES_TABLE_NAME: &str = "tables";
    let table_repository =
        table::repository::TableRepository::new(client, TABLES_TABLE_NAME.to_string());
    let mut table_service = table::service::TableService::new(table_repository);
    let result = table_service.create_table().await;
    println!("Create table result: {:?}", result);
    let tables = table_service.get_tables_with_space().await;
    println!("Tables: {:?}", tables);

    //let list_tables_input: ListTablesInput = Default::default();
    //let result = client.list_tables(list_tables_input).await.unwrap();
    //println!("Tables: {:?}", result.table_names.unwrap_or_default());
    //let args = Options::parse();
    //
    //use Command::*;
    //match args.command {
    //    Deal(args) => {
    //        println!("Requesting deal for player count: {}", args.player_count);
    //        let mut client = DealerClient::connect("http://127.0.0.1:5003").await?;
    //        let req = Request::new(HandRequest {
    //            player_count: args.player_count as i32,
    //        });
    //        println!("deal request: {:#?}", req);
    //        let hand: HandResponse = client.deal(req).await?.into_inner();
    //        println!("Requested deal: {:#?}", hand);
    //    }
    //}

    Ok(())
}
