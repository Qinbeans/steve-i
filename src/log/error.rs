use chrono::Utc;
use crate::log::color::{RED, MAGENTA, CLEAR};

// var arg
pub fn errf(err_msg: &str, server_name: &str){
   //get date
   let now = Utc::now();
   let date = now.format("%Y-%m-%d %H:%M:%S").to_string();
   eprintln!("{}{}:{}<{}>{} -> {}",RED,date,MAGENTA,server_name,CLEAR,err_msg);
}