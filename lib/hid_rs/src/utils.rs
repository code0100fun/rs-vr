pub fn bytes_to_str(bytes: &[u8]) -> &str {
    let first_null = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &std::str::from_utf8(bytes).unwrap()[0..first_null]
}

pub fn str_to_os_str(s: &str) -> Vec<i8> {
    let append = vec![0i8]; // terminate with 0
    let mut s_bytes = vec![];
    let bytes: &[i8] = unsafe { std::mem::transmute(s.as_bytes()) };
    s_bytes.extend(bytes.to_vec());
    s_bytes.extend(append);
    s_bytes
}