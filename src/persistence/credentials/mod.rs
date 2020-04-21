use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::persistence::errors::*;
use crate::persistence::{PostgresConfig, PostgresPersistance, Create};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Serialize,  Deserialize, Default, Debug, Clone)]
pub struct MetaData {
    valid_until: Option<DateTime<Utc>>,
    exportable:  bool,
    is_modifiable: bool,
    can_delete: bool,
    crypto_protection: CryptoType,
    key_id: String,
    extra: Vec<String>,
}

#[derive(Serialize,  Deserialize, Default, Debug, Clone)]
pub struct Value {
    metadata: MetaData,
    value: String,
}

trait Store {
    fn store_value(&mut self, metadata: MetaData, value: String, db_config : PostgresConfig) -> PersistenceErrorKind;
}

trait Create {
    fn create_metadata(&self) -> Result<String, PersistenceErrorKind>;

}

impl Create for MetaData {
    fn create_metadata(&self) -> Result<String, PersistenceErrorKind> {
       serde_json::to_string(&self).map_err(|_i| PersistenceErrorKind::IOError)
    }
}

impl Store for Value {
    fn store_value(&mut self, metadata: MetaData, value: String, &mut db_config : PostgresPersistance) -> PersistenceErrorKind {
        self.metadata = metadata;
        self.value = value;
        if !self.metadata.key_id.is_empty() && !self.value.is_empty() {
            /// need to think out how to connect to database ... does this come with the metadata??
            ///
            match db_config.config.uri {
                Some(t) => db_config.create_uri(),
                None => PersistenceErrorKind::InvalidConfig,
            }

            db_config.open()?;
            match db_config.client {
                Some(T) => T.batch_execute
            }
            
        } else {
            PersistenceErrorKind::IOError

        }
        PersistenceErrorKind::Success
    }
}
//fn store_value(value: String, metadata : MetaData) ->  PersistenceErrorKind {
//    if metadata.key_id {
//        let value_to_store = Value { metadata, value };
//
//    } else {
//        return PersistenceErrorKind::IOError;
//    }
//
//    PersistenceErrorKind::Success



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


