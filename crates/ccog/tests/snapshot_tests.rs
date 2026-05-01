use ccog::abi::{powl64_to_jsonld, powl64_to_postcard};
use ccog::powl64::{Powl64, Powl64RouteCell};

fn create_stable_powl64() -> Powl64 {
    let mut p = Powl64::new();
    for i in 1..=3 {
        p.extend(Powl64RouteCell {
            chain_head: i as u64,
            ..Default::default()
        });
    }
    p
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

#[test]
fn test_powl64_jsonld_snapshot() {
    let p = create_stable_powl64();
    let jsonld = powl64_to_jsonld(&p);
    let pretty = serde_json::to_string_pretty(&jsonld).unwrap();
    insta::assert_snapshot!("powl64_jsonld", pretty);
}

#[test]
fn test_powl64_postcard_snapshot() {
    let p = create_stable_powl64();
    let bytes = powl64_to_postcard(&p).expect("serialize postcard");
    let hex_str = bytes_to_hex(&bytes);
    insta::assert_snapshot!("powl64_postcard", hex_str);
}
