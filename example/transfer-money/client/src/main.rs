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

    ensure_player_balance(&player, &conn);

    let program: Keypair =
        dt::client::get_program(program_keypair_path, &conn).unwrap();

    let _result = dt::client::request_transfer(&program, &player, &conn);

    // println!("Result is: {}", result);

    // dt::client::create_greeting_account(&player, &program, &conn).unwrap();

    // dt::client::say_hello(&player, &program, &conn).unwrap();
    
    // println!(
    //     "({}) greetings have been sent.",
    //     dt::client::count_greetings(&player, &program, &conn).unwrap()
    // )
}


// ------------------------------------------------
// Not needed for transfer MVP client
// ------------------------------------------------

fn ensure_player_balance(player: &Keypair, conn: &RpcClient) {
    let player_balance =
        dt::client::get_player_balance(&player, &conn).unwrap();
    println!("({}) lamports are owned by player.", player_balance);

    let balance_requirement =
        dt::client::get_balance_requirement(&conn).unwrap();
    println!("balance req: ({})", balance_requirement);

    if player_balance < balance_requirement {
       let request = balance_requirement - player_balance;
        println!(
            "Player does not own sufficient lamports. Airdropping ({}) lamports.",
            request
        );
        dt::client::request_airdrop(&player, &conn, request).unwrap();
    }
}
