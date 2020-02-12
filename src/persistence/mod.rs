/// The errors that can occur during a persistence operation
pub mod errors;

use postgres::{Client, NoTls};
//use std::{fmt, env,path::PathBuf};
//use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::persistence::errors::PersistenceErrorKind;


pub enum PersistenceConfig
{
    PostgresStorage(PostgresConfig)
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresConfig {
    user : Option<String>,
    password : Option<String>,
    server : Option<String>,
    port : Option<String>,
    name : Option<String>,
    uri : Option<String>
}

trait Connect {
    fn create_uri(&self) -> Result<String, PersistenceErrorKind>;
    fn open(&self) -> Result<Client, PersistenceErrorKind>;
}


impl Connect for PostgresConfig
{

    fn create_uri(&self) -> Result<String, PersistenceErrorKind> {

        let uri = self.uri.as_ref().map_or( "", |p| p.as_str());
        if uri.is_empty() {

            let mut postgres_uri : String = "postgresql://".to_string();

            let username= self.user.as_ref().map_or("postgres", |s| s.as_str());

            let password = self.password.as_ref().map_or(String::new(), |p| format!(":{}", p));

            let sever = self.server.as_ref().map_or( format!("@localhost"), |p| format!("@{}", p));

            let port = self.port.as_ref().map_or(format!(""), |p| format!(":{}", p));
//
            let name = self.name.as_ref().map_or(format!(""), |p| format!("/{}", p));

            postgres_uri.push_str(username);

            if !password.is_empty() {
                postgres_uri.push_str(&password);
            }

            postgres_uri.push_str(sever.as_str());

            if !port.is_empty() {
                postgres_uri.push_str(port.as_str()) ;
            }
            if !name.is_empty() {
                postgres_uri.push_str(name.as_str());
            }

            Ok(postgres_uri)
        }
        else {

            Err(errors::PersistenceErrorKind::IOError)

        }


    }


    fn open(&self) -> Result<Client, PersistenceErrorKind> {

        let mut postgres_uri = self.uri.as_ref().map_or("", |p| p.as_str());

        let mut client = Client::connect(postgres_uri, NoTls).map_err(PersistenceErrorKind::IOError);

        Ok(client)
    }
}


#[cfg(test)]
mod persistence_tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_postgres_config_searlizes() {
        let demo_config = r#"{"user":"person1","password":"securepassword","server":"127.0.0.1","port":"3976","name":"wallet"}"#;
        let test_postgres_config_object : PostgresConfig =  serde_json::from_str(&demo_config).unwrap();
        assert_eq!("person1", test_postgres_config_object.user.unwrap())
    }

    #[test]
    fn test_postgres_config_() {
        let correct_postgres_uri = String::from("postgresql://person1:securepassword@127.0.0.1:3976/wallet");
        let demo_config = r#"{"user":"person1","password":"securepassword","server":"127.0.0.1","port":"3976","name":"wallet"}"#;
        let test_postgres_config_object: PostgresConfig = serde_json::from_str(&demo_config).unwrap();
        assert_eq!(test_postgres_config_object.create_uri().unwrap(), correct_postgres_uri)

    }

    #[test]
    fn test_if_uri_is_not_initialized(){

    }
}




