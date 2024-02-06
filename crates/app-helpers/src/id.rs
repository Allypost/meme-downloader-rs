use std::{process, thread, time};

use base64::Engine;

fn now_ns() -> u128 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos()
}

fn encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(data)
}

#[must_use]
pub fn time_id() -> String {
    encode(now_ns().to_string())
}

#[must_use]
pub fn time_thread_id() -> String {
    let thread_id = thread::current().id();
    let process_id = process::id();
    let ns = now_ns();

    let id = format!("{ns}-{process_id}-{thread_id:?}");

    encode(id)
}
