use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;
use crate::persistence::errors::*;
use crate::persistence::{PostgresPersistance, Connect, Create};
use ursa::encryption::symm::prelude::*;



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
    encryption : Option<CryptoType>,
}

trait Store {
    fn store_value_synchronous(&mut self, value: Value, db_config : PostgresPersistance) -> Result<(),PersistenceErrorKind>;

}
trait EncryptData {

    fn encrypt_string_aes_128gcm(msg : String, key : String) -> Vec<u8>;
    fn decrypt_string_aes_128gcm(msg : String, key : String) -> Vec<u8>;
}

impl EncryptData for CryptoType {

    fn encrypt_string_aes_128gcm(msg : String, key : String) -> Vec<u8> {

        let converted_msg = msg.as_bytes();
        let converted_key = key.as_bytes();
        let aad = b"Using Aes128Gcm to encrypt credential";

        io::stdout().flush().unwrap();

        let encryptor = SymmetricEncryptor::<Aes128Gcm>::new_with_key(converted_key);

        let res = match encryptor {
            Ok(t) => t.encrypt_easy(aad.as_ref(), converted_msg.as_ref()) ,
            Err(e) => panic!("{:?}", e),
        };

        let res_unrwaped = match res {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        };

        res_unrwaped
    }


    fn decrypt_string_aes_128gcm(msg : String, key : String) -> Vec<u8> {

        let converted_msg = msg.as_bytes();
        let converted_key = key.as_bytes();
        let aad = b"Using Aes128Gcm to encrypt credential";

        io::stdout().flush().unwrap();

        let encryptor = SymmetricEncryptor::<Aes128Gcm>::new_with_key(converted_key);

        let res = match encryptor {
            Ok(t) => t.decrypt_easy(aad.as_ref(), converted_msg.as_ref()) ,
            Err(e) => panic!("{:?}", e),
        };

        let res_unrwaped = match res {
            Ok(t) => t,
            Err(e) => panic!("{:?}", e),
        };

        res_unrwaped
    }


}

trait CreateData {
    fn create_metadata(&self) -> Result<String, PersistenceErrorKind>;

}

impl CreateData for MetaData {
    fn create_metadata(&self) -> Result<String, PersistenceErrorKind> {
       serde_json::to_string(&self).map_err(|_i| PersistenceErrorKind::IOError)
    }
}



impl Store for Value {
    fn store_value_synchronous(&mut self, value: Value, db_config : PostgresPersistance)
                               -> Result<(),PersistenceErrorKind> {


        let mut db_config = db_config;
        if !self.metadata.key_id.is_empty() && !self.value.is_empty() {

            let _uri= db_config.config.create_uri()?;

            db_config.open()?;

            let mut db_client = match db_config.client {
                Ok(v) => v,
                Err(e) => panic!("{:?}", e),
            };


            // work around look to fix this

            let serialized_credential  = match serde_json::to_string(&value) {
                Ok(v) => v,
                Err(e) => panic!("{:?}", e),
            };
            let credential_clone = serialized_credential.clone();

            match db_client.batch_execute(format!("
                    DO $$
                        BEGIN
                            IF EXISTS(SELECT * FROM information_schema.tables WHERE table_schema = current_schema()
                            AND table_name = 'indy_storage') THEN
                                INSERT INTO indy_storage(cred_value)
                                VALUES('{}');
                            ELSE
                                CREATE TABLE indy_storage (cred_value json NOT NULL);
                                INSERT INTO indy_storage(cred_value)
                                VALUES('{}');
                            END IF
                    END $$;", serialized_credential, credential_clone).as_str())  {
                Ok(t) => t,
                Err(e) => panic!("{:?}", e),
            };
        }
        Ok(())
    }

}



#[cfg(test)]
mod credential_tests {
    use crate::persistence::credentials::{MetaData, Value, CryptoType};

    #[test]
    fn test_create_metadata_searilizes(){
        let demo_config=
            r#"{"owner":"","valid_until":"2020-02-25T19:31:01.147Z","exportable":false,"sensitive":[],"is_modifiable":false,"can_delete":false,"crypto_protection":"Aes128Gcm","synchronized":false, "key_id": "123", "extra":["none"]}"#;
        let test_metadata_config_object: MetaData =
            serde_json::from_str(&demo_config).unwrap();
        assert!(!test_metadata_config_object.is_modifiable)
    }


    #[test]
    fn test_store_metadata_in_value_without_data(){

        let demo_config=
            r#"{"owner":"","valid_until":"2020-02-25T19:31:01.147Z","exportable":false,"sensitive":[],"is_modifiable":false,"can_delete":false,"crypto_protection":"Aes128Gcm","synchronized":false, "key_id": "123", "extra":["none"]}"#;
        let test_metadata_config_object: MetaData =
            serde_json::from_str(&demo_config).unwrap();

        let test_value : Value  = Value {
            metadata : test_metadata_config_object,
            value : "".to_string(),
            encryption : Some(CryptoType::NoEncryption)
        };

        assert!(!test_value.metadata.is_modifiable)

    }

    #[test]
    fn test_serialize_value_without_data(){

        let demo_config=
            r#"{"owner":"","valid_until":"2020-02-25T19:31:01.147Z","exportable":false,"sensitive":[],"is_modifiable":false,"can_delete":false,"crypto_protection":"Aes128Gcm","synchronized":false, "key_id": "123", "extra":["none"]}"#;
        let test_metadata_config_object: MetaData =
            serde_json::from_str(&demo_config).unwrap();

        let test_value : Value  = Value {
            metadata : test_metadata_config_object,
            value : "".to_string(),
            encryption : Some(CryptoType::NoEncryption)
        };

        let test_serialized_credential  = match serde_json::to_string(&test_value) {
            Ok(v) => v,
            Err(e) => panic!("{:?}", e),
        };

        assert!(!test_serialized_credential.is_empty())

    }

    #[test]
    fn test_allows_no_encryption(){

        let demo_config=
            r#"{"owner":"","valid_until":"2020-02-25T19:31:01.147Z","exportable":false,"sensitive":[],"is_modifiable":false,"can_delete":false,"crypto_protection":"Aes128Gcm","synchronized":false, "key_id": "123", "extra":["none"]}"#;
        let test_metadata_config_object: MetaData =
            serde_json::from_str(&demo_config).unwrap();

        let test_value : Value  = Value {
            metadata : test_metadata_config_object,
            value : "".to_string(),
            encryption : None
        };

        let test_serialized_credential  = match serde_json::to_string(&test_value) {
            Ok(v) => v,
            Err(e) => panic!("{:?}", e),
        };

        assert!(!test_serialized_credential.is_empty())

    }

}


