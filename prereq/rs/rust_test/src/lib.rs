pub mod prelude {
    pub use solana_sdk::signature::{Keypair, Signer, read_keypair_file};
    pub use bs58;
    pub use std::io::{self, BufRead};
    pub use solana_client::rpc_client::RpcClient;
    pub use solana_system_interface::instruction::transfer;
    pub use std::str::FromStr;
    pub use solana_sdk::{ hash::hash, pubkey::Pubkey, transaction::Transaction,message::Message };
    pub use solana_system_interface::{program as system_program};
    pub use solana_sdk::instruction::{Instruction,AccountMeta};
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    const RPC_URL: &str = "https://turbine-solanad-4cde.devnet.rpcpool.com/9a9da9cf-6db1-47dc-839a-55aca5c9c80a";

    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!("You've generated a new Solana wallet: {}\n", kp.pubkey());
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn claim_airdrop() {
        println!("preparing to claim airdrop");
        let client = RpcClient::new(RPC_URL);
        // Import keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(sig) => {
                println!("Success! Check your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet",
                sig);
            }
            Err(err) => {
                println!("Airdrop failed: {}", err);
            }
        }
    }

    #[test]
    fn transfer_sol() {
        println!("Starting to transfer SOL from dev wallet to turbin3 wallet");
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        // Generate a signature from the keypair
        let pubkey = keypair.pubkey();
        let message_bytes = b"I verify my Solana Keypair!";
        let sig = keypair.sign_message(message_bytes);
        println!("Signature: {}", sig);
        let sig_hashed = hash(sig.as_ref());
        // Verify the signature using the public key
        match sig.verify(&pubkey.to_bytes(), &sig_hashed.to_bytes()) {
            true => println!("Signature verified"),
            false => println!("Verification failed"),
        }

        let to_pubkey = Pubkey::from_str("Dbh6kfHKPm7E94ci9HeVt3nj2NLa5Syk2pzEHsG6GGon").unwrap();
        let rpc_client = RpcClient::new(RPC_URL);
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        //create an sign the transaction
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, 1_000_000)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        //Send and confirm transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }

    #[test]
    fn empty_wallet(){
        let client = RpcClient::new(RPC_URL);
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        let to_pubkey = Pubkey::from_str("Dbh6kfHKPm7E94ci9HeVt3nj2NLa5Syk2pzEHsG6GGon").unwrap();
        let recent_blockhash = client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        // get balance
        let balance = client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");
        // build mock transaction to calculate fee
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );
        //estimate fee
        let fee = client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");
        
        // create final transction
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        //send transaction and verify
        let signature = client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send final transaction");
        println!("Success! Entire balance transferred: https://explorer.solana.com/tx/{}/?cluster=devnet",signature);
    }

    #[test]
    fn submit_rs(){
        let client = RpcClient::new(RPC_URL);
        let keypair = read_keypair_file("turbine-wallet.json").expect("Couldn't find wallet file");
        println!("Proceeding to submit rust prereq to turbin3. Turbine wallet is: {}", keypair.pubkey());
        let mint = Keypair::new();
        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();
        let mpl_core_program = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = keypair.pubkey();
        let prereq_seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(prereq_seeds,&turbin3_prereq_program);
        let authority_seeds = &[b"collection", collection.as_ref()];
        let (authority, _bump) = Pubkey::find_program_address(authority_seeds,&turbin3_prereq_program);
        let data = vec![77, 124, 82, 163, 21, 133, 181, 206];

        let accounts = vec![
            AccountMeta::new(keypair.pubkey(), true), // user signer
            AccountMeta::new(prereq_pda, false), // PDA account
            AccountMeta::new(mint.pubkey(), true), // mint keypair
            AccountMeta::new(collection, false), // collection
            AccountMeta::new_readonly(authority, false), // authority (PDA)
            AccountMeta::new_readonly(mpl_core_program, false), // mpl core program
            AccountMeta::new_readonly(system_program, false), // system program
        ];

        let blockhash = client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // build the instruction
        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        //create and sign the transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&keypair.pubkey()),
            &[&keypair, &mint],
            blockhash,
        );

        //send transaction and verify
        let signature = client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        println!("Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",signature);
    }

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as a base58 string:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file format is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a JSON byte array (e.g.
        [12,34,...]):");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        println!("Your Base58-encoded private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }
}