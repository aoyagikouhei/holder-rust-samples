#[path = "holder.rs"]
mod holder;

use chrono::prelude::*;
use std::sync::OnceLock;

use crate::holder::Holder;

pub static MAP: OnceLock<Holder<String, String>> = OnceLock::new();

pub fn setup(expire_interval: std::time::Duration) {
    MAP.get_or_init(|| Holder::new(expire_interval));
}

fn main() {
    setup(std::time::Duration::from_secs(2));
    let cache = MAP.get().unwrap();

    let now = Utc::now();
    cache.insert("user:1".to_string(), "Alice".to_string(), now);
    cache.insert("user:2".to_string(), "Bob".to_string(), now);

    println!("--- 直後 ---");
    println!("user:1 = {:?}", cache.get(&"user:1".to_string(), Utc::now()));
    println!("user:2 = {:?}", cache.get(&"user:2".to_string(), Utc::now()));
    println!("user:3 = {:?}", cache.get(&"user:3".to_string(), Utc::now()));

    // 期限内の insert は無視される(既存の値が保持される)
    cache.insert("user:1".to_string(), "Alice2".to_string(), Utc::now());
    println!("user:1 (上書き試行後) = {:?}", cache.get(&"user:1".to_string(), Utc::now()));

    println!("--- 3秒待機して期限切れを確認 ---");
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("user:1 = {:?}", cache.get(&"user:1".to_string(), Utc::now()));

    // 期限切れ後は insert が反映される
    cache.insert("user:1".to_string(), "Alice3".to_string(), Utc::now());
    println!("user:1 (再 insert 後) = {:?}", cache.get(&"user:1".to_string(), Utc::now()));
}
