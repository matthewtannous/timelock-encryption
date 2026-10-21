use crate::beacon::{Beacon, get_beacon};

mod beacon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let b1: Beacon = get_beacon(12345).await;
    println!("{b1:?}");
    let b1: Beacon = get_beacon(12345).await;
    println!("{b1:?}");
    let b1: Beacon = get_beacon(12345).await;
    println!("{b1:?}");

    Ok(())
}
