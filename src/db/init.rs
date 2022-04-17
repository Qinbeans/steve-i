use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> MysqlConnection{
   dotenv().ok();
   let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set.");
   MysqlConnection::establish(&database_url).expect("Error connecting to database")
}