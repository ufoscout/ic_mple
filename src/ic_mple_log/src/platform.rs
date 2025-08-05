use std::time::SystemTime;

/// returns the timestamp in nanoseconds
#[inline]
pub fn current_system_time() -> SystemTime {
    #[cfg(not(target_arch = "wasm32"))]
    {
        SystemTime::now()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let timestamp_in_nanos = ic_cdk::api::time();
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(timestamp_in_nanos)
    }
}

/// returns the timestamp in nanoseconds
#[inline]
pub fn current_timestamp_in_nanosecs() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("get current timestamp error")
            .as_nanos() as u64
    }

    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::time()
    }
}

#[inline]
pub fn print(data: &[u8]) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        print!("{}", String::from_utf8_lossy(data))
    }

    // ic::time() return the nano_sec, we need to change it to sec.
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::print(format!("{}", String::from_utf8_lossy(data)))
    }
}
