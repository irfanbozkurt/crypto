use rsa::PrivateKey;

fn main() {
    let mod_bits = 1 << 11;
    let plaintext = String::from(
        "WHAT DOES STRAMBINI MEAN? Try googling this surname and you won't find anything. Uncommon surname.",
    );

    let private_key = PrivateKey::new(mod_bits).expect("Error generating skey");
    let public_key = private_key
        .get_public_key()
        .expect("Error extracting pubkey");

    let c = public_key
        .encrypt(plaintext.as_bytes())
        .expect("error encrypting");

    let decrypted_msg_bytes = private_key.decrypt(&c).expect("error decrypting");

    let decryped_msg = String::from_utf8(decrypted_msg_bytes)
        .expect("Error converting decrypted bytes to utf8 string");

    assert_eq!(decryped_msg, plaintext);
}
