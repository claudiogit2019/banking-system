// SPDX-License-Identifier: Unlicense

use banking_system::cli;
use banking_system::database;
use banking_system::luhn::AccountNumber;
use banking_system::menu;
use clap::Parser;
use rusqlite::Result;

fn main() -> Result<()> {
    let cli = cli::Opts::parse();

    match cli.account {
        cli::AccountOpts::Login { account, pin } => {
            let db = database::initialise_bankdb()?;
            let query_string = format!(
                "SELECT pin FROM account where account_number='{}';",
                account
            );

            let pin_from_db: Result<String> = db.query_row(&query_string, [], |row| row.get(0));
            match pin_from_db {
                Ok(p) => {
                    if p == pin {
                        menu::prompt(&account).expect("Something went wrong");
                    } else {
                        eprintln!("Wrong pin... try again");
                    };
                }
                Err(_) => {
                    eprintln!("No such account");
                }
            };
        }
        cli::AccountOpts::Delete { account, pin } => {
            database::delete_account(&account, &pin)?;
        }
        cli::AccountOpts::Create => {
            let mut new_account = AccountNumber::default();

            let db = database::initialise_bankdb()?;
            loop {
                let query_string = format!(
                    "SELECT 1 FROM account where account_number='{}';",
                    new_account
                );

                match db.query_row(&query_string, [], |row| row.get::<usize, usize>(0)) {
                    Ok(_) => {
                        new_account = AccountNumber::default();
                    }
                    Err(_) => break,
                }
            }

            let _ = database::create_account(&new_account, 0);

            let query_string = format!(
                "SELECT pin FROM account where account_number='{}';",
                &new_account
            );
            let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;
            println!(
                "YOUR NEW ACCOUNT: `{}`\nYOUR PIN: `{}`\n",
                &new_account, &pin_from_db
            );
        }
    };
    Ok(())
}