#[warn(deprecated)]
use clap::{Parser, Subcommand};
use std::fs;
use solana_pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use std::str::FromStr;
use std::error::Error;
use solana_sdk::{
    commitment_config::CommitmentConfig, lamports, native_token::{LAMPORTS_PER_SOL, lamports_to_sol},signature::{self, Keypair, Signer}, system_transaction
};


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
           let signature = client.request_airdrop(&pubkey, lamports)?;
           println!("request airdrop");
           client.poll_for_signature_confirmation(&signature, 30)?;
           println!("airdrop done dear")
        }
        Commands::Balance { address} => {
            let pubkey = Pubkey::from_str(&address)?;
            let balance_lamports = client.get_balance(&pubkey)?;
            println!("Balance: {} Sol", lamports_to_sol(balance_lamports)); // or you can divide (just using the deprecated fxn)

        }
        Commands::Transfer { from_keypair, to_pubkey, amount } =>  {
            let keypair_path = from_keypair.unwrap_or_else(|| {
                let default_path = dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".config/solana/id.json")
                    .to_str()
                    .expect("Failed to construct default keypair path")
                    .to_string();
                println!("--from-keypair not provided, using default wallet: {}", default_path);
                default_path
            });
            
            // Manually read and parse the keypair file
            let keypair_json = fs::read_to_string(&keypair_path)
                .map_err(|e| format!("Failed to read keypair file '{}': {}", keypair_path, e))?;
            
            let keypair_bytes: Vec<u8> = serde_json::from_str(&keypair_json)
                .map_err(|e| format!("Failed to parse keypair file '{}': {}", keypair_path, e))?;

            let from_signer = Keypair::from_bytes(&keypair_bytes)
                 .map_err(|e| format!("Invalid keypair data in '{}': {}", keypair_path, e))?;
            
            let from_pubkey = from_signer.pubkey();
            let to_pubkey = Pubkey::from_str(&to_pubkey)?;
            let lamports = (amount * 1_000_000_000.0) as u64;

            println!("   From: {}", from_pubkey);
            println!("   To:   {}", to_pubkey);

            let latest_blockhash = client.get_latest_blockhash()?;
            
            let tx = system_transaction::transfer(
                &from_signer,
                &to_pubkey,
                lamports,
                latest_blockhash,
            );

            let signature = client.send_and_confirm_transaction(&tx)?;
            println!("✅ Transfer successful! Signature: {}", signature);
    }
}

    Ok(())
}