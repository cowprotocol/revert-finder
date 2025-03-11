use {
    alloy::{
        consensus::Transaction,
        primitives::{Address, TxKind},
        providers::{Provider, ProviderBuilder, ext::DebugApi},
        rpc::types::{Bundle, StateContext, TransactionRequest},
        transports::http::reqwest::Url,
    },
    clap::Parser,
};

#[derive(clap::Parser, Debug, Clone)]
#[command(about, version, long_about)]
struct Args {
    /// RPC to simulate calls with. Needs to support `debug_traceCallMany`.
    #[clap(short, long, env)]
    rpc: Url,

    /// Block in which index for last successful simulation should be found.
    #[clap(short, long, env)]
    block: u64,

    /// Contract to which the transaction should be sent.
    #[clap(short, long, env)]
    to: Address,

    /// Address that would have sent the transaction.
    #[clap(short, long, env)]
    from: Address,

    /// Calldata of the transaction (hex string with or without `0x` prefix).
    #[clap(short, long, env)]
    calldata: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // parse calldata hex string (with or without 0x prefix)
    let calldata = {
        let input = if args.calldata.starts_with("0x") {
            &args.calldata[2..]
        } else {
            &args.calldata[..]
        };
        hex::decode(input).unwrap()
    };

    // this is the transaction we wanted to submit
    let target_transaction = TransactionRequest {
        from: Some(args.from),
        to: Some(TxKind::Call(args.to)),
        input: calldata.into(),
        ..Default::default()
    };

    let web3 = ProviderBuilder::new().on_http(args.rpc);
    let block = web3
        .get_block_by_number(args.block.into())
        .await
        .unwrap()
        .unwrap();
    let num_txs = block.transactions.len();
    println!("block contains {num_txs} transactions");

    // find last index the tx would have worked by binary
    // searching through the block
    let mut low = 0;
    let mut high = num_txs;
    let mut target_tx_gas = 0;
    while low <= high {
        let mid = low + (high - low) / 2;

        let res = web3
            .debug_trace_call_many(
                vec![Bundle {
                    transactions: vec![target_transaction.clone()],
                    ..Default::default()
                }],
                StateContext {
                    block_number: Some(args.block.into()),
                    transaction_index: Some(mid.into()),
                },
                Default::default(),
            )
            .await
            .unwrap()
            .pop()
            .unwrap();
        let res = res.try_into_json_value().unwrap();
        let res = res.as_array().unwrap().first().unwrap();
        let failed = res["failed"].as_bool().unwrap();

        let label = if failed { "fails" } else { "succeeds" };
        println!("simulation on index {mid} {label}");

        if failed {
            // tx failed => ignore left half
            low = mid + 1;
        } else {
            // still works => ignore right half
            high = mid - 1;
            target_tx_gas = res["gas"].as_u64().unwrap() as u128;
        };
    }

    let last_successful_index = high;
    let rival_tx_hash = block.transactions.as_hashes().unwrap()[last_successful_index];
    let rival_tx = web3
        .get_transaction_by_hash(rival_tx_hash)
        .await
        .unwrap()
        .unwrap();
    let rival_tx_receipt = web3
        .get_transaction_receipt(rival_tx_hash)
        .await
        .unwrap()
        .unwrap();
    let rival_tx_gas = rival_tx_receipt.gas_used as u128;
    let max_fee_per_gas = rival_tx.max_fee_per_gas();
    let max_priority_fee_per_gas = rival_tx.max_priority_fee_per_gas().unwrap();
    let base_fee = block.header.base_fee_per_gas.unwrap();
    let final_prio_fee_per_gas =
        std::cmp::min(max_priority_fee_per_gas, max_fee_per_gas - base_fee as u128);

    println!("\n");
    println!("rival tx: {:?}", rival_tx_hash);
    println!("index: {last_successful_index}");
    println!("base_fee: {:?} Gwei", base_fee as f64 / 1e9);
    println!(
        "max_priority_fee: {:?} Gwei",
        max_priority_fee_per_gas as f64 / 1e9
    );
    println!("max_fee: {:?} Gwei", max_fee_per_gas as f64 / 1e9);
    println!("gas_used: {:?}", target_tx_gas);
    println!(
        "final priority_fee_per_gas: {:?} Gwei",
        final_prio_fee_per_gas as f64 / 1e9
    );
    println!(
        "final total_tip: {:?} ETH",
        (final_prio_fee_per_gas * rival_tx_gas) as f64 / 1e18
    );
}
