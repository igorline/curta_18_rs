use clap::Parser;
use curta_18_solver::{
    brute_codehash::{bruteforce_code_hash, Materia},
    submit_bundle::submit_bundle,
};
use dotenv::dotenv;
use eyre::Result;
use opts::{Opts, Subcommands};

mod opts;

const GOLD: [u8; 3] = [0, 0x90, 0x1d];
const LEAD: [u8; 3] = [0, 0x1e, 0xad];

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let opts = Opts::parse();
    match opts.sub {
        Subcommands::BruteforceCodeHash(bruteforce_codehash_args) => {
            let code_hash = match bruteforce_codehash_args.materia {
                Materia::Gold => &GOLD,
                Materia::Lead => &LEAD,
            };
            bruteforce_code_hash(code_hash, &bruteforce_codehash_args.init_code_hash);
            Ok(())
        }
        Subcommands::SubmitBundle(submit_bundle_args) => {
            submit_bundle(
                &submit_bundle_args.lead_deploy_code,
                &submit_bundle_args.gold_deploy_code,
            )
            .await?;
            Ok(())
        }
    }
}
