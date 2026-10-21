use time::UtcDateTime;

const QUICKNET_GENESIS_TIME: u64 = 1692803367;

pub fn get_round_from_date(date: UtcDateTime) -> u64 {
    (date.unix_timestamp() as u64 - QUICKNET_GENESIS_TIME) / 3
}

pub fn get_date_from_round(round: u64) -> UtcDateTime {
    println!("{}", QUICKNET_GENESIS_TIME + round * 3);
    UtcDateTime::from_unix_timestamp((QUICKNET_GENESIS_TIME + round * 3) as i64).unwrap()
}
