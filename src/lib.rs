pub use structopt::StructOpt;
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}, time::{Duration, timeout}};
use bytes::BytesMut;
pub mod aes256cbc;

const CODE_OK: &[u8] = "200".as_bytes();
const CODE_BAD_REQ: &[u8] = "400".as_bytes();
const CODE_BAD_ADDR: &[u8] = "401".as_bytes();
const CODE_DIAL_ERR: &[u8] = "502".as_bytes();
const CODE_DIAL_TIMEOUT: &[u8] = "504".as_bytes();

#[derive(StructOpt, Debug)]
#[structopt(name = "config")]
pub struct Config {
    #[structopt(long, default_value="PS8Ujwuv", env = "SECRET")]
    pub secret: String,
    #[structopt(long, default_value = "0.0.0.0:6001", env = "ADDR")]
    pub addr: String,
    #[structopt(long, env = "REUSE")]
    pub reuse: bool,
    #[structopt(long, default_value = "3", env = "RETRY")]
    pub retry: u8,
    #[structopt(short, long, default_value = "5", env = "TIMEOUT")]
    pub timeout: u64,
}

#[derive(Debug)]
pub struct Proxy {
    config: Config
}

impl Proxy {
    pub fn new() ->Proxy {
        let opt = Config::from_args();
        println!("{:?}", opt);
        Proxy { config: opt }
    }

    pub async fn start(&self) {
        let listener = TcpListener::bind(&self.config.addr).await.unwrap();

        let retry = self.config.retry;
        let secret =  self.config.secret.clone();
        let timeout_sec =  self.config.timeout;
        tokio::spawn(async move {
            loop {
                let (socket,_) = listener.accept().await.unwrap();
                let secret_clone = secret.clone();
                tokio::spawn(async move {
                    Proxy::handle(socket, timeout_sec, retry, &secret_clone).await;
                });
            }
        });
    }

    async fn handle( socket: TcpStream, timeout_sec: u64, retry: u8, secret: &str) {
        if let Some((socket,agent)) = Proxy::handshake(socket, timeout_sec, retry, secret).await {
                let (mut rd,mut wr) = tokio::io::split(socket);
                let (mut agent_rd, mut agent_wr) = tokio::io::split(agent);
                tokio::spawn(async move {
                    match tokio::io::copy(&mut agent_rd, &mut wr).await {
                        Ok(_) => {},
                        Err(err) => {
                            if err.kind() != std::io::ErrorKind::BrokenPipe {
                                println!("copy data to client error: {}",err);
                            }
                        },
                    }
                });
                match tokio::io::copy(&mut rd, &mut agent_wr).await {
                    Ok(_) => {},
                    Err(err) => {
                            if err.kind() != std::io::ErrorKind::ConnectionReset {
                                println!("copy data to agent error: {}",err);
                            }
                        },
                }
        }
    }

    async fn handshake(mut socket: TcpStream, timeout_sec: u64, retry: u8, secret: &str) -> Option<(TcpStream,TcpStream)> {
        let mut buf = BytesMut::with_capacity(64);
        let (addr, remain) = loop {
            if let Some(num) = buf.iter().position(|b| *b == b'\n') {
                let encrypt = &buf[..num];

                if let Ok(address) = aes256cbc::decrypt_string(encrypt, secret) {
                    break (address, &buf[num+1..]);
                }
                let _ = socket.write(CODE_BAD_ADDR).await;
                return None;
            }

            match socket.read_buf(&mut buf).await {
                Ok(size) => {
                    if size == 0 {
                        return None;
                    }
                },
                Err(_) => { let _ = socket.write(CODE_BAD_REQ).await; return None},
            };
        };

        let timeout_duration = Duration::from_secs(timeout_sec);
        for _ in 0..retry {
            let result =  timeout(timeout_duration, TcpStream::connect(addr.clone())).await;
            match result {
                Ok(Ok(mut agent_socket)) => {
                    if socket.write(CODE_OK).await.is_err() {
                        return None;
                    }

                    if remain.len() > 0 {
                        if agent_socket.write(remain).await.is_err() {
                            return None;
                        }
                    }

                    return Some((socket, agent_socket))
                },
                Ok(Err(_)) => {
                    // 连接失败
                    let _ = socket.write(CODE_DIAL_ERR).await; 
                    return None;
                },
                Err(_) => {
                    // 连接超时
                },
            }
        }

        // 连接超时
        let _ = socket.write(CODE_DIAL_TIMEOUT).await;
        return None;
    }
}

