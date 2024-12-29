//! timechain_bot
//! poast on x.com and nostr stats about bitcoin's timechain
//! x.com/timechain_bot
//! timechain_bot@luisschwab.net

use dotenv::dotenv;
use nostr_sdk::prelude::*;
use serde_json::Value;
use std::env;
use thousands::Separable;
use tweety_rs::TweetyClient;

const POAST_X: bool = true;
const POAST_NOSTR: bool = true;

const MEMPOOL_API: &str = "https://mempool.space";

#[rustfmt::skip]
const NOSTR_RELAYS: &[&str] = &[
    "wss://nostr.luisschwab.net",
    "wss://relay.primal.net"
    // add your relays here
];

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let env = vec![
        "X_CONSUMER_KEY",
        "X_CONSUMER_SECRET",
        "X_ACCESS_TOKEN",
        "X_ACCESS_TOKEN_SECRET",
        "NOSTR_SEC",
    ];

    dotenv().ok();
    let mut kv = std::collections::HashMap::new();

    // save env to kv
    for var in &env {
        match env::var(var) {
            Ok(value) => {
                //println!("{}: {}", var, value);
                kv.insert(*var, value);
            }
            Err(e) => {
                eprintln!("couldn't read {} from environment: {}", var, e);
                std::process::exit(-1);
            }
        }
    }

    // chain tip height
    let tip_endpoint: &str = "/api/blocks/tip/height";
    let tip = reqwest::get(format!("{}{}", MEMPOOL_API, tip_endpoint))
        .await?
        .text()
        .await?;
    let tip: u32 = tip.parse().unwrap();
    //println!("{}", tip);

    // epoch progress
    let epoch: u32 = tip / (210_000 as u32);
    let epoch_progress: f32 = (tip as f32 % 210_000 as f32) / (210_000 as f32);
    //println!("{}, {}", epoch, epoch_progress);

    // hashrate in EH/s
    let hashrate_endpoint: &str = "/api/v1/mining/hashrate/3d";
    let hashrate = reqwest::get(format!("{}{}", MEMPOOL_API, hashrate_endpoint))
        .await?
        .json::<Value>()
        .await?
        .get("currentHashrate")
        .and_then(|h| h.as_f64())
        .map(|h| h / 1e18)
        .unwrap_or(0.0);
    //println!("{}", hashrate);

    // supply
    let mut supply: f32 = 0.0;
    for i in 0..epoch {
        supply += 210_000.0 * (50.0 / (2_f32.powf(i as f32)));
    }
    supply += 210_000.0 * (50.0 / 2_f32.powf(epoch as f32)) * epoch_progress;
    //println!("{}", supply);
    
    // build post
    let mut post = format!( 
        "height: {}\n\
        hashrate: {:.2} EH/s\n\
        supply: ₿{} [{:.2}%]\n\
        epoch: {} [{:.2}%]",
        tip.separate_with_commas(),
        hashrate,
        supply.separate_with_commas(),
        100.0 * (supply / 21e6 as f32),
        epoch,
        100.0 * epoch_progress
    );
    println!("{}", post);

    // holidays 
    // let holiday_api: &str = "https://bitcoinexplorer.org/api/holidays/01-03/";
    // TODO: figure out how to bypass CF's 403
    // works with curl but no with reqwest
    
    let payload = post; // + holiday

    // poast on x.com
    if POAST_X {
        let x_client = TweetyClient::new(
            &kv["X_CONSUMER_KEY"],
            &kv["X_ACCESS_TOKEN"],
            &kv["X_CONSUMER_SECRET"],
            &kv["X_ACCESS_TOKEN_SECRET"],
        );

        match x_client.post_tweet(&payload, None).await {
            Ok(_) => {
                println!("successfully poasted to x: https://x.com/timechain_bot");
            }
            Err(e) => {
                println!("error while poasting to https://x.com/timechain_bot: {:?}", e);
            }
        }
    }
    
    // poast on nostr
    if POAST_NOSTR {
        let privkey = match Keys::parse(&kv["NOSTR_SEC"]) {
            Ok(key) => key,
            Err(e) => {
                println!("error while parsing nostr private key: {:#?}", e);
                std::process::exit(-1);
            }
        };

        let nostr_client = Client::new(privkey);

        for relay in NOSTR_RELAYS {
            nostr_client.add_relay(*relay).await.unwrap();
            nostr_client.connect().await;
        }

        match nostr_client.publish_text_note(payload, []).await {
            Ok(output) => {
                println!("successfully poasted to nostr: https://njump.me/{}", output.id().to_hex());
            }
            Err(e) => {
                println!("error while poasting to nostr:\n{:#?}", e);
            }
        }
    }

    Ok(())
}
