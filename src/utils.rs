pub fn long_address_to_string(address: u64) -> String {
    let addr = format!("{:012x}", address);
    let pairs: Vec<&[u8]> = addr.as_bytes().chunks(2).collect();
    let pairs = pairs.join(&(':' as u8));
    String::from_utf8(pairs).unwrap()
}

pub fn address_bytes_to_string(address: &[u8]) -> String {
    let pairs: Vec<String> = address.to_vec()
        .into_iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    pairs.join(&(":"))
}