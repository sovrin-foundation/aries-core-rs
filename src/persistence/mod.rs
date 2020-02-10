use postgres::{Client, NoTls};
use std::{fmt, env,path::PathBuf};
use serde::{Serialize, Deserialize};



pub enum PersistenceConfig <A, B>
where
    A: AsRef<PathBuf>,
    B: Into<String>,
{
    PostgresStorage(PostgresConfig<A, B>)
}


#[derive(Serialize, Deserialize, Debug)]
struct PostgreConfig<A: AsRef<PathBuf>, B: Into<String>> {
    user : Option<B>,
    password : Option<B>,
    server : Option<B>,
    port : Option<B>,
    name : Option<B>,
}

trait Connect {
    fn open(&self) -> Result<String, Error>;
}

fn reference_string_to_str(userstring : String){
    if !userstring.is_empty() {

    }
}

impl <A, B>  Connect for PostgreConfig<A, B>
where
    A: AsRef<PathBuf>,
    B: Into<String>,
{
    fn open(&self) {

        let mut postgres_uri : String = "postgresql://".to_string();

        let username= self.name.as_ref().unwrap_or(|| String("postgres"));

        let password = self.password.as_ref().map(|p| format!(":{}", p) )
            .unwrap_or(String(""));


        let sever = self.server.as_ref().as_ref().map(|p| format!(":{}", p))
            .unwrap_or(String("@localhost"));


        let port = self.port.as_ref().map(|p| format!(":{}", p))
            .unwrap_or(String(""));

        let name = self.name.as_ref().map(|p| format!("/{}", p))
            .unwrap_or(String(""));

        postgres_uri.push_str(username);

        if !password.is_empty() {
            postgres_uri.push_str(password.as_str())
        }







    }
}