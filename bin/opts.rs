use clap::{Parser, Subcommand};
use curta_18_solver::brute_codehash::Materia;

#[derive(Debug, Parser)]
#[clap(name = "curta-18-solver")]
pub struct Opts {
    #[clap(subcommand)]
    pub sub: Subcommands,
}

#[derive(Debug, Parser, Clone)]
pub struct BruteforceCodeHashArgs {
    pub materia: Materia,
    pub init_code_hash: String,
}

#[derive(Debug, Parser, Clone)]
pub struct SubmitBundleArgs {
    pub lead_deploy_code: String,
    pub gold_deploy_code: String,
}

#[derive(Debug, Subcommand)]
pub enum Subcommands {
    BruteforceCodeHash(BruteforceCodeHashArgs),
    SubmitBundle(SubmitBundleArgs),
}
