use crate::{
    constants::{FEE_PER_DIV, NATIVE_MINT_STR},
    error::PumpFunError,
};
use anchor_lang::{prelude::*, solana_program::program::invoke};
use anchor_spl::token::{self, CloseAccount, SyncNative, TokenAccount};

// This function checks accounts's Token/WSOL balance
// Params
//   ata - Creator's Token/WSOL ATA
//   required_amount - Amount of token/WSOL that the creator must have
// Return
//   true if sufficient, else false
pub fn check_balance(ata: &TokenAccount, require_amount: u64) -> bool {
    if (ata.mint.to_string() == NATIVE_MINT_STR) {
        return true;
    }
    if ata.amount < require_amount {
        return false;
    }
    true
}

// This function calculates trading fee
// Params
//   fee - feeBps
//   amount - Trading amount of SOL
// Return
//   trading fee in SOL
pub fn calculate_trading_fee(fee: u64, amount: u64) -> u64 {
    (amount as u128)
        .checked_mul(fee.into())
        .unwrap()
        .checked_div(FEE_PER_DIV)
        .unwrap()
        .checked_div(100)
        .unwrap() as u64
}

// This function calculates total amount of SOL
// Params
//   fee - feeBps
//   input_amount - Amount of SOL
// Return
//   total amount in SOL
pub fn calculate_total_amount(fee: u64, input_amount: u64) -> u64 {
    (input_amount as u128)
        .checked_mul(FEE_PER_DIV.checked_mul(100).unwrap())
        .unwrap()
        .checked_div(FEE_PER_DIV.checked_mul(100).unwrap() - fee as u128)
        .unwrap() as u64
}

// This function converts required amount of owner's SOL to WSOL if insufficient
// Params
//   owner - Owner
//   ata - Owner's Token ATA
//   required_amount - Required amount of WSOL
//   system_program - System program
//   token_program - Token program
// Return
//   Ok on success, ErrorCode on failure
pub fn sync_native_amount<'a>(
    owner: AccountInfo<'a>,
    ata: &Account<'a, TokenAccount>,
    require_amount: u64,
    system_program: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
) -> Result<()> {
    let ata_balance = ata.amount;
    let mut sync_amount = 0;
    if require_amount > ata_balance {
        sync_amount = require_amount - ata_balance
    }
    if sync_amount != 0 {
        if (owner.lamports.borrow().clone() < sync_amount) {
            return Err(PumpFunError::InsufficientFund.into());
        }
        let sol_transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            owner.key,
            &ata.key(),
            sync_amount,
        );
        invoke(
            &sol_transfer_ix,
            &[
                owner.to_account_info(),
                ata.to_account_info(),
                system_program,
            ],
        )?;
        let sync_accounts = SyncNative {
            account: ata.to_account_info(),
        };
        token::sync_native(CpiContext::new(token_program, sync_accounts))?;
    }
    Ok(())
}

// This function closes token account (WSOL is converted to SOL)
// Params
//   owner - Owner
//   ata - Owner's Token ATA
//   token_program - Token program
// Return
//   Ok
pub fn close_token_account<'a>(
    owner: AccountInfo<'a>,
    ata: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
) -> Result<()> {
    let cpi_accounts = CloseAccount {
        account: ata,
        authority: owner.clone(),
        destination: owner,
    };
    token::close_account(CpiContext::new(token_program, cpi_accounts))?;
    Ok(())
}
