use crate::utils;
use crate::{Error, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::Message;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::transaction::Transaction;

/// Establishes an RPC conn. See `solana config set --url <URL>`.
/// See the solana config file `~/.config/solana/cli/config.yml`.
pub fn establish_conn() -> Result<RpcClient> {
    let rpc_url = utils::get_rpc_url()?;
    Ok(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    ))
}

// pub fn request_transfer(program: &Keypair,
//                         source: &Keypair,
//                         destination: &Pubkey,
//                         conn: &RpcClient) -> Result<()> {
//     let greeting_pubkey
// }

// pub fn say_hello(player: &Keypair,
//                  program: &Keypair,
//                  conn: &RpcClient) -> Result<()> {
//     let greeting_pubkey =
//         utils::get_greeting_public_key(&player.pubkey(), &program.pubkey())?;
//     let instruction = Instruction::new_with_bytes(
//         program.pubkey(),
//         &[],
//         vec![AccountMeta::new(greeting_pubkey, false)],
//     );
//     let message = Message::new(&[instruction], Some(&player.pubkey()));
//     let transaction =
//         Transaction::new(&[player], message, conn.get_latest_blockhash()?);

//     conn.send_and_confirm_transaction(&transaction)?;

//     Ok(())
// }


// ---------------------------------------------------
// Code that is not needed for the MVP Client
// ---------------------------------------------------

/// Calculate execution fee plus balance for rent-free.
pub fn get_balance_requirement(conn: &RpcClient) -> Result<u64> {
    let min_balance = get_rent_exempt_balance(conn).unwrap();
    
    let (_, fee_calculator) = conn.get_recent_blockhash()?;
    let transaction_fee = fee_calculator.lamports_per_signature * 100;

    Ok(transaction_fee + min_balance)
}

/// Get balance of PLAYER in lamports via RPC over CONN.
pub fn get_player_balance(player: &Keypair,
                          conn: &RpcClient) -> Result<u64> {
    Ok(conn.get_balance(&player.pubkey())?)
}

/// Request for amount ETH airdrop to PLAYER.
pub fn request_airdrop(player: &Keypair,
                       conn: &RpcClient,
                       amount: u64) -> Result<()> {
    let sig = conn.request_airdrop(&player.pubkey(), amount)?;
    loop {
        let confirmed = conn.confirm_transaction(&sig)?;
        if confirmed {
            break;
        }
    }
    Ok(())
}

/// Loads program keypair from disk, verifies the keypair corresponds
/// to an executable account. Error on failure.
pub fn get_program(kp_path: &str,
                   conn: &RpcClient) -> Result<Keypair> {
    let program_kp = read_keypair_file(kp_path).map_err(|e| {
        Error::InvalidConfig(format!(
            "read fail on program keypair ({}): ({})", kp_path, e
        ))
    })?;

    let program_info = conn.get_account(&program_kp.pubkey())?;
    if !program_info.executable {
        return Err(Error::InvalidConfig(format!(
            "keypair ({}) is not executable", kp_path
        )));
    }

    Ok(program_kp)
}

// -------------------------------------------------------
// Code That is Still Greeting Coupled
// -------------------------------------------------------

/// Solana charges rent, but not if you have enough in the account
/// to pay for two years of rent.
/// See: https://docs.solana.com/implemented-proposals/rent
pub fn get_rent_exempt_balance(conn: &RpcClient) -> Result<u64> {
    let greet_size = utils::get_greeting_data_size()?;
    let min_balance =
        conn.get_minimum_balance_for_rent_exemption(greet_size)?;
    Ok(min_balance)
}

/// The greeting account has a [derived address](https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses)
/// which allows it to own and manage the account. Additionally, the
/// address being derived means that we can regenerate it when we'd
/// like, to find the greeting account again later.
pub fn create_greeting_account(player: &Keypair,
                               program: &Keypair,
                               conn: &RpcClient) -> Result<()> {
    let greeting_pubkey =
        utils::get_greeting_public_key(&player.pubkey(), &program.pubkey())?;

    if let Err(_) = conn.get_account(&greeting_pubkey) {
        println!("creating greeting account");
        let starting_balance = get_rent_exempt_balance(conn).unwrap();

        // This instruction creates an account with the key
        // "greeting_pubkey". The created account is owned by the
        // program. The account is loaded with enough lamports to stop
        // it from needing to pay rent. The lamports to fund this are
        // paid by the player.
        //
        // It is important that the program owns the created account
        // because it needs to be able to modify its contents.
        //
        // The address of the account created by
        // create_account_with_seed is the same as the address
        // generated by utils::get_greeting_public_key. We do this as
        // opposed to create_account because create_account doesn't
        // use a derived address.
        let instruction = solana_sdk::system_instruction::create_account_with_seed(
            &player.pubkey(),
            &greeting_pubkey,
            &player.pubkey(),
            utils::get_greeting_seed(),
            starting_balance,
            utils::get_greeting_data_size()? as u64,
            &program.pubkey(),
        );
        let message = Message::new(&[instruction], Some(&player.pubkey()));
        let transaction =
            Transaction::new(&[player], message, conn.get_latest_blockhash()?);
        
        conn.send_and_confirm_transaction(&transaction)?;
    }

    Ok(())
}

/// Sends an instruction from PLAYER to PROGRAM via CONNECTION. The
/// instruction contains no data but does contain the address of our
/// previously generated greeting account. The program will use that
/// passed in address to update its greeting counter after verifying
/// that it owns the account that we have passed in.
pub fn say_hello(player: &Keypair,
                 program: &Keypair,
                 conn: &RpcClient) -> Result<()> {
    let greeting_pubkey =
        utils::get_greeting_public_key(&player.pubkey(), &program.pubkey())?;

    // Submit an instruction to the chain which tells the program to
    // run. We pass the account that we want the results to be stored
    // in as one of the accounts arguments which the program will
    // handle.

    let instruction = Instruction::new_with_bytes(
        program.pubkey(),
        &[],
        vec![AccountMeta::new(greeting_pubkey, false)],
    );
    let message = Message::new(&[instruction], Some(&player.pubkey()));
    let transaction =
        Transaction::new(&[player], message, conn.get_latest_blockhash()?);

    conn.send_and_confirm_transaction(&transaction)?;

    Ok(())
}

/// Pulls down the greeting account data and the value of its counter
/// which ought to track how many times the 'say_hello' method has
/// been run.
pub fn count_greetings(player: &Keypair,
                       program: &Keypair,
                       conn: &RpcClient) -> Result<u32> {
    let greeting_pubkey =
        utils::get_greeting_public_key(&player.pubkey(), &program.pubkey())?;
    let greeting_account = conn.get_account(&greeting_pubkey)?;
    Ok(utils::get_greeting_count(&greeting_account.data)?)
}
