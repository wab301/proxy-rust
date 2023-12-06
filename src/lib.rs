pub use structopt::StructOpt;
use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}, time::{Duration, timeout}};
use bytes::BytesMut;
pub mod aes256cbc;
mod copy_bidirectional;

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

    async fn handle(mut socket: TcpStream, timeout_sec: u64, retry: u8, secret: &str) {
        let mut buf = BytesMut::with_capacity(64);
        let (addr, remain) = loop {
            if let Some(num) = buf.iter().position(|b| *b == b'\n') {
                let encrypt = &buf[..num];

                if let Ok(address) = aes256cbc::decrypt_string(encrypt, secret) {
                    break (address, &buf[num+1..]);
                }
                let _ = socket.write(CODE_BAD_ADDR).await;
                let _ = Proxy::close_stream(socket);
                return;
            }

            match socket.read_buf(&mut buf).await {
                Ok(size) => {
                    if size == 0 {
                        let _ = Proxy::close_stream(socket);
                        return;
                    }
                },
                Err(_) => { 
                    let _ = socket.write(CODE_BAD_REQ).await;
                    let _ = Proxy::close_stream(socket);
                    return;
                },
            };
        };

        let timeout_duration = Duration::from_secs(timeout_sec);
        for _ in 0..retry {
            let result =  timeout(timeout_duration, TcpStream::connect(addr.clone())).await;
            match result {
                Ok(Ok(mut agent_socket)) => {
                    if socket.write(CODE_OK).await.is_err() {
                        let _ = Proxy::close_stream(agent_socket);
                        let _ = Proxy::close_stream(socket);
                        return;
                    }

                    if remain.len() > 0 {
                        if agent_socket.write(remain).await.is_err() {
                            let _ = Proxy::close_stream(agent_socket);
                            let _ = Proxy::close_stream(socket);
                            return;
                        }
                    }

                    let _ = copy_bidirectional::copy_bidirectional(&mut socket,&mut agent_socket).await;
                    let _ = Proxy::close_stream(agent_socket);
                    let _ = Proxy::close_stream(socket);
                    return;
                },
                Ok(Err(_)) => {
                    // 连接失败
                    let _ = socket.write(CODE_DIAL_ERR).await;
                    let _ = Proxy::close_stream(socket);
                    return;
                },
                Err(_) => {
                    // 连接超时
                },
            }
        }

        // 连接超时
        let _ = socket.write(CODE_DIAL_TIMEOUT).await;
        let _ = Proxy::close_stream(socket);
    }

    fn close_stream(socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        socket.into_std()?.shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }
}
