pub mod api {
    use bip39::{Language, Mnemonic, MnemonicType};
    use parity_scale_codec::{Decode, Encode};
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use sp_core::{
        hexdisplay::HexDisplay,
        sr25519::{self, Signature},
        Pair, H512,
    };

    /// The URL of the node to connect to
    pub const URL: &str = "http://127.0.0.1:9944";
    /// key used to save fees collected
    const FEES_KEY: &[u8] = b"FEES_KEY";

    /// The runtime call enum with possible calls / extrinsics
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    pub enum Call {
        SetValue(u32),
        Transfer([u8; 32], [u8; 32], u128, u128),
        Mint([u8; 32], u128),
        Burn([u8; 32], u128),
        Upgrade(Vec<u8>),
    }

    /// Basic extrinsic type
    #[derive(Debug, Encode, Decode, PartialEq, Eq, Clone)]
    pub struct BasicExtrinsic(Call, Option<H512>);

    /// Helper functions to create extrinsics
    impl BasicExtrinsic {
        pub fn new_unsigned(call: Call) -> Self {
            <Self as sp_runtime::traits::Extrinsic>::new(call, None).unwrap()
        }
        pub fn new_signed(call: Call, signature: Signature) -> Self {
            <Self as sp_runtime::traits::Extrinsic>::new(call, Some(signature.into())).unwrap()
        }
    }

    impl sp_runtime::traits::Extrinsic for BasicExtrinsic {
        type Call = Call;
        type SignaturePayload = H512;

        fn new(data: Self::Call, signature: Option<Self::SignaturePayload>) -> Option<Self> {
            Some(Self(data, signature))
        }
    }

    /// The response from a call to the node
    #[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct CallResponse {
        jsonrpc: String,
        result: String,
        id: u8,
    }
    pub struct NewAccountInfo {
        pub address: String,
        pub mnemonic: String,
    }

    /// Sends a post request to the node specified in the URL
    pub async fn execute_rpc_request(req: String) -> CallResponse {
        let client: Client = reqwest::Client::new();
        let result = client
            .post(URL)
            .body(req)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|_| ())
            .unwrap();

        let result_text = result.text().await.unwrap();
        match serde_json::from_str(&result_text) {
            Ok(v) => v,
            Err(e) => CallResponse {
                jsonrpc: "2.0".to_string(),
                result: "0x0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                id: 100u8,
            },
        }
    }

    /// Get a value from the runtime storage using its key
    pub async fn get_from_storage(storage_key: &[u8]) -> CallResponse {
        let req = format!(
            r#"{{"jsonrpc":"2.0", "id": 1, "method":"state_getStorage", "params": ["{}"]}}"#,
            HexDisplay::from(&storage_key)
        );
        execute_rpc_request(String::from(req)).await
    }

    /// Decode a hex string representing an encoded balance into a u128
    pub fn decode_into_u128(balance: &str) -> u128 {
        let decoded_bytes = hex::decode(&balance[2..]).unwrap();
        let decoded_number: u128 = Decode::decode(&mut &decoded_bytes[..]).unwrap();
        decoded_number
    }

    /// Get balance of an account
    pub async fn get_balance(address: &[u8]) -> u128 {
        let parsed_result = get_from_storage(address).await;
        decode_into_u128(&parsed_result.result)
    }

    pub fn generate_phrase() -> String {
        //create a new randomly generated mnemonic phrase
        let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
        //get the phrase
        String::from(mnemonic.phrase())
    }

    pub async fn get_fees() -> u128 {
        let parsed_result = get_from_storage(FEES_KEY).await;
        decode_into_u128(&parsed_result.result)
    }

    /// Creates a new account with 100 balance using the provided seed
    pub async fn create_account(password: &str) -> Option<NewAccountInfo> {
        let seed = generate_phrase();
        let (pair, _seed) = sr25519::Pair::from_phrase(&seed, Some(password)).unwrap();
        let public = pair.public();
        let call = Call::Mint(*public.as_array_ref(), 100u128);
        let encoded_tx = call.encode();
        let signature = pair.sign(&encoded_tx[..]).into();
        let ext_mint = BasicExtrinsic::new_signed(call, signature);
        let req_mint = format!(
            r#"{{"jsonrpc":"2.0", "id": 1, "method":"author_submitExtrinsic", "params": ["{:?}"]}}"#,
            HexDisplay::from(&ext_mint.encode())
        );
        let result = execute_rpc_request(String::from(req_mint)).await;
        //a better business logic would be to check the result of the transaction comparing
        // balances before and after. This is good enough for this demo
        match result.id == 100u8 {
            true => None,
            false => {
                let public_address = HexDisplay::from(public.as_array_ref()).to_string();
                Some(NewAccountInfo {
                    address: public_address,
                    mnemonic: seed,
                })
            }
        }
    }

    /// Send funds from one account to another
    pub async fn transfer(
        seed: &str,
        password: &str,
        receiver: [u8; 32],
        amount: u128,
        fees: u128,
    ) -> bool {
        let (pair, _seed) = sr25519::Pair::from_phrase(&seed, Some(password)).unwrap();
        let public = pair.public();
        let call = Call::Transfer(public.as_array_ref().clone(), receiver, amount, fees);
        let encoded_tx = call.encode();
        let signature = pair.sign(&encoded_tx[..]).into();
        let ext_mint = BasicExtrinsic::new_signed(call, signature);
        let req_mint = format!(
            r#"{{"jsonrpc":"2.0", "id": 1, "method":"author_submitExtrinsic", "params": ["{:?}"]}}"#,
            HexDisplay::from(&ext_mint.encode())
        );

        let result = execute_rpc_request(String::from(req_mint)).await;
        //a better business logic would be to check the result of the transaction comparing
        // balances before and after. This is good enough for this demo
        result.id != 100u8
    }
}


use api::*;
/// Test cases
use parity_scale_codec::{Decode, Encode};
use sp_core::{
    hexdisplay::HexDisplay,
    sr25519::{self, Signature},
    Pair, H512,
};

#[test]
fn generate_phrase_test() {
    let phrase = generate_phrase();
    println!("phrase: {}", phrase);
}

#[tokio::test]
async fn parse_balance_response_test() {
    let balance = "0xdc050000000000000000000000000000";
    let decoded_bytes = hex::decode(&balance[2..]).unwrap();
    print!("original vector: {:?}", decoded_bytes);
    let decoded_number: u128 = Decode::decode(&mut &decoded_bytes[..]).unwrap();
    print!("original: {}", decoded_number);
    let decoded_using_function = decode_into_u128(balance);
    assert_eq!(decoded_number, decoded_using_function);
}
#[tokio::test]
async fn parse_storage_balance_response_test() {
    use sp_core::crypto::Pair;
    let pair = sr25519::Pair::from_seed(&[42u8; 32]);
    let public = pair.public();
    let balance = get_balance(public.as_array_ref()).await;
    print!("balance: {}", balance)
}

#[tokio::test]
async fn extrinsic_request_test() {
    /// The key used to save SetValue in the runtime storage
    pub const SET_VALUE_KEY: &[u8] = b"VALUE_KEY";
    let ext = BasicExtrinsic::new_unsigned(Call::SetValue(52));
    println!("Starting...");
    let req = format!(
        r#"{{"jsonrpc":"2.0", "id": 1, "method":"author_submitExtrinsic", "params": ["{:?}"]}}"#,
        HexDisplay::from(&ext.encode())
    );
    println!("ext {}", req);
    let mut result = execute_rpc_request(String::from(req)).await;
    println!("result: {:?}", result);
    result = get_from_storage(SET_VALUE_KEY).await;
    println!("result: {:?}", result);

    //mint
    use sp_core::crypto::Pair;
    let seed: Vec<_> = [
        "congress", "ill", "cluster", "render", "border", "piano", "embark", "age", "gloom", "hat",
        "card", "buffalo", "decorate", "ghost", "deliver", "obvious", "armed", "uniform", "need",
        "multiply", "rather", "exile", "receive", "panic",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    let (pair, _seed) = sr25519::Pair::from_phrase(&seed.join(" "), None).unwrap();
    let public = pair.public();
    print!("public address: {:?}", public);
    let call = Call::Mint(*public.as_array_ref(), 100u128);
    let encoded_tx = call.encode();
    let signature = pair.sign(&encoded_tx[..]).into();
    let ext_mint = BasicExtrinsic::new_signed(call, signature);
    let req_mint = format!(
        r#"{{"jsonrpc":"2.0", "id": 1, "method":"author_submitExtrinsic", "params": ["{:?}"]}}"#,
        HexDisplay::from(&ext_mint.encode())
    );
    result = execute_rpc_request(String::from(req_mint)).await;
    println!("result mint: {:?}", result);
}
