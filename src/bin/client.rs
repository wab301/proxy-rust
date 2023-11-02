use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use structopt::StructOpt;
use std::{fs::{File, self}, io::Write};
use tokio::signal::{self, unix};

#[derive(StructOpt, Debug)]
#[structopt(name = "config")]
pub struct Config {
    #[structopt(long, default_value="sofunny", env = "SECRET")]
    pub secret: String,
    #[structopt(long, default_value = "127.0.0.1:6001", env = "PROXY_ADDR")]
    pub proxy_addr: String,
    #[structopt(long, default_value = "127.0.0.1:6002", env = "SERVER_ADDR")]
    pub server_addr: String,
    #[structopt(long, default_value = "33", env = "WAIT")]
    pub wait: u64,
    #[structopt(short, long, default_value = "10", env = "NUM")]
    pub num: u64,
}

#[tokio::main]
async fn main() {
    let config = Config::from_args();
    println!("{:?}", config);

    let address = proxy_rust::aes256cbc::encrypt_string(config.server_addr.as_bytes(), 
                            &config.secret).expect("Address entry failed")+ "\n";

    for _ in 0..config.num {
        let proxy_addr_clone = config.proxy_addr.clone();
        let server_address_clone = address.clone();
        tokio::spawn(async move {
            let mut socket = TcpStream::connect(proxy_addr_clone).await.expect("Connect failed");
            let _ = socket.write_all(server_address_clone.as_bytes()).await;
            let mut code = [0u8;3];
            let _ = socket.read(&mut code).await;

            if &code == b"200" {
                let test_data = b"ddatalllldahitap1924k1p jkdau0f9dafhj2389r hdalfy983hjlahdfa3l2hr98aldhfa98dfahfda90832lf ajhjgly890 hdalkfuy0-p02huk23*hflahfda89h2l3hr98shlkdsfay8o9f382hrla8yd8";
                let mut data = [0u8;162];
                loop {
                    socket.write_all(test_data).await.expect("Failed to write data");
                    socket.read(&mut data).await.expect("Failed to read data");
                    tokio::time::sleep(tokio::time::Duration::from_millis(config.wait)).await;
                }
            } else {
                println!("Close client with code: {:?}", String::from_utf8(code.to_vec()));
            }
        });
    }

    let pid = std::process::id();
    let mut file = File::create("client.pid").expect("Can't create pid file");
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
    let _ = fs::remove_file("client.pid"); 
}

