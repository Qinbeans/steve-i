use crate::log::color::{GREEN, MAGENTA, CLEAR};

pub fn logf(log_msg: &str, server_name: &str){
    //get date
    let now = chrono::Utc::now();
    let date = now.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{}{}:{}<{}>{} -> {}",GREEN,date,MAGENTA,server_name,CLEAR,log_msg);
}