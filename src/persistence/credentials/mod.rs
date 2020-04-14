use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::persistence::errors::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum CryptoType {
    Aes128Gcm,
    // This means that it is tamper protected but not secured with cryptography
    HmacSha256,
    NoEncryption
}
impl Default for CryptoType {
    fn default() -> CryptoType {
        CryptoType::NoEncryption
    }
}
#[derive(Serialize,  Deserialize, Default, Debug)]
pub struct MetaData {
    valid_until: Option<DateTime<Utc>>,
    exportable:  bool,
    is_modifiable: bool,
    can_delete: bool,
    crypto_protection: CryptoType,
    key_id: String,
    extra: Vec<String>,
}



#[derive(Serialize,  Deserialize, Default, Debug)]
pub struct Value {
    metadata: MetaData,
    value: String,
}

trait Store {
    fn store_value(&self) -> PersistenceErrorKind;
}

trait Create {
    fn create_metadata(&self) -> Result<String, PersistenceErrorKind>;

}

impl Create for MetaData {
    fn create_metadata(&self) -> Result<String, PersistenceErrorKind> {
       serde_json::to_string(&self).map_err(|_i| PersistenceErrorKind::IOError)
    }
}

fn store_value(value: Value, metadata : MetaData) ->  PersistenceErrorKind {



}


#[cfg(test)]
mod credential_tests {
    use crate::persistence::credentials::MetaData;

    #[test]
    fn test_create_metadata_searilizes(){
        let demo_config=
            r#"{"owner":"","valid_until":"2020-02-25T19:31:01.147Z","exportable":false,"sensitive":[],"is_modifiable":false,"can_delete":false,"crypto_protection":"Aes128Gcm","synchronized":false, "key_id": "123", "extra":["none"]}"#;
        let test_metadata_config_object: MetaData =
            serde_json::from_str(&demo_config).unwrap();
        assert!(!test_metadata_config_object.is_modifiable)
    }


}


