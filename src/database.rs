// SPDX-License-Identifier: Unlicense

use std::path::PathBuf;

use crate::luhn::AccountNumber;

use rand::prelude::*;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Account {
    pub id: u64,
    pub account_number: String,
    pub balance: u64,
    pub pin: String,
}

fn database_path() -> PathBuf {
    PathBuf::from("bank.s3db")
}

pub fn initialise_bankdb() -> Result<Connection> {
    let db = Connection::open(database_path())?;

    let command = "CREATE TABLE IF NOT EXISTS account(
id INTEGER PRIMARY KEY,
account_number TEXT,
pin TEXT DEFAULT '000000',
balance INTEGER DEFAULT 0
)
";

db.execute(command, rusqlite::params![])?;

    Ok(db)
}

pub fn create_account(data: &AccountNumber, balance: u64) -> Result<()> {
    let db = initialise_bankdb()?;
    let account_number = data.to_string();
    let mut stmt = db.prepare("SELECT id, account_number, balance, pin FROM account")?;
    let accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            account_number: row.get(1)?,
            balance: row.get(2)?,
            pin: row.get(3)?,
        })
    })?;

    let get_latest_max_id = {
        let mut x = 0;
        for account in accounts.flatten() {
            if account.id > x {
                x = account.id
            }
        }
        x
    };

    let newest_max_id = get_latest_max_id + 1;
    let mut rng = thread_rng();
    let mut pin: Vec<String> = Vec::new();

    // Six digit pin
    for _ in 1..=6 {
        let y = rng.gen_range(0..=9).to_string();
        pin.push(y);
    }

    let pin: String = String::from_iter(pin);

    let new_account = Account {
        id: newest_max_id,
        account_number,
        balance,
        pin,
    };

    db.execute(
        "INSERT INTO account (id, account_number, pin, balance) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            new_account.id,
            new_account.account_number,
            new_account.pin,
            new_account.balance,
        ],
    )?;
    
    Ok(())
}

pub fn deposit(amount: &str, pin: &str, account_number: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account where account_number='{}';",
        account_number
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;

    let correct_pin = { pin_from_db == pin };

    if correct_pin {
        db.execute(
            "UPDATE account SET balance = balance + ?1 WHERE account_number=?2",
            rusqlite::params![amount, account_number],
        )?;
        

        let query_string = format!(
            "SELECT balance FROM account where account_number='{}';",
            account_number
        );

        let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

        println!(
            "The account number `{}` now has a balance of `{}`.\n",
            &account_number, &amount_from_db
        );
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

pub fn transfer(
    amount: &str,
    pin: &str,
    account_number1: &str,
    account_number2: &str,
) -> Result<()> {
    if *account_number1 == *account_number2 {
        eprintln!("Cannot perform a transfer to the same account!");
        return Ok(());
    };

    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account where account_number='{}';",
        account_number1
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;

    let correct_pin = { pin_from_db == pin };
    if correct_pin {
        let query_string = format!(
            "SELECT balance FROM account where account_number='{}';",
            account_number1
        );

        let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

        println!(
            "The account number `{}` has a balance of `{}`.\n",
            &account_number1, &amount_from_db
        );

        let amount = amount
            .parse::<usize>()
            .expect("Not able to parse string to usize");

        if amount > amount_from_db {
            eprintln!(
                "You are trying to transfer that exceeds your current balance... aborting...\n"
            );
        } else {
            // Add money to account 2
            db.execute(
                "UPDATE account SET balance = balance + ?1 WHERE account_number=?2",
                rusqlite::params![amount, account_number2],
            )?;
            

            // Subtract money from account 1
            db.execute(
                "UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
                rusqlite::params![amount, account_number1],
            )?;
            

            let query_string = format!(
                "SELECT balance FROM account where account_number='{}';",
                account_number1
            );

            let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

            println!(
                "The account number `{}` now has a balance of `{}`.\n",
                &account_number1, &amount_from_db
            );
        };
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

pub fn withdraw(amount: &str, pin: &str, account_number: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account where account_number='{}';",
        account_number
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;

    let correct_pin = { pin_from_db == pin };

    if correct_pin {
        let query_string = format!(
            "SELECT balance FROM account where account_number='{}';",
            account_number
        );

        let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

        println!(
            "The account number `{}` has a balance of `{}`.\n",
            &account_number, &amount_from_db
        );

        let amount = amount
            .parse::<usize>()
            .expect("Not able to parse string to usize");

        if amount > amount_from_db {
            eprintln!(
                "You are trying to withdraw that exceeds your current deposit... aborting...\n"
            );
        } else {
            db.execute(
                "UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
                rusqlite::params![amount, account_number],
            )?;
            

            let query_string = format!(
                "SELECT balance FROM account where account_number='{}';",
                account_number
            );

            let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

            println!(
                "The account number `{}` now has a balance of `{}`.\n",
                &account_number, &amount_from_db
            );
        };
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

pub fn delete_account(account_number: &str, pin: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin FROM account where account_number='{}';",
        &account_number
    );

    let pin_from_db: String = db.query_row(&query_string, [], |row| row.get(0))?;
    let correct_pin = { pin_from_db == pin };

    if correct_pin {
        db.execute(
            "DELETE FROM account WHERE account_number=?1",
            rusqlite::params![account_number],
        )?;
        
        println!("DELETED ACCOUNT: {}", &account_number);
    } else {
        eprintln!("Wrong pin. Try again...");
    }
    Ok(())
}

pub fn show_balance(account_number: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT balance FROM account where account_number='{}';",
        account_number
    );

    let amount_from_db: usize = db.query_row(&query_string, [], |row| row.get(0))?;

    println!(
        "The account number `{}` now has a balance of `{}`.\n",
        &account_number, &amount_from_db
    );
    Ok(())
}