use base64::Engine;
use std::time;

pub fn time_id() -> Result<String, time::SystemTimeError> {
    let now_ns = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_nanos();

    let id = base64::engine::general_purpose::STANDARD_NO_PAD.encode(now_ns.to_string());

    Ok(id)
}
