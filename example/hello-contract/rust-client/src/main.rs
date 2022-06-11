use dmd_hello_client as dmd_client;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!(
            "usage: {} <pat to solana hello world example program keypair>",
            args[0]
        );
        std::process::exit(-1);
    }
    let keypair_path = &args[1];

    let conn = dmd_client::client::establish_connection().unwrap();
    println!(
        "Connected to remote solana node running version ({}).",
        conn.get_version().unwrap()
    );

    let balance_requirement =
        dmd_client::client::get_balance_requirement(&conn).unwrap();

    println!(
        "({}) lamports are required for this transaction.",
        balance_requirement
    );

    let player = dmd_client::utils::get_player().unwrap();
    let player_balance =
        dmd_client::client::get_player_balance(&player, &conn).unwrap();
    println!("({}) lamports are owned by player.", player_balance);

    if player_balance < balance_requirement {
        let request = balance_requirement - player_balance;
        println!(
            "Player does not own sufficient lamports. Airdropping ({}) lamports.",
            request
        );
        dmd_client::client::request_airdrop(&player, &conn, request).unwrap();
    }

    let program = dmd_client::client::get_program(keypair_path, &conn).unwrap();

    dmd_client::client::create_greeting_account(&player, &program, &conn).unwrap();

    dmd_client::client::say_hello(&player, &program, &conn).unwrap();
    
    println!(
        "({}) greetings have been sent.",
        dmd_client::client::count_greetings(&player, &program, &conn).unwrap()
    )
}
