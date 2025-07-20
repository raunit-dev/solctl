use clap::{Parser, Subcommand};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::{instruction as system_instruction};
use std::error::Error;
use std::fs;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author = "raunit the king", version = "6969", about = "sunday mood")]
struct Cli {
    #[arg(short, long, global = true, default_value = "https://api.devnet.solana.com")]
    rpc_url: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Balance {
        #[arg(required = true)]
        address: String,
    },
    Airdrop {
        #[arg(required = true)]
        address: String,
        #[arg(default_value = "1")]
        amount: u8,
    },
    Transfer {
        #[arg(short, long)]
        from_keypair: Option<String>,
        #[arg(short, long, required = true)]
        to_pubkey: String,
        #[arg(required = true)]
        amount: f64,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let client = RpcClient::new_with_commitment(cli.rpc_url, CommitmentConfig::confirmed());

    match cli.command {
        Commands::Airdrop { address, amount } => {
            let pubkey = Pubkey::from_str(&address)?;
            let lamports = (amount as u64) * LAMPORTS_PER_SOL;
            
            println!("Requesting airdrop...");
            let signature = client.request_airdrop(&pubkey, lamports)?;
            
            // Use the modern confirmation approach
            let mut confirmed = false;
            for _ in 0..30 {
                match client.get_signature_status(&signature)? {
                    Some(Ok(_)) => {
                        confirmed = true;
                        break;
                    }
                    Some(Err(e)) => return Err(format!("Transaction failed: {}", e).into()),
                    None => {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                }
            }
            
            if confirmed {
                println!(" Airdrop completed successfully!");
            } else {
                return Err("Transaction confirmation timeout".into());
            }
        }
        Commands::Balance { address } => {
            let pubkey = Pubkey::from_str(&address)?;
            let balance_lamports = client.get_balance(&pubkey)?;
            let balance_sol = balance_lamports as f64 / LAMPORTS_PER_SOL as f64;
            println!("Balance: {:.9} SOL", balance_sol);
        }
        Commands::Transfer {
            from_keypair,
            to_pubkey,
            amount,
        } => {
            let keypair_path = from_keypair.unwrap_or_else(|| {
                let default_path = dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".config/solana/id.json")
                    .to_str()
                    .expect("Failed to construct default keypair path")
                    .to_string();
                println!(
                    "--from-keypair not provided, using default wallet: {}",
                    default_path
                );
                default_path
            });

            // Read and parse the keypair file
            let keypair_json = fs::read_to_string(&keypair_path).map_err(|e| {
                format!("Failed to read keypair file '{}': {}", keypair_path, e)
            })?;

            let keypair_bytes: Vec<u8> = serde_json::from_str(&keypair_json).map_err(|e| {
                format!("Failed to parse keypair file '{}': {}", keypair_path, e)
            })?;

            // Use the new recommended keypair creation method
            let from_keypair = Keypair::from_bytes(&keypair_bytes).map_err(|e| {
                format!("Invalid keypair data in '{}': {}", keypair_path, e)
            })?;

            let from_pubkey = from_keypair.pubkey();
            let to_pubkey = Pubkey::from_str(&to_pubkey)?;
            let lamports = (amount * LAMPORTS_PER_SOL as f64) as u64;

            // Get the latest blockhash
            let recent_blockhash = client.get_latest_blockhash()?;

            // Use the new system interface for transfer instruction
            let transfer_instruction =
                system_instruction::transfer(&from_pubkey, &to_pubkey, lamports);

            // Create and sign the transaction using the modern approach
            let transaction = Transaction::new_signed_with_payer(
                &[transfer_instruction],
                Some(&from_pubkey),
                &[&from_keypair],
                recent_blockhash,
            );

            // Send the transaction and manually confirm
            let signature = client.send_transaction(&transaction)?;
            println!("Transaction sent with signature: {}", signature);
            
            // Manual confirmation loop
            let mut confirmed = false;
            for _ in 0..30 {
                match client.get_signature_status(&signature)? {
                    Some(Ok(_)) => {
                        confirmed = true;
                        break;
                    }
                    Some(Err(e)) => return Err(format!("Transaction failed: {}", e).into()),
                    None => {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                }
            }
            
            if confirmed {
                println!(" Transfer successful! Signature: {}", signature);
            } else {
                return Err("Transaction confirmation timeout".into());
            }
        }
    }

    Ok(())
}
