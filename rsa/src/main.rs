use rsa::{PrivateKey, PublicKey};

fn main() {
    let mod_bits = 1 << 11;
    let msg = String::from("WHAT DOES STRAMBINI MEAN?");

    let private_key = PrivateKey::new(mod_bits).expect("Error generating skey");
    let public_key = PublicKey::new(private_key.n().clone()).expect("Error extracting pubkey");

    let c = public_key
        .encrypt(msg.as_bytes())
        .expect("error encrypting");

    let decrypted_msg = private_key.decrypt(&c).expect("error decrypting");

    let decryped_msg_string =
        String::from_utf8(decrypted_msg).expect("Error converting decrypted bytes to utf8 string");

    assert_eq!(decryped_msg_string, msg);
}
