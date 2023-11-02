use std::{fs::{File, self}, io::Write};

use proxy_rust::Proxy;
use tokio::signal::{self, unix};

#[tokio::main]
async fn main() {
    let proxy =  Proxy::new();

    proxy.start().await;
    println!("Start listening");

    let pid = std::process::id();
    let mut file = File::create("proxy-rust.pid").expect("Can't create pid file");
    write!(file,"{}",pid).expect("Can't write pid file");

    let mut stream = unix::signal(unix::SignalKind::terminate())
                                .expect("Failed to create SIGTERM signal stream");
    tokio::select! {
        _ =  signal::ctrl_c() => { 
            println!("ctrl-c received!");
        },
        _ = stream.recv() => {
            println!("Got signal terminate");
        },
    };
    let _ = fs::remove_file("proxy-rust.pid"); 
}