use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::persistence::errors::*;

#[derive(Serialize, Deserialize, Debug)]
enum CryptoTypes {
    Aes128Gcm
//    aes_128_gcm,
}

#[derive(Serialize, Default, Debug)]
pub struct MetaData {
    owner : Option<String>,
    valid_until: Option<DateTime<Utc>>,
    exportable:  bool,
    sensitive: Option<Vec<String>>,
    is_modifiable: bool,
    can_delete: bool,
    crypto_protection: CryptoTypes,
    synchronized: bool,
}

trait Store {
    fn create_metadata(&self) -> Result<String, PersistenceErrorKind>;
}


impl Store for MetaData {


    fn create_metadata(&self) -> Result<String, PersistenceErrorKind> {

        Ok(serde_json::to_string(&self)?)

    }

}


#[cfg(test)]
mod credential_tests {
    use crate::persistence::credentials::MetaData;

    #[test]
    fn test_create_metadata_searilizes(){

        let demo_config= r#"{"owner":"","valid_until":"2020-02-25T19:31:01.147Z","exportable":false,"sensitive":[],"is_modifiable":false,"can_delete":false,"crypto_protection":"","synchronized":false}"#;
        let test_metadata_config_object: MetaData =
            serde_json::from_str(&demo_config).unwrap();

    }





}


