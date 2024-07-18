use uuid::Uuid;

pub fn generate_offline_uuid(name: &'_ str) -> Vec<u8> {
    Vec::from(
        uuid::Uuid::new_v3(
            &Uuid::NAMESPACE_OID,
            &format!("OfflinePlayer:{}", name).into_bytes(),
        )
        .into_bytes(),
    )
}

#[macro_export]
macro_rules! measure {
    ($name:expr, $e:expr) => {{
        use std::time::Instant;

        let now = Instant::now();
        let result = $e;
        let elapsed = now.elapsed();
        log::debug!("Measure[{}] => {:.2?}", $name, elapsed);

        result
    }};
}
