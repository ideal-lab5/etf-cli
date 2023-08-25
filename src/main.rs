mod etf;
use clap::{Args, Parser, Subcommand};
use parity_scale_codec::{Decode, Encode};
use sp_core::hexdisplay::AsBytesRef;
use std::fs::File;
use std::io::prelude::*;
/// Command line
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt a message using a set of slot ids and a threshold. A file name should be provided to save the encryption ouput.
    Encrypt(EncryptionDetails),
    /// Decrypt a ciphertext using the .etf file where the ciphertext all encryption details were saved>
    Decrypt(DecryptionDetails),
}

#[derive(Args)]
struct EncryptionDetails {
    /// Message to encrypt
    message: String,
    /// Slot ids
    ids: String,
    /// Threshold
    t: u8,
    /// File name to output the encryption details
    file_name: String,
}

#[derive(Args)]
struct DecryptionDetails {
    /// file path with the encryption details
    file_name: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Encrypt(args) => {
            let message = args.message.as_bytes();
            let ids = args
                .ids
                .split(' ')
                .into_iter()
                .map(|id| id.as_bytes().to_vec())
                .collect::<Vec<_>>();
            match etf::etf_api::encrypt(message, ids.clone(), args.t) {
                Ok(ct) => {
                    println!("Encryption worked!");
                    println!("Encryption ciphertext: {:?}", ct.aes_ct.ciphertext.encode());
                    println!("Encryption nonce: {:?}", ct.aes_ct.nonce.encode());
                    println!("Encryption capsule: {:?}", ct.etf_ct.encode());
                    let secrets = etf::etf_api::calculate_secret_keys(ids);
                    println!("Encryption secrets: {:?}", secrets);
                    println!("Writing details to file...");
                    let etf_file = format!("{}{}", &args.file_name, String::from(".etf"));
                    let mut file = match File::create(&etf_file) {
                        Ok(file) => file,
                        Err(e) => panic!("couldn't create {}: {}", &etf_file, e),
                    };
                    let encryption_details =
                        etf::etf_api::convert_to_encryption_result(ct, secrets);
                    file.write_all(&encryption_details.encode()[..])
                        .expect("write encryption details failed");
                    println!("File created: {}", &etf_file);
                }
                Err(e) => {
                    println!("Encryption failed: {:?}", e);
                }
            }
        }
        Commands::Decrypt(args) => {
            let mut file = match File::open(&args.file_name) {
                Ok(file) => file,
                Err(e) => panic!("couldn't open {}: {}", args.file_name, e),
            };

            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .expect("read encryption details failed");
            let encryption_details =
                etf::etf_api::EncryptionResult::decode(&mut buf.as_bytes_ref())
                    .expect("decode failed");
            println!("Encryption details: ");
            println!("ciphertext... {:?}", encryption_details.ciphertext);
            println!("nonce... {:?}", encryption_details.nonce);
            println!("capsule... {:?}", encryption_details.etf_ct);
            println!("secrets... {:?}", encryption_details.secrets);
            //TODO calls decrypt and check its result
        }
        _ => {}
    }
}
