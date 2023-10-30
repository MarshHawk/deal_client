# Deal CLI client

A place to implement the game logic and tests for the texas-holdem-rs poker game.

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