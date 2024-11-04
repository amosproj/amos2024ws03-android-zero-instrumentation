use std::time::{SystemTime, UNIX_EPOCH};



pub fn ebpf_program1() -> u128 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time
}

pub fn ebpf_program2() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}", time)
}