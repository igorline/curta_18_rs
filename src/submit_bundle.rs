use std::{env, str::FromStr, sync::Arc};

use dotenv::dotenv;
use ethers_core::{
    types::{Address, BlockId, H160, U256},
    utils::get_contract_address,
};
use ethers_middleware::MiddlewareBuilder;
use ethers_providers::{Middleware, Provider, Ws};
use ethers_signers::{LocalWallet, Signer};
use eyre::Result;
use futures_util::StreamExt;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use ethers_contract::abigen;

use crate::{
    bundle::{bundle_txs, send_bundle, BundleData},
    clone::get_clone_address,
    CHALLENGE_ADDRESS,
};

abigen!(
    IPuzzle,
    r#"[function generate(address _seed) external returns (uint256)]"#
);

pub async fn submit_bundle(lead_deploy_code: &str, gold_deploy_code: &str) -> Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let solver =
        LocalWallet::from_str(&env::var("PRIV_KEY").expect("PRIV_KEY must be set")).unwrap();
    let ws_eth_rpc = env::var("WS_ETH_RPC").expect("WS_ETH_RPC should be set");
    let provider = Provider::<Ws>::connect(ws_eth_rpc).await?;
    let client = Arc::new(provider.clone());
    let provider = provider
        .clone()
        .with_signer(solver.clone())
        .nonce_manager(solver.address());

    let solver_nonce = provider
        .initialize_nonce(Some(BlockId::Number(
            ethers_core::types::BlockNumber::Latest,
        )))
        .await?;
    println!("next nonce {}", solver_nonce);
    let solver_address: H160 = solver.address();

    // 1. 0x001ead deploy address
    let lead_address = get_contract_address(solver_address, solver_nonce);
    println!("lead address: {:?}", lead_address);

    // 2. 0x00901d deploy address
    let gold_address = get_contract_address(solver_address, solver_nonce + 1);
    println!("gold address: {:?}", gold_address);

    // 3. get sigil parts
    let puzzle = IPuzzle::new(CHALLENGE_ADDRESS.parse::<Address>()?, client);
    let sigils = puzzle.generate(solver.address()).call().await?;

    let mut bytes = [0u8; 32];
    sigils.to_big_endian(&mut bytes);

    let (coagula_sigil, solve_sigil) = bytes.split_at(16);

    let mut stream = provider.subscribe_blocks().await?;
    while let Some(block) = stream.next().await {
        let target_block = block.number.unwrap() + 1;
        println!("target block number: {:?}", target_block);

        // 3. Get solve clone address
        let solve_clone_address =
            get_clone_address(target_block, U256::from_big_endian(solve_sigil));
        println!("solve clone address: {:?}", solve_clone_address);

        // 4. Get coagula clone address
        let coagula_clone_address =
            get_clone_address(target_block, U256::from_big_endian(coagula_sigil));
        println!("coagula clone address: {:?}", coagula_clone_address);

        let bundle = bundle_txs(BundleData {
            block_number: target_block,
            lead_deploy_address: lead_address,
            lead_deploy_code,
            gold_deploy_address: gold_address,
            gold_deploy_code,
            solve_clone_address,
            coagula_clone_address,
        })
        .await?;
        if block.number.unwrap() == target_block - 1 {
            send_bundle(bundle).await?;
            println!("gonna send bundle");
            break;
        }
        println!(
            "Ts: {:?}, block number: {} -> {:?}",
            block.timestamp,
            block.number.unwrap(),
            block.hash.unwrap()
        );
    }

    Ok(())
}
