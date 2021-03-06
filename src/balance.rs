extern crate serde_yaml;

use colored::*;
use super::models::LedgerFile;
use num_format::{Locale, ToFormattedString};

struct BalanceAccount {
    account: String,
    account_type: String,
    amount: i32,
}

struct TransactionAccount {
    account: String,
    account_type: String,
    offset_account: String,
    amount: i32,
}

/// returns balances of all general ledger accounts
pub fn balance(filename: &String) -> Result<(), std::io::Error> {
    let file = std::fs::File::open(filename)?;
    let deserialized_file: LedgerFile = serde_yaml::from_reader(file).unwrap();

    let mut accounts_vec: Vec<BalanceAccount> = Vec::new();
    let mut transactions_vec: Vec<TransactionAccount> = Vec::new();

    // push opening balances into Vec
    for account in deserialized_file.accounts {
        accounts_vec.push(BalanceAccount {
            account: account.acct_name,
            account_type: account.acct_type,
            amount: account.debit_credit,
        });
    }

    // push transactions into Vec
    for transaction in deserialized_file.transactions {
        let offset_account = &transaction.acct_offset_name;

        match transaction.split {
            None => {
                let amount = match transaction.acct_type.as_ref() {
                    "income" => -transaction.debit_credit,
                    _ => transaction.debit_credit,
                };

                transactions_vec.push(TransactionAccount {
                    account: transaction.acct_name,
                    account_type: transaction.acct_type,
                    offset_account: offset_account.to_string(),
                    amount: amount,
                });
            },
            Some(split) => {
                let mut credit: i32 = 0;
                
                for i in split {
                    let acct_type = transaction.acct_type.as_ref();
                    let amount = match acct_type {
                        "income" => -i.amount,
                        _ => i.amount,
                    };
                    credit += amount;
                    transactions_vec.push(TransactionAccount {
                        account: i.account,
                        account_type: i.account_type.unwrap_or(acct_type.to_string()),
                        offset_account: offset_account.to_string(),
                        amount: i.amount,
                    })
                }

                transactions_vec.push(TransactionAccount {
                    account: transaction.acct_name,
                    account_type: transaction.acct_type,
                    offset_account: offset_account.to_string(),
                    amount: transaction.debit_credit - credit,
                });
            }
        }
    }

    // loop over Vecs and increment(+)/decrement(-) totals
    // for each transaction

    for transaction in &transactions_vec {
        for account in &mut accounts_vec {
            if account.account == transaction.account 
                && account.account_type == transaction.account_type {
                account.amount += &transaction.amount;
            }
            if account.account == transaction.offset_account {
                account.amount -= &transaction.amount;
            }
        }
    }

    // create output

    let mut check_figure: i32 = 0;

    println!("\n {0: <29} {1: <20}", "Account".bold(), "Balance".bold());

    println!("{0:-<39}", "".bright_blue());

    let mut current_account_type = String::new();

    for account in accounts_vec {
        check_figure += account.amount;

        if !current_account_type.eq(&account.account_type) {
            current_account_type = account.account_type;
            println!("{}", current_account_type);
        }

        println!(
            "  {0: <28} {1: <20}",
            account.account,
            if account.amount < 0 {
                if current_account_type.eq("asset") {
                    (account.amount).to_formatted_string(&Locale::en).red().bold()
                } else {
                    (account.amount).to_formatted_string(&Locale::en).bold()
                }
            } else if account.amount == 0 {
                (account.amount).to_formatted_string(&Locale::en).yellow().bold()
            } else {
                (account.amount).to_formatted_string(&Locale::en).bold()
            }
        );
    }

    println!("\n{:-<39}", "".bright_blue());
    print!("{: <30}", "check");
    print!(" {:<20}\n", match check_figure {
        0 => check_figure
            .to_formatted_string(&Locale::en)
            .bold(),
        _ => check_figure
            .to_formatted_string(&Locale::en)
            .red().bold(),
    });

    println!("\n");

    Ok(())
}
