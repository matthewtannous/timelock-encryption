use drand_verify::{G2PubkeyRfc, Pubkey};
use hex_literal::hex;
use reqwest::Error;
use serde::Deserialize;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// Only these relays support the v2 API
const URLS: [&str; 3] = [
    "https://api.drand.sh/v2/beacons/quicknet/rounds/",
    "https://api2.drand.sh/v2/beacons/quicknet/rounds/",
    "https://api3.drand.sh/v2/beacons/quicknet/rounds/",
];

const QUICKNET_PUBLIC_KEY: [u8; 96] = hex!(
    "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a"
);

// HashMap to store all fetched beacons
static BEACON_CACHE: LazyLock<Mutex<HashMap<u64, Beacon>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Beacon {
    round: u64,
    signature: String,
}

pub async fn get_beacon(round: u64) -> Beacon {
    // Check if the round value exists in the hashmap already to avoid network calls
    if let Some(beacon) = BEACON_CACHE.lock().unwrap().get(&round) {
        return beacon.clone();
    }

    let response: Result<(Beacon, Beacon, Beacon), Error> = tokio::try_join!(
        async {
            reqwest::get(format!("{}{round}", URLS[0]))
                .await?
                .error_for_status()?
                .json::<Beacon>()
                .await
        },
        async {
            reqwest::get(format!("{}{round}", URLS[1]))
                .await?
                .error_for_status()?
                .json::<Beacon>()
                .await
        },
        async {
            reqwest::get(format!("{}{round}", URLS[2]))
                .await?
                .error_for_status()?
                .json::<Beacon>()
                .await
        },
    );

    let (resp1, resp2, resp3) = response.unwrap();

    if resp1 == resp2 && resp2 == resp3 {
        let pk: G2PubkeyRfc = G2PubkeyRfc::from_fixed(QUICKNET_PUBLIC_KEY).unwrap();
        // Validate
        let result = pk
            .verify(resp1.round, b"", &hex::decode(&resp1.signature).unwrap())
            .unwrap();
        if result {
            // Add result to HashMap
            BEACON_CACHE.lock().unwrap().insert(round, resp2); // all responses are equal
            resp1
        } else {
            panic!("Response could not be verified.");
        }
    } else {
        panic!("Responses do not have the same value.");
    }
}
