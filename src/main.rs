mod api;
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
    /// Use this command to send money to another account
    Transfer(Transfer),
    /// Current balance of provided account
    Balance(Balance),
    /// Create a new wallet with 100 balance
    Create(Create),
    /// Burn some tokens
    Fees(Fees),
}

#[derive(Args)]
struct Transfer {
    /// sender seed phrase (pk)
    sender_nmonic: String,
    /// sender seed password
    password: String,
    /// receiver address
    receiver: String,
    /// amount to transfer
    amount: u128,
    /// fee to pay
    fee: u128,
}

#[derive(Args)]
struct Fees {}

#[derive(Args)]
struct Balance {
    /// sender address
    address: String,
}

#[derive(Args)]
struct Create {
    /// Password to generate your wallet
    password: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Create(args) => {
            println!(
                "Creating a new wallet with 100 balance and password { }",
                args.password.trim()
            );
            let result = api::api::create_account(&args.password.trim()).await;
            match result {
                Some(account) => {
                    println!("New account created: { }", account.address);
                    println!("Mnemonic: { }", account.mnemonic);
                }
                None => {
                    println!("Error creating account");
                }
            }
        }
        Commands::Balance(args) => {
            print!("Current balance of { }", args.address);
            let address: [u8; 32] = hex::decode(args.address.as_str())
                .expect(format!("Not a valid address provided for sender").as_str())
                .try_into()
                .unwrap();
            let balance = api::api::get_balance(&address).await;
            println!("Balance of args.address: { }", balance);
        }
        Commands::Transfer(args) => {
            print!(
                "Sending money... { } { } { } { } { }",
                args.sender_nmonic, args.password, args.receiver, args.amount, args.fee
            );
            let receiver: [u8; 32] = hex::decode(args.receiver.as_str())
                .expect(format!("Not a valid address provided for receiver").as_str())
                .try_into()
                .unwrap();
            let result = api::api::transfer(
                &args.sender_nmonic,
                &args.password,
                receiver,
                args.amount,
                args.fee,
            )
            .await;
            if result == true {
                println!("Transfer successful");
            } else {
                println!("Transfer failed");
            }
        }
        Commands::Fees(_) => {
            print!("Current fees collected { }", api::api::get_fees().await);
        }
        _ => {}
    }
}
