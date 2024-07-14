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
