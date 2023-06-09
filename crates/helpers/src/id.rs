use anyhow::Result;
use base64::Engine;
use std::{process, thread, time};

fn now_ns() -> Result<u128> {
    let ns = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_nanos();

    Ok(ns)
}

#[allow(clippy::unnecessary_wraps)]
fn encode<T: AsRef<[u8]>>(data: T) -> Result<String> {
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(data);

    Ok(encoded)
}

pub fn time_id() -> Result<String> {
    let ns = now_ns()?;

    encode(ns.to_string())
}

pub fn time_thread_id() -> Result<String> {
    let thread_id = thread::current().id();
    let process_id = process::id();
    let ns = now_ns()?;

    let id = format!("{ns}-{process_id}-{thread_id:?}");

    encode(id)
}
