use clap::Parser;
use curta_18_solver::{brute_codehash::bruteforce_code_hash, submit_bundle::submit_bundle};
use dotenv::dotenv;
use eyre::Result;
use opts::{Opts, Subcommands};

mod opts;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let opts = Opts::parse();
    match opts.sub {
        Subcommands::BruteforceCodeHash(bruteforce_codehash_args) => {
            bruteforce_code_hash(
                bruteforce_codehash_args.materia,
                &bruteforce_codehash_args.init_code_hash,
            );
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
