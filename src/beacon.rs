use blst::min_sig::{PublicKey, Signature};
use hex_literal::hex;
use reqwest::Error;
use serde::Deserialize;
use sha2::{Digest, Sha256};

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
        if verify_beacon(&resp1) {
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

fn verify_beacon(beacon: &Beacon) -> bool {
    let pk = PublicKey::from_bytes(&QUICKNET_PUBLIC_KEY).expect("Invalid Quicknet public key");

    let signature_bytes = hex::decode(&beacon.signature).expect("Invalid signature hex");

    let signature = Signature::from_bytes(&signature_bytes).expect("Invalid signature encoding");

    // Quicknet message = SHA-256(round.to_be_bytes())
    let message = Sha256::digest(beacon.round.to_be_bytes());

    let result = signature.verify(
        true,
        &message,
        b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_",
        &[],
        &pk,
        true,
    );

    result == blst::BLST_ERROR::BLST_SUCCESS
}
