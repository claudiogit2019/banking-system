// SPDX-License-Identifier: Unlicense

use crate::database;
use std::io::BufRead;

pub fn prompt(account_number: &str) -> std::io::Result<()> {
    let prompt_text = "0) Show Current Balance
1) Deposit Money
2) Transfer Money
3) Withdraw Money
4) Delete Account
5) Exit";

    loop {
        println!("{}", &prompt_text);
        let stdin = std::io::stdin();
        let mut query = String::new();
        let mut handle = stdin.lock();

        handle.read_line(&mut query)?;

        let query = query.trim();

        if query == "5" {
            eprintln!("Exiting bank machine...");
            break;
        } else if query == "4" {
            println!("You are going to delete your account...");
            println!("Please input the pin:");
            let mut pin = String::new();
            handle.read_line(&mut pin)?;
            let pin = pin.trim();

            database::delete_account(account_number, pin)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            eprintln!("Account is not accessible. Exiting...");
            break;
        } else if query == "3" {
            println!("Please input the amount:");
            let mut amount = String::new();
            handle.read_line(&mut amount)?;
            let amount = amount.trim();
            println!("The amount you wanted to withdraw: {}\n", &amount);

            println!("Please input the pin:");
            let mut pin = String::new();
            handle.read_line(&mut pin)?;
            let pin = pin.trim();

            database::withdraw(amount, pin, account_number)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        } else if query == "2" {
            println!("Please input the amount:");
            let mut amount = String::new();
            handle.read_line(&mut amount)?;
            let amount = amount.trim();
            println!("The amount you wanted to transfer: {}\n", &amount);

            println!("Please input the account number of the recipient:");
            let mut account_number2 = String::new();
            handle.read_line(&mut account_number2)?;
            let account_number2 = account_number2.trim();
            println!(
                "The account number you want to send is {} and the amount to transfer is {}\n",
                &account_number2, &amount
            );

            println!("Please input your pin:");
            let mut pin = String::new();
            handle.read_line(&mut pin)?;
            let pin = pin.trim();
            database::transfer(amount, pin, account_number, account_number2)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        } else if query == "1" {
            println!("Please input the amount:");
            let mut amount = String::new();
            handle.read_line(&mut amount)?;
            let amount = amount.trim();
            println!("The amount you wanted to deposit: {}\n", &amount);

            println!("Please input the pin:");
            let mut pin = String::new();
            handle.read_line(&mut pin)?;
            let pin = pin.trim();

            database::deposit(amount, pin, account_number)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        } else if query == "0" {
            database::show_balance(account_number)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        } else {
            eprintln!("Invalid choice. Please try again...");
        }
    }
    Ok(())
}