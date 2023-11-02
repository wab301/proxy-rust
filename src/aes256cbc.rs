use std::error::Error;
use std::fmt;
use crypto::{ symmetriccipher, buffer, aes, blockmodes };
use crypto::buffer::{ ReadBuffer, WriteBuffer, BufferResult };
use md5;
use rand::RngCore;

// 自定义错误类型
#[derive(Debug)]
struct CustomError {
    message: String,
}

impl CustomError {
    fn new(message: &str) -> CustomError {
        CustomError::with_string(message.to_string())
    }

    fn with_string(message: String) -> CustomError {
        CustomError {
            message: message,
        }
    }
}

// 实现 std::error::Error trait
impl Error for CustomError {
    fn description(&self) -> &str {
        &self.message
    }
}

// 实现 Display trait 以便可以使用 println! 宏打印错误消息
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomError: {}", self.message)
    }
}

const BLOCK_SIZE: usize = 16;
const OPEN_SSL_SALT_HEADER: &[u8; 8] = b"Salted__";

// Encrypt a buffer with the given key and iv using
// AES-256/CBC/Pkcs encryption.
fn encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
    // Create an encryptor instance of the best performing
    // type available for the platform.
    let mut encryptor = aes::cbc_encryptor(
            aes::KeySize::KeySize256,
            key,
            iv,
            blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;

        // "write_buffer.take_read_buffer().take_remaining()" means:
        // from the writable buffer, create a new readable buffer which
        // contains all data that has been written, and then access all
        // of that data as a slice.
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }

    Ok(final_result)
}

// Decrypts a buffer with the given key and iv using
// AES-256/CBC/Pkcs encryption.
pub fn decrypt(encrypted_data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {

    let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize256,
            key,
            iv,
            blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

pub fn encrypt_string(plaintext_string: &[u8], secret: &str) -> Result<String, Box<dyn Error>>  {
    // Generate an 8 byte salt
    let mut salt = [0u8; 8];
    let mut rng: rand::rngs::ThreadRng = rand::thread_rng();
    rng.fill_bytes(&mut salt);

    let m = extract_open_ssl_creds(secret.as_bytes().to_vec(), salt.to_vec());
    let (key,iv) = m.split_at(32);

    let encrypted_data = match encrypt(plaintext_string, key, iv) {
        Ok(data) => data,
        Err(err) => return Err(Box::new(CustomError::with_string(format!("encrypted error: {:?}", err)))),
    };

    let mut data: Vec<u8> = Vec::with_capacity(encrypted_data.len()+BLOCK_SIZE);
    data.extend(OPEN_SSL_SALT_HEADER);
    data.extend(&salt);
    data.extend(encrypted_data);

    Ok(base64::encode(data))
}

pub fn decrypt_string(message: &[u8], secret: &str) -> Result<String, Box<dyn Error>> {
    let data = base64::decode(message)?;
    
    if data.len() < BLOCK_SIZE {
        return Err(Box::new(CustomError::new("Data is too short")));
    }
    
    let (salt_header, crypto_data) = data.split_at(BLOCK_SIZE);
    let (header, salt) = salt_header.split_at(8);
    if header != OPEN_SSL_SALT_HEADER {
        return Err(Box::new(CustomError::new("Does not appear to have been encrypted with OpenSSL, salt header missing.")));
    }

    let m = extract_open_ssl_creds(secret.as_bytes().to_vec(), salt.to_vec());
    let (key,iv) = m.split_at(32);

    let decrypted_data = match decrypt(crypto_data, &key, &iv) {
        Ok(data) => data,
        Err(err) => return Err(Box::new(CustomError::with_string(format!("decrypted error: {:?}", err)))),
    };
    Ok(String::from_utf8(decrypted_data)?)
}

fn extract_open_ssl_creds<'a>(password: Vec<u8>, salt: Vec<u8>) ->Vec<u8>{
    let mut m: Vec<u8> = Vec::with_capacity(48);
    let mut prev = Vec::new();
    for _i in 0..3 {
        prev = hash(prev, password.clone(), salt.clone());
        m.append(&mut prev.clone());
    }
    
    m
}

fn hash(mut prev: Vec<u8>, mut password: Vec<u8>, mut salt: Vec<u8>) -> Vec<u8> {
    let mut a = Vec::with_capacity(prev.len() + password.len() + salt.len());
    a.append(&mut prev);
    a.append(&mut  password);
    a.append(&mut  salt);

    md5::compute(a).to_vec()
}
