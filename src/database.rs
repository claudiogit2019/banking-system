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

#[cfg(not(test))]
fn database_path() -> PathBuf {
    PathBuf::from("bank.s3db")
}

#[cfg(test)]
fn database_path() -> PathBuf {
    PathBuf::from("mock_bank.s3db")
}

pub fn initialise_bankdb() -> Result<Connection> {
    let db = Connection::open(database_path())?;
    let command = "CREATE TABLE IF NOT EXISTS account(
id INTEGER PRIMARY KEY,
account_number TEXT,
pin TEXT DEFAULT '000000',
balance INTEGER DEFAULT 0
)";
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
    origin_account: &str,
    target_account: &str,
) -> Result<(Account, Account)> {
    if *origin_account == *target_account {
        return Err(rusqlite::Error::QueryReturnedNoRows); // Makes sense. We haven't returned any.
    }

    // Create new binding
    let origin_account = fetch_account(origin_account)?;
    let target_account = fetch_account(target_account)?;

    let correct_pin = origin_account.pin == pin;

    if correct_pin {
        let amount = amount
            .parse::<u64>().map_err(|_| {
                rusqlite::Error::QueryReturnedNoRows
            })?;

        if amount > origin_account.balance {
        } else {
            let db = initialise_bankdb()?;
            // Add money to account 2
            db.execute(
                "UPDATE account SET balance = balance + ?1 WHERE account_number=?2",
                rusqlite::params![amount as i64, &target_account.account_number],
            )?;
            
            db.execute(
                "UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
                rusqlite::params![amount as i64, &origin_account.account_number],
            )?;
            
        };
    } else {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let origin_account = fetch_account(&origin_account.account_number)?;
    let target_account = fetch_account(&target_account.account_number)?;

    Ok((origin_account, target_account))
}

pub fn withdraw(amount: &str, pin: &str, account_number: &str) -> Result<()> {
    let db = initialise_bankdb()?;
    let query_string = format!(
        "SELECT pin, balance FROM account WHERE account_number='{}';",
        account_number
    );

    let (pin_from_db, balance_from_db): (String, u64) = db.query_row(&query_string, [], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    if pin_from_db == pin {
        let amount = amount.parse::<u64>().map_err(|_| rusqlite::Error::InvalidParameterName("Invalid amount".into()))?;
        if balance_from_db >= amount {
            db.execute(
                "UPDATE account SET balance = balance - ?1 WHERE account_number=?2",
                rusqlite::params![amount, account_number],
            )?;

            println!(
                "The account number `{}` now has a balance of `{}`.\n",
                account_number,
                balance_from_db - amount
            );
        } else {
            eprintln!("Insufficient funds.");
        }
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

fn fetch_account(account: &str) -> Result<Account> {
    let db = initialise_bankdb()?;
    let mut stmt = db.prepare("SELECT id, account_number, balance, pin FROM account")?;
    let accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            account_number: row.get(1)?,
            balance: row.get(2)?,
            pin: row.get(3)?,
        })
    })?;

    let accounts = accounts.flatten().find(|acc| acc.account_number == account);
    if let Some(fetched_account) = accounts {
        Ok(fetched_account)
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn created_account_is_correct_fetched_from_db() -> Result<()> {
        let account_number = AccountNumber::new(10);
        create_account(&account_number, 100)?;
        let account = fetch_account(&account_number.to_string())?;

        assert_eq!(account.account_number, account_number.to_string());
        assert_eq!(account.balance, 100);

        Ok(())
    }

    #[test]
    fn transferred_balance_is_correct() -> Result<()> {
        let origin_account_number = AccountNumber::new(10);
        let target_account_number = AccountNumber::new(10);

        create_account(&origin_account_number, 10000)?;
        create_account(&target_account_number, 0)?;

        let origin_account = fetch_account(&origin_account_number.to_string())?;
        let target_account = fetch_account(&target_account_number.to_string())?;

        let pin = origin_account.pin.clone();
        transfer("10000", &pin, &origin_account.account_number, &target_account.account_number)?;

        let origin_account = fetch_account(&origin_account.account_number)?;
        let target_account = fetch_account(&target_account.account_number)?;

        assert_eq!(origin_account.balance, 0);
        assert_eq!(target_account.balance, 10000);

        Ok(())
    }
}
