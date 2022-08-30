use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use crate::log::error::errf;

const MAX_ATTEMPTS: u32 = 5;

pub fn establish_connection(attempts: u32) -> Option<MysqlConnection>{
   if attempts >= MAX_ATTEMPTS{
      errf(&format!("Too many attempts made, giving up."), "DB");
      return None;
   }
   let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set.");
   let results = MysqlConnection::establish(&database_url);
   if results.is_err(){
      errf(&format!("{:?}, Will wait 5s...",results.err().unwrap()), "DB");
      sleep(Duration::from_secs(5));
      return establish_connection(attempts+1);
   }
   //guard case above makes this possible
   Some(results.ok().unwrap())
}