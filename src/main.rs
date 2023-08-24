mod etf;
use clap::{Args, Parser, Subcommand};
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
    /// Encrypt a message using a set of slot ids and a threshold
    Encrypt(EncryptionDetails),
    /// Decrypt a cipher text
    Decrypt(DecryptionDetails),
}

#[derive(Args)]
struct DecryptionDetails {
    /// cipher text to be decripted
    ciphertext: String,
}

#[derive(Args)]
struct EncryptionDetails {
    /// Message to encrypt
    message: String,
    /// Slot ids
    ids: String,
    /// Threshold
    t: u8,
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
                    print!("Encryption ciphertext: {:?}", ct.aes_ct.ciphertext);
                    print!("Encryption nonce: {:?}", ct.aes_ct.nonce);
                    print!("Encryption capsule: {:?}", ct.etf_ct);
                    let secrets = etf::etf_api::calculate_secret_keys(ids);
                    print!("Encryption secrets: {:?}", secrets);
                }
                Err(e) => {
                    println!("Encryption failed: {:?}", e);
                }
            }
        }
        Commands::Decrypt(args) => {
            print!("Decrypting... { }", args.ciphertext);
            //TODO calls decrypt and check its result
        }
        _ => {}
    }
}
