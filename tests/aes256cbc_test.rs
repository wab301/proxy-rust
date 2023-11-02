#![feature(test)]

#[cfg(test)]
mod tests {

    use proxy_rust::aes256cbc;
    use test::Bencher;
    #[test]
    fn test_encrypt_to_decrypt() {
        let secret = "123456";
        let data = aes256cbc::encrypt_string("127.0.0.1:6002".as_bytes(), secret).unwrap();
        println!("data: {}",data);

        let message = "U2FsdGVkX19BEXTlZLFOj01yP2jJ7mjZa8NIpW+JIRU=";

        println!("{:?} {:?}", aes256cbc::decrypt_string(data.as_bytes(), secret), aes256cbc::decrypt_string(message.as_bytes(), secret));
    }

    #[bench]
    fn bench_decrypt2(b: &mut Bencher) {

        b.iter(|| {
            let encrypted_data:[u8;16] = [194, 45, 245, 147, 240, 60, 67, 89, 69, 204, 74, 70, 43, 91, 52, 31];
            let key: [u8; 32] = [240, 221, 71, 127, 101, 37, 72, 182, 194, 2, 2, 6, 138, 138, 53, 119, 189, 153, 166, 14, 125, 146, 228, 24, 29, 171, 159, 142, 33, 74, 116, 88];
            let iv: [u8; 16] = [169, 170, 129, 86, 234, 95, 105, 115, 5, 217, 158, 93, 10, 49, 84, 179];
            let _ = aes256cbc::decrypt(&encrypted_data,&key,&iv);
        })
    }

    #[bench]
    fn bench_decrypt(b: &mut Bencher) {
        b.iter(|| {
            let message = b"U2FsdGVkX19BEXTlZLFOj01yP2jJ7mjZa8NIpW+JIRU=";
            let secret = "123456";

            assert_eq!(aes256cbc::decrypt_string(message, secret).unwrap(), "127.0.0.1:6002");
        })
    }

    #[bench]
    fn bench_encrypt(b: &mut Bencher) {
        b.iter(|| {
            let secret = "123456";
            let data = aes256cbc::encrypt_string(b"127.0.0.1:6002", secret).unwrap();

            assert_eq!(aes256cbc::decrypt_string(data.as_bytes(), secret).unwrap(), "127.0.0.1:6002");
        })
    }
}
