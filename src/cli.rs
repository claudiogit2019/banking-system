// SPDX-License-Identifier: Unlicense

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "bank", version)]
pub struct Opts {
    /// Account subcommand
    #[command(subcommand)]
    pub account: AccountOpts,
}

#[derive(Subcommand, Debug)]
#[command(name = "account", about = "A simple bank system in Rust")]
pub enum AccountOpts {
    /// Login into account with account number. This will get you to the Main Menu.
    #[command(name = "login")]
    Login {
        #[arg(help = "Account number of user")]
        account: String,
        #[arg(help = "PIN of the account")]
        pin: String,
    },
    /// Delete into account with account number. Your account will be gone forever.
    #[command(name = "delete")]
    Delete {
        #[arg(help = "Account number of user")]
        account: String,
        #[arg(help = "PIN of the account.")]
        pin: String,
    },
    /// Create new account. This will have a randomly generated PIN.
    #[command(name = "create")]
    Create,
}