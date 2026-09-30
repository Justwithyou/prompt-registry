use crypto::AesGcmCipher;
use prompt_core::SyncCipher;

#[test]
fn encrypt_decrypt_roundtrip() {
    let cipher = AesGcmCipher;
    let key = [7u8; 32];
    let plaintext = b"Hello sync data -- \xe4\xbd\xa0\xe5\xa5\xbd\xe5\x90\x8c\xe6\xad\xa5";

    let payload = cipher.encrypt(plaintext, &key).unwrap();
    assert_eq!(payload.version, 1);
    assert!(!payload.salt.is_empty());
    assert!(!payload.nonce.is_empty());
    assert!(!payload.ciphertext.is_empty());
    assert_ne!(payload.ciphertext.as_slice(), plaintext);

    let decrypted = cipher.decrypt(&payload, &key).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_empty_plaintext() {
    let cipher = AesGcmCipher;
    let key = [9u8; 32];
    let payload = cipher.encrypt(&[], &key).unwrap();
    let decrypted = cipher.decrypt(&payload, &key).unwrap();
    assert!(decrypted.is_empty());
}

#[test]
fn wrong_key_fails() {
    let cipher = AesGcmCipher;
    let key = [1u8; 32];
    let other = [2u8; 32];
    let payload = cipher.encrypt(b"secret", &key).unwrap();
    assert!(cipher.decrypt(&payload, &other).is_err());
}

#[test]
fn tampered_ciphertext_fails() {
    let cipher = AesGcmCipher;
    let key = [3u8; 32];
    let mut payload = cipher.encrypt(b"secret", &key).unwrap();
    let last = payload.ciphertext.len() - 1;
    payload.ciphertext[last] ^= 0xff;
    assert!(cipher.decrypt(&payload, &key).is_err());
}

#[test]
fn distinct_nonces_produce_distinct_ciphertexts() {
    let cipher = AesGcmCipher;
    let key = [4u8; 32];
    let p1 = cipher.encrypt(b"same plaintext", &key).unwrap();
    let p2 = cipher.encrypt(b"same plaintext", &key).unwrap();
    assert_ne!(p1.nonce, p2.nonce);
    assert_ne!(p1.ciphertext, p2.ciphertext);
}
