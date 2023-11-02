use tokio::io;
use tokio::net::TcpListener;
use std::{fs::{File, self}, io::Write};
use tokio::signal::{self, unix};

#[tokio::main]
async fn main(){
    tokio::spawn(async {
        let listener = TcpListener::bind("0.0.0.0:6002").await.expect("Failed to listen");
        loop {
            let (mut socket, _) = listener.accept().await.expect("Fail to accept");
            tokio::spawn(async move {
                let (mut rd, mut wr) = socket.split();
    
                if let Err(err) = io::copy(&mut rd, &mut wr).await {
                    println!("Faile to copy: {:?}", err);
                }
            });
        }
    });


    let pid: u32 = std::process::id();
    let mut file = File::create("server.pid").expect("Can't create pid file");
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
    let _ = fs::remove_file("server.pid");
}

