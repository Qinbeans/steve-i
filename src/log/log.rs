use std::env;
use crate::log::color::{GREEN, MAGENTA, CLEAR, YELLOW};

pub fn logf(log_msg: &str, server_name: &str){
    //get date
    let now = chrono::Utc::now();
    let date = now.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{}{}:{}<{}>{} -> {}",GREEN,date,MAGENTA,server_name,CLEAR,log_msg);
}

pub fn dbgf(log_msg: &str, server_name: &str){
    //check if debug is enabled in .env
    if let Ok(debug) = env::var("DEBUG") {
        if debug == "true" {
            let now = chrono::Utc::now();
            let date = now.format("%Y-%m-%d %H:%M:%S").to_string();
            println!("{}{}:{}<{}>{} -> {}",GREEN,date,YELLOW,server_name,CLEAR,log_msg);
        }
    }
}