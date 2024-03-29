use std::sync::mpsc;
use std::thread;

use common::data::Data;

mod renderer;

const ADDRESS: &str = "0.0.0.0:7200";
const FRAMES_PER_SECOND: f64 = 50.0;
const WELCOME_MESSAGE: &str = r#"
          ,e,                                         888    
     dP"Y  "  888 888 8e  Y8b Y888P  e88 88e  888 8e  888 ee 
    C88b  888 888 888 88b  Y8b Y8P  d888 888b 888 88b 888 P  
     Y88D 888 888 888 888   Y8b "   Y888 888P 888 888 888 b  
    d,dP  888 888 888 888    Y8P     "88 88"  888 888 888 8b 
    
                By Koen & Bauke Westendorp, 2023.
"#;

fn main() {
    println!("{WELCOME_MESSAGE}");
    let (sender, receiver) = mpsc::channel::<Data>();

    thread::spawn(move || renderer::render(receiver));
    common::listener::listener(ADDRESS, sender)
}
