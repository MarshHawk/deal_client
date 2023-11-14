# Deal CLI client

A place to implement the game logic and tests for the texas-holdem-rs poker game.

## Problem Statement
Implement backend logic for multiplayer 'never-ending poker tournament' using the deal client payload, nosql database and streaming solution (e.g. kafka-esque)

### Deal Client payload
```rust
cargo run deal --player-count 3
   Compiling deal_client v0.1.0 (/Users/seanglover/Development/deal_client)
    Finished dev [unoptimized + debuginfo] target(s) in 0.83s
     Running `target/debug/deal_client deal --player-count 3`
Requesting deal for player count: 3
deal request: Request {
    metadata: MetadataMap {
        headers: {},
    },
    message: HandRequest {
        player_count: 3,
    },
    extensions: Extensions,
}
Requested deal: HandResponse {
    board: Some(
        Board {
            flop: [
                "6c",
                "Ac",
                "8d",
            ],
            turn: "7d",
            river: "5s",
        },
    ),
    hands: [
        Hand {
            cards: [
                "2h",
                "Ad",
            ],
            score: 0.5294827124095417,
            description: "Pair",
        },
        Hand {
            cards: [
                "5d",
                "5c",
            ],
            score: 0.7039667649423746,
            description: "Three of a Kind",
        },
        Hand {
            cards: [
                "Qh",
                "4h",
            ],
            score: 0.7847761994103457,
            description: "Straight",
        },
    ],
}
```

### Database Init
```bash
aws dynamodb create-table --table-name tables --attribute-definitions AttributeName=id,AttributeType=S --key-schema AttributeName=id,KeyType=HASH --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 --endpoint-url http://localhost:8000 --region us-east-2

aws dynamodb create-table --table-name hands --attribute-definitions AttributeName=id,AttributeType=S --key-schema AttributeName=id,KeyType=HASH --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 --endpoint-url http://localhost:8000 --region us-east-2

aws dynamodb create-table --table-name users --attribute-definitions AttributeName=id,AttributeType=S --key-schema AttributeName=id,KeyType=HASH --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 --endpoint-url http://localhost:8000 --region us-east-2
```

### References
- https://github.com/ArtRand/kafka-actix-example/blob/master/docker-compose.yml
- https://github.com/awslabs/dynein/blob/main/k8s-deploy-dynamodb-local.yml
- https://github.com/fairingrey/actix-realworld-example-app
- https://github.com/hyperium/tonic
- https://docs.iggy.rs/introduction/getting-started/
- https://konghq.com/blog/engineering/building-grpc-apis-with-rust
- https://github.com/Unleash/actix-template/blob/main/src/main.rs
- https://www.youtube.com/watch?v=mqpbELU3chQ&t=659s
- https://github.com/SaltyAom/actix-web-k8s-example/blob/main/README.md
- https://actix.rs/docs/extractors/
- https://dev.to/ciscoemerge/how-to-build-a-simple-kafka-producerconsumer-application-in-rust-3pl4
- https://projecteuler.net/problem=54
- https://github.com/rohanvedula/poker-cpp
- https://github.com/AndreiVasilev/Poker_Game_Engine
- https://github.com/EricSteinberger/PokerRL
- https://github.com/EricSteinberger/DREAM