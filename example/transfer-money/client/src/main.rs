use transfer_client as dt;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!(
            "usage: {} <path to solana hello world example program keypair>",
            args[0]
        );
        std::process::exit(-1);
    }
    let keypair_path = &args[1];

    let conn = dt::client::establish_conn().unwrap();
    println!(
        "Connected to remote solana node running version ({}).",
        conn.get_version().unwrap()
    );

    let balance_requirement =
        dt::client::get_balance_requirement(&conn).unwrap();

    println!(
        "({}) lamports are required for this transaction.",
        balance_requirement
    );

    let player = dt::utils::get_player().unwrap();
    let player_balance =
        dt::client::get_player_balance(&player, &conn).unwrap();
    println!("({}) lamports are owned by player.", player_balance);

    if player_balance < balance_requirement {
        let request = balance_requirement - player_balance;
        println!(
            "Player does not own sufficient lamports. Airdropping ({}) lamports.",
            request
        );
        dt::client::request_airdrop(&player, &conn, request).unwrap();
    }

    let program = dt::client::get_program(keypair_path, &conn).unwrap();

    dt::client::create_greeting_account(&player, &program, &conn).unwrap();

    dt::client::say_hello(&player, &program, &conn).unwrap();
    
    println!(
        "({}) greetings have been sent.",
        dt::client::count_greetings(&player, &program, &conn).unwrap()
    )
}
