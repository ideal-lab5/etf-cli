pub mod etf_api {
    use ark_bls12_381::{Fr, G1Affine as G1, G2Affine as G2};
    use ark_ec::AffineRepr;
    use ark_std::{ops::Mul, test_rng, UniformRand};
    use crypto::{
        client::client::*,
        ibe::fullident::BfIbe,
        utils::{convert_to_bytes, hash_to_g1},
    };
    use parity_scale_codec::{Decode, Encode};

    #[derive(Debug, Encode, Decode, PartialEq, Eq, Clone)]
    /// Representation of an encryption result
    pub struct EncryptionResult {
        pub ciphertext: Vec<u8>,
        pub nonce: Vec<u8>,
        pub etf_ct: Vec<Vec<u8>>,
        pub secrets: Vec<Vec<u8>>,
    }

    /// Helper function to convert from AesIbeCt to EncryptionResult
    pub fn convert_to_encryption_result(
        encryption_info: AesIbeCt,
        secrets: Vec<Vec<u8>>,
    ) -> EncryptionResult {
        EncryptionResult {
            ciphertext: encryption_info.aes_ct.ciphertext,
            nonce: encryption_info.aes_ct.nonce,
            etf_ct: encryption_info.etf_ct,
            secrets,
        }
    }

    /// calculates secret keys: Q = H1(id), d = sQ
    pub fn calculate_secret_keys(ids: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let s = Fr::rand(&mut test_rng());
        ids.iter()
            .map(|id| {
                let q = hash_to_g1(id);
                let d = q.mul(s);
                convert_to_bytes::<G1, 48>(d.into()).to_vec()
            })
            .collect::<Vec<_>>()
    }

    /// encrypt wrapper
    pub fn encrypt(message: &[u8], ids: Vec<Vec<u8>>, t: u8) -> Result<AesIbeCt, ClientError> {
        let s = Fr::rand(&mut test_rng());
        let ibe_pp: G2 = G2::generator();
        let p_pub: G2 = ibe_pp.mul(s).into();
        let ibe_pp_bytes = convert_to_bytes::<G2, 96>(G2::generator());
        let p_pub_bytes = convert_to_bytes::<G2, 96>(p_pub);

        DefaultEtfClient::<BfIbe>::encrypt(
            ibe_pp_bytes.to_vec(),
            p_pub_bytes.to_vec(),
            message,
            ids,
            t,
        )
    }

    /// decrypt wrapper
    pub fn decrypt(
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        capsule: Vec<Vec<u8>>,
        secrets: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, ClientError> {
        let ibe_pp_bytes = convert_to_bytes::<G2, 96>(G2::generator());
        DefaultEtfClient::<BfIbe>::decrypt(
            ibe_pp_bytes.to_vec(),
            ciphertext,
            nonce,
            capsule,
            secrets,
        )
    }
}

#[test]
/// Tests to ensure that encrypt and decrypt functions are working properly
fn can_encrypt_and_decrypt() {
    let message = b"this is a test";
    let ids = vec![b"id1".to_vec(), b"id2".to_vec(), b"id3".to_vec()];
    let t = 2;

    match etf_api::encrypt(message, ids.clone(), t) {
        Ok(ct) => {
            let secrets = etf_api::calculate_secret_keys(ids.clone());
            match etf_api::decrypt(ct.aes_ct.ciphertext, ct.aes_ct.nonce, ct.etf_ct, secrets) {
                Ok(m) => {
                    assert_eq!(message.to_vec(), m);
                }
                Err(e) => {
                    panic!("Decryption should work but was: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Encryption should work but was {:?}", e);
        }
    }
}
