use transfer_client as dt;
use solana_sdk::signer::keypair::{Keypair};
use solana_client::rpc_client::RpcClient;

fn args_sanity(args: &Vec<String>) {
    if args.len() != 2 {
        eprintln!(
            "usage: {} <path to program keypair>",
            args[0]
        );
        std::process::exit(-1);
    }
}

fn connect() -> solana_client::rpc_client::RpcClient {
    let conn = dt::client::establish_conn().unwrap();
    println!(
        "Connected to remote solana node running version ({}).",
        conn.get_version().unwrap()
    );
    conn
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    args_sanity(&args);
    let program_keypair_path = &args[1];
    
    let conn: RpcClient = connect();

    let player: Keypair = dt::utils::get_player().unwrap();

    let program: Keypair =
        dt::client::get_program(program_keypair_path, &conn).unwrap();

    let _result = dt::client::request_transfer(&program, &player, &conn);
}
