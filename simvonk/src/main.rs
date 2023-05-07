use std::sync::mpsc;
use std::thread;

use common::data::Data;
use common::listener::ConnectionInformation;

mod controller;

const ADDRESS: &str = "0.0.0.0:7200";
const FRAMES_PER_SECOND: f64 = 50.0;
const WELCOME_MESSAGE: &str = r#"
                                                       888
    8P d8P  ,e e,   ,e e,  Y8b Y888P  e88 88e  888 8e  888 ee
    P d8P  d88 88b d88 88b  Y8b Y8P  d888 888b 888 88b 888 P
     d8P d 888   , 888   ,   Y8b "   Y888 888P 888 888 888 b
    d8P d8  "KoeN"  "BauX"    Y8P     "88 88"  888 888 888 8b
    
                By Koen & Bauke Westendorp, 2023.
"#;

fn main() {
    println!("{WELCOME_MESSAGE}");
    let (sender, receiver) = mpsc::channel::<(ConnectionInformation, Data)>();

    thread::spawn(move || controller::controller(receiver));
    common::listener::listener(ADDRESS, sender)
}
