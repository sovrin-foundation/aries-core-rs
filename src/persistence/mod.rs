/// The errors that can occur during a persistence operation
pub mod errors;
pub mod credentials;
pub mod protocol_state;

use async_trait::async_trait;
use postgres;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;
use tokio_postgres::{tls, Client, Connection, Socket};
use zeroize::*;

/// General storage configuration options
pub enum PersistenceConfig {
    /// Storage configuration used for opening a postgres database
    PostgresStorage(PostgresConfig),
    /// Storage configuration used for opening an sqlite database
    SqliteStorage(SqliteConfig),
}

/// Sqlite configuration options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqliteConfig {
    /// Path to the file. If None, the database will be opened in memory
    path: Option<PathBuf>,
    /// The flags to use when opening the sqlite connection
    flags: SqliteOpenFlags,
}

bitflags! {
    /// Flags for opening Sqlite database connections.
    #[derive(Serialize, Deserialize)]
    pub struct SqliteOpenFlags: u16 {
        /// Open in read only mode
        const READ_ONLY     = 0x0001;
        /// Open with read and write permissions
        const READ_WRITE    = 0x0002;
        /// Create the database if it doesn't exist
        const CREATE        = 0x0004;
        /// Use a URI for the database location
        const USE_URI       = 0x0008;
        /// Use an in-memory database
        const USE_MEMORY    = 0x0010;
        /// Don't allow mutexes
        const NO_MUTEX      = 0x0020;
        /// Allow mutexes during queries
        const FULL_MUTEX    = 0x0040;
        /// All connections to the database use the same cache
        const SHARED_CACHE  = 0x0080;
        /// All connections to the database independent caches
        const PRIVATE_CACHE = 0x0100;
    }
}

impl Default for SqliteOpenFlags {
    fn default() -> SqliteOpenFlags {
        SqliteOpenFlags::READ_WRITE
            | SqliteOpenFlags::CREATE
            | SqliteOpenFlags::NO_MUTEX
            | SqliteOpenFlags::USE_URI
    }
}

/// Postgres configuration options
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PostgresConfig {
    #[serde(deserialize_with = "empty_string_is_none")]
    user: Option<String>,
    #[serde(deserialize_with = "empty_string_is_none")]
    password: Option<String>,
    #[serde(deserialize_with = "empty_string_is_none")]
    server: Option<String>,
    #[serde(deserialize_with = "empty_string_is_none")]
    port: Option<String>,
    #[serde(deserialize_with = "empty_string_is_none")]
    name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_is_none")]
    uri: Option<String>,
//    #[serde(skip_deserializing,skip_serializing)]
//    client: Client,
}

pub struct PostgresPersistance {
    config : PostgresConfig,
    async_client: Result<(Client, Connection<Socket, tls::NoTlsStream>), errors::PersistenceErrorKind>,
    client : Result<postgres::Client, errors::PersistenceErrorKind>,
}

impl Zeroize for PostgresConfig {
    #[inline]
    fn zeroize(&mut self) {
        if let Some(ref mut s)  = self.password {
                s.zeroize();
        }
    }
}

fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}


/// Create new connections
#[async_trait]
trait Create {
    fn create_uri(&mut self) -> Result<Option<String>, errors::PersistenceErrorKind>;
}

#[async_trait]
trait Connect {
    fn open(&mut self) -> Result<(), errors::PersistenceErrorKind>;
    async fn async_open(
        &mut self,
    ) -> Result<(), errors::PersistenceErrorKind>;
}
#[async_trait]
impl Create for PostgresConfig {
    fn create_uri(&mut self) -> Result<Option<String>, errors::PersistenceErrorKind>{
        let uri = self.uri.as_ref().map_or("", |p| p.as_str());
        if uri.is_empty() {
            let mut postgres_uri: String = "postgresql://".to_string();
            let username = self.user.as_ref().map_or(String::from("postgres"), |s| format!("{}",s));
            let password = self
                .password
                .as_ref()
                .map_or(String::new(), |p| format!(":{}", p));
            let sever = self
                .server
                .as_ref()
                .map_or(String::from("@localhost"), |p| format!("@{}", p));
            let port = self
                .port
                .as_ref()
                .map_or(String::new(), |p| format!(":{}", p));
            let name = self
                .name
                .as_ref()
                .map_or(String::new(), |p| format!("/{}", p));

            postgres_uri.push_str(username.as_str());

            if !password.is_empty() {
                postgres_uri.push_str(&password);
            }

            postgres_uri.push_str(sever.as_str());

            if !port.is_empty() {
                postgres_uri.push_str(port.as_str());
            }
            if !name.is_empty() {
                postgres_uri.push_str(name.as_str());
            }
            self.uri = Some(postgres_uri.clone());
            Ok(Some(postgres_uri))
        } else {
            Err(errors::PersistenceErrorKind::IOError)
        }
    }
}


#[async_trait]
impl Connect for PostgresPersistance {

    fn open(&mut self) -> Result<(), errors::PersistenceErrorKind> {
        let postgres_uri = self.config.uri.as_ref().map_or("", |p| p.as_str());
        if !postgres_uri.is_empty() {
            self.client = Ok(postgres::Client::connect(postgres_uri, postgres::NoTls)
                .map_err(|_| errors::PersistenceErrorKind::IOError)?);
            Ok(())
        } else {
            let new_uri = self.config.create_uri()?;
            let new_uri = new_uri.ok_or(errors::PersistenceErrorKind::IOError)?;

            self.client = Ok(postgres::Client::connect(new_uri.as_str() , postgres::NoTls)
                .map_err(|_| errors::PersistenceErrorKind::IOError)?);

            Ok(())
        }
    }

    async fn async_open(
        &mut self,
    ) -> Result<(), errors::PersistenceErrorKind> {
        let postgres_uri = self.config.uri.as_ref().map_or("", |p| p.as_str());
        if !postgres_uri.is_empty() {
             self.async_client = match tokio_postgres::connect(postgres_uri, tokio_postgres::NoTls).await {
                Ok((client, connection)) => Ok((client, connection)),
                Err(_) => Err(errors::PersistenceErrorKind::IOError),
            };
            Ok(())
        } else {
            Err(errors::PersistenceErrorKind::IOError)
        }
    }
}




#[cfg(test)]
mod persistence_tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;

    #[test]
    fn test_postgres_config_searlizes() {
        let demo_config = r#"{"user":"person1","password":"securepassword","server":"127.0.0.1","port":"3976","name":"wallet"}"#;
        let test_postgres_config_object: PostgresConfig =
            serde_json::from_str(&demo_config).unwrap();
        assert_eq!("person1", test_postgres_config_object.user.unwrap())
    }

    #[test]
    fn test_postgres_config_searlizes_empty() {
        let demo_config = r#"{"user":"","password":"","server":"","port":"","name":"", "uri":""}"#;
        let _test_postgres_config_object: PostgresConfig =
            serde_json::from_str(&demo_config).unwrap();
    }


    #[test]
    fn test_postgres_config_() {
        let correct_postgres_uri =
            String::from("postgresql://person1:securepassword@127.0.0.1:3976/wallet");
        let demo_config = r#"{"user":"person1","password":"securepassword","server":"127.0.0.1","port":"3976","name":"wallet","uri":""}"#;
        let mut test_postgres_config_object: PostgresConfig =
            serde_json::from_str(demo_config).unwrap();
        let _conenction = match test_postgres_config_object.create_uri() {
            Ok(c) => c,
            Err(e) => panic!("{:?}", e)
        };
        assert_eq!(
            test_postgres_config_object.uri.unwrap(),
            correct_postgres_uri
        )
    }

    #[test]
    fn test_if_uri_is_not_initialized() {}

    #[test]
    fn test_open_default_wallet_and_write() {
        let demo_config = r#"{"user":"","password":"","server":"","port":"","name":"", "uri":""}"#;
        let test_postgres_config_object: PostgresConfig =
            serde_json::from_str(&demo_config).unwrap();
        let mut new_postgres_persistance = PostgresPersistance {
            config: test_postgres_config_object,
            async_client: Err(errors::PersistenceErrorKind::IOError),
            client: Err(errors::PersistenceErrorKind::IOError),

        };

        new_postgres_persistance.open().unwrap();


//        let mut default_client= test_postgres_config_object;

        let s: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .collect();


        new_postgres_persistance.client.unwrap().batch_execute(format!("
            CREATE TABLE {} (
            id      SERIAL PRIMARY KEY,
            name    TEXT NOT NULL,
            data    BYTEA
            )
        ", s).as_str()).unwrap()
    }

}
