/// The errors that can occur during a persistence operation
pub mod errors;

use async_trait::async_trait;
use postgres;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio_postgres::{tls, Client, Connection, Socket};

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
#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresConfig {
    user: Option<String>,
    password: Option<String>,
    server: Option<String>,
    port: Option<String>,
    name: Option<String>,
    uri: Option<String>,
}

/// Create new connections
#[async_trait]
trait Connect {
    fn create_uri(&self) -> Result<String, errors::PersistenceErrorKind>;
    fn open(&self) -> Result<postgres::Client, errors::PersistenceErrorKind>;
    async fn async_open(
        &self,
    ) -> Result<(Client, Connection<Socket, tls::NoTlsStream>), errors::PersistenceErrorKind>;
}

#[async_trait]
impl Connect for PostgresConfig {
    fn create_uri(&self) -> Result<String, errors::PersistenceErrorKind> {
        let uri = self.uri.as_ref().map_or("", |p| p.as_str());
        if uri.is_empty() {
            let mut postgres_uri: String = "postgresql://".to_string();
            let username = self.user.as_ref().map_or("postgres", |s| s.as_str());
            let password = self
                .password
                .as_ref()
                .map_or(String::new(), |p| format!(":{}", p));
            let sever = self
                .server
                .as_ref()
                .map_or(format!("@localhost"), |p| format!("@{}", p));
            let port = self
                .port
                .as_ref()
                .map_or(String::new(), |p| format!(":{}", p));
            let name = self
                .name
                .as_ref()
                .map_or(String::new(), |p| format!("/{}", p));

            postgres_uri.push_str(username);

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

            Ok(postgres_uri)
        } else {
            Err(errors::PersistenceErrorKind::IOError)
        }
    }

    fn open(&self) -> Result<postgres::Client, errors::PersistenceErrorKind> {
        let postgres_uri = self.uri.as_ref().map_or("", |p| p.as_str());
        if !postgres_uri.is_empty() {
            let client = postgres::Client::connect(postgres_uri, postgres::NoTls)
                .map_err(|_| errors::PersistenceErrorKind::IOError)?;
            Ok(client)
        } else {
            Err(errors::PersistenceErrorKind::IOError)
        }
    }

    async fn async_open(
        &self,
    ) -> Result<(Client, Connection<Socket, tls::NoTlsStream>), errors::PersistenceErrorKind> {
        let postgres_uri = self.uri.as_ref().map_or("", |p| p.as_str());
        if !postgres_uri.is_empty() {
            match tokio_postgres::connect(postgres_uri, tokio_postgres::NoTls).await {
                Ok((client, connection)) => Ok((client, connection)),
                Err(_) => Err(errors::PersistenceErrorKind::IOError),
            }
        } else {
            Err(errors::PersistenceErrorKind::IOError)
        }
    }
}

#[cfg(test)]
mod persistence_tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_postgres_config_searlizes() {
        let demo_config = r#"{"user":"person1","password":"securepassword","server":"127.0.0.1","port":"3976","name":"wallet"}"#;
        let test_postgres_config_object: PostgresConfig =
            serde_json::from_str(&demo_config).unwrap();
        assert_eq!("person1", test_postgres_config_object.user.unwrap())
    }

    #[test]
    fn test_postgres_config_() {
        let correct_postgres_uri =
            String::from("postgresql://person1:securepassword@127.0.0.1:3976/wallet");
        let demo_config = r#"{"user":"person1","password":"securepassword","server":"127.0.0.1","port":"3976","name":"wallet"}"#;
        let test_postgres_config_object: PostgresConfig =
            serde_json::from_str(&demo_config).unwrap();
        assert_eq!(
            test_postgres_config_object.create_uri().unwrap(),
            correct_postgres_uri
        )
    }

    #[test]
    fn test_if_uri_is_not_initialized() {}
}
