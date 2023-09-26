use std::{env, str::FromStr};

use ethers_core::types::Bytes;
use ethers_core::{
    abi::{self, Token},
    types::{transaction::eip2718::TypedTransaction, Eip1559TransactionRequest, H160, U64},
};
use ethers_middleware::MiddlewareBuilder;
use ethers_providers::{Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use eyre::Result;
use jsonrpsee::{
    core::async_trait,
    http_client::{transport::Error as HttpError, HttpClientBuilder},
};
use mev_share_rpc_api::{
    BundleItem, FlashbotsSignerLayer, Inclusion, MevApiClient, SendBundleRequest,
};
use tower::ServiceBuilder;

use crate::CHALLENGE_ADDRESS;

pub struct BundleData<'a> {
    pub block_number: U64,
    pub lead_deploy_address: H160,
    pub lead_deploy_code: &'a str,
    pub gold_deploy_address: H160,
    pub gold_deploy_code: &'a str,
    pub solve_clone_address: H160,
    pub coagula_clone_address: H160,
}

pub async fn bundle_txs<'a>(bundle_data: BundleData<'a>) -> Result<SendBundleRequest> {
    let tx_signer =
        LocalWallet::from_str(&env::var("PRIV_KEY").expect("PRIV_KEY must be set")).unwrap();
    let eth_rpc = env::var("ETH_RPC").expect("ETH_RPC is not set");

    let provider = Provider::try_from(eth_rpc)?
        .with_signer(tx_signer.clone())
        .nonce_manager(tx_signer.address());

    // Build bundle
    let mut bundle_body = Vec::new();

    let BundleData {
        block_number,
        lead_deploy_address,
        gold_deploy_address,
        solve_clone_address,
        lead_deploy_code,
        gold_deploy_code,
        coagula_clone_address,
    } = bundle_data;

    // 1. deploy 0x001ead
    let lead_deploy_code = hex::decode(lead_deploy_code).unwrap();
    let lead_deploy_tx = Eip1559TransactionRequest::new()
        .data(lead_deploy_code)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: lead_deploy_tx,
        can_revert: false,
    });

    // 2. deploy 0x00901d
    let gold_deploy_code = hex::decode(gold_deploy_code).unwrap();
    let gold_deploy_tx = Eip1559TransactionRequest::new()
        .data(gold_deploy_code)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: gold_deploy_tx,
        can_revert: false,
    });

    // 3. transmute - 1
    let mut sig = hex::decode("0ddeaa5c").unwrap();
    let encoded = abi::encode(&[Token::Address(lead_deploy_address)]);
    sig.extend(encoded);
    let encoded_data_as_bytes = Bytes::from(sig);

    let transmute_tx = Eip1559TransactionRequest::new()
        .data(encoded_data_as_bytes)
        .to(CHALLENGE_ADDRESS)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: transmute_tx,
        can_revert: false,
    });

    // 4. solve
    let sig = hex::decode("890d6908").unwrap();

    let solve_tx = Eip1559TransactionRequest::new()
        .data(sig)
        .to(CHALLENGE_ADDRESS)
        .gas(200000)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: solve_tx,
        can_revert: false,
    });

    // 5. transmute - 2
    let mut sig = hex::decode("0ddeaa5c").unwrap();
    let encoded = abi::encode(&[Token::Address(gold_deploy_address)]);
    sig.extend(encoded);
    let encoded_data_as_bytes = Bytes::from(sig);

    let transmute_tx2 = Eip1559TransactionRequest::new()
        .data(encoded_data_as_bytes)
        .to(CHALLENGE_ADDRESS)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: transmute_tx2,
        can_revert: false,
    });

    // 7. 0x841271ed - 1
    let sig = hex::decode("841271ed").unwrap();

    let bartled_tx = Eip1559TransactionRequest::new()
        .data(sig)
        .to(solve_clone_address)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: bartled_tx,
        can_revert: false,
    });

    // 6. coagula
    let sig = hex::decode("39a6909f").unwrap();

    let coagula_tx = Eip1559TransactionRequest::new()
        .data(sig)
        .to(CHALLENGE_ADDRESS)
        .gas(200000)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: coagula_tx,
        can_revert: false,
    });

    // 8. 0x841271ed - 2
    let sig = hex::decode("841271ed").unwrap();

    let bartled_tx2 = Eip1559TransactionRequest::new()
        .data(sig)
        .to(coagula_clone_address)
        .fill(&provider)
        .await?
        .sign(&tx_signer)
        .await?;

    bundle_body.push(BundleItem::Tx {
        tx: bartled_tx2,
        can_revert: false,
    });

    let bundle = SendBundleRequest {
        bundle_body,
        inclusion: Inclusion {
            block: block_number,
            max_block: Some(block_number),
        },
        ..Default::default()
    };
    Ok(bundle)
}

pub async fn send_bundle(bundle: SendBundleRequest) -> Result<()> {
    // Set up the rpc client
    let fb_signer =
        LocalWallet::from_str(&env::var("REP_PRIV_KEY").expect("REP_PRIV_KEY must be set"))
            .unwrap()
            .with_chain_id(1_u64);

    let signing_middleware = FlashbotsSignerLayer::new(fb_signer);
    let service_builder = ServiceBuilder::new()
        // map signer errors to http errors
        .map_err(HttpError::Http)
        .layer(signing_middleware);

    let url = "https://relay.flashbots.net:443";
    let client = HttpClientBuilder::default()
        .set_middleware(service_builder)
        .build(url)
        .expect("Failed to create http client");

    let resp = client.send_bundle(bundle.clone()).await;
    println!("Got a bundle response: {:?}", resp);
    //
    // Simulate bundle
    let sim_res = client.sim_bundle(bundle, Default::default()).await;
    println!("Got a simulation response: {:?}", sim_res);

    Ok(())
}

#[async_trait]
trait TxFill {
    async fn fill<P: Middleware>(mut self, eth_client: &P) -> Result<TypedTransaction>;
}

#[async_trait]
impl TxFill for Eip1559TransactionRequest {
    async fn fill<P: Middleware>(mut self, eth_client: &P) -> Result<TypedTransaction> {
        let mut tx: TypedTransaction = self.into();
        eth_client
            .fill_transaction(&mut tx, None)
            .await
            .expect("Failed to fill backrun transaction");
        Ok(tx)
    }
}

#[async_trait]
trait TxSign {
    async fn sign<S: Signer>(&self, signer: &S) -> Result<Bytes>
    where
        S::Error: 'static;
}

#[async_trait]
impl TxSign for TypedTransaction {
    async fn sign<S: Signer>(&self, signer: &S) -> Result<Bytes>
    where
        S::Error: 'static,
    {
        let signature = signer.sign_transaction(self).await?;
        Ok(self.rlp_signed(&signature))
    }
}
