// SPDX-License-Identifier: Unlicense

use rand::prelude::*;
use std::fmt::Display;
use std::str::FromStr;

/// The account number uses random number generation
/// for the payload. This payload is then calculated
/// to produce the check digit.
#[derive(Debug)]
pub struct AccountNumber {
    /// Randomly generated number. Each digit is within 0..9
    pub(crate) payload: Vec<u8>,
    /// Check digit using the Luhn formula
    pub(crate) check_digit: u8,
}

impl Default for AccountNumber {
    fn default() -> Self {
        Self::new(10)
    }
}

impl FromStr for AccountNumber {
    type Err = std::io::ErrorKind;

    fn from_str(s: &str) -> Result<Self, std::io::ErrorKind> {
        if verify(s) {
            let mut payload: Vec<u8> = s
                .chars()
                .map(|d| {
                    d.to_digit(10)
                        .expect("Not able to create u32 from character") as u8
                })
                .collect();
            let check_digit = payload.pop().expect("Cannot pop. Got an empty vector");
            Ok(Self {
                payload,
                check_digit,
            })
        } else {
            eprintln!("{s} is not a valid account number");
            Err(std::io::ErrorKind::InvalidData)
        }
    }
}

impl Display for AccountNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut account_number: String = self.payload.iter().map(|d| d.to_string()).collect();
        account_number.push_str(&self.check_digit.to_string());
        write!(f, "{}", account_number)
    }
}

impl AccountNumber {
    pub fn new(length: usize) -> Self {
        let mut payload: Vec<u8> = Vec::new();
        for _ in 1..=length - 1 {
            let mut rng = thread_rng();
            let zero_to_nine: u8 = rng.gen_range(0..=9);
            payload.push(zero_to_nine);
        }

        let check_digit = get_check_digit(&payload);

        Self {
            payload,
            check_digit,
        }
    }

    pub fn check_digit(&self) -> u8 {
        get_check_digit(&self.payload)
    }

    pub fn human_readable(&self) -> String {
        let mut payload: String = self.payload.iter().map(|d| d.to_string()).collect();
        payload.push_str(&self.check_digit.to_string());
        payload
    }
}

pub fn verify(account_number: &str) -> bool {
    let account_number = account_number.to_string();
    let digits: Vec<char> = account_number.trim().chars().collect();

    let mut payload: Vec<u8> = digits
        .iter()
        .map(|digit| digit.to_digit(10).expect("Not a number") as u8)
        .collect();

    let check_digit = payload.pop().expect("This shouldn't be an empty iterator");

    get_check_digit(&payload) == check_digit
}

fn get_check_digit(payload: &[u8]) -> u8 {
    let mut new_payload = payload.iter().copied();
    let mut luhn_sum = 0;
    let mut index = 0;

    while let Some(item) = new_payload.next_back() {
        let divisible_by_2 = { (index as f32 % 2_f32) == 0_f32 };
        if divisible_by_2 {
            let mul = item * 2;

            let digits: Vec<u8> = mul
                .to_string()
                .chars()
                .map(|d| d.to_digit(10).expect("Not a number character") as u8)
                .collect();

            let sum = {
                let mut x = 0;
                for digit in digits {
                    let y: u8 = digit;
                    x += y;
                }
                x
            };

            luhn_sum += sum;
        } else {
            luhn_sum += item;
        };
        index += 1;
    }

    10 - (luhn_sum % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Valid account numbers in different lengths
    fn valid_account_numbers_in_different_lengths() {
        let accounts = [
            AccountNumber::from_str("35001576202"),
            AccountNumber::from_str("8536276945"),
            AccountNumber::from_str("20024"),
            AccountNumber::from_str("26"),
        ];

        for account in accounts {
            assert!(account.is_ok());
        }
    }

    #[test]
    fn valid_account_numbers_fixed_length() {
        let valid_account_numbers = [
            "2334841596",
            "5072686164",
            "8330789085",
            "2303133926",
            "7730632457",
            "1310866767",
            "9083062142",
            "8936042657",
            "3188178648",
            "1513312791",
            "0204434294",
        ];

        for v in valid_account_numbers {
            let account = AccountNumber::from_str(v);
            assert!(account.is_ok());
        }
    }

    #[test]
    fn invalid_account_numbers_fixed_length() {
        let invalid_account_numbers = [
            "2334841592",
            "5072686163",
            "8330789084",
            "2303133925",
            "7730632456",
            "1310866766",
            "9083062141",
            "8936042656",
            "3188178647",
            "1513312790",
            "0204434293",
        ];

        for v in invalid_account_numbers {
            let account = AccountNumber::from_str(v);
            assert!(account.is_err());
        }
    }
}