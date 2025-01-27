use anchor_lang::prelude::*;

// BondingCurve struct
#[account]
pub struct PoolState {
    pub owner: Pubkey,              // BondingCurve creator
    pub base_mint: Pubkey,          // Base token mint address
    pub virt_base_reserves: u64,    // Amount of virtual base tokens in the bonding curve
    pub real_base_reserves: u64,    // Amount of real base tokens remaining in the bonding curve
                                    //   Starts with total_supply and is changed on buying/selling tokens
                                    //   When the bonding curve is complete, it should be ~20% of total_supply
    pub quote_mint: Pubkey,         // Quote token mint address
    pub virt_quote_reserves: u64,   // Amount of virtual quote token in the bonding curve
    pub real_quote_reserves: u64,   // Amount of real quote token deposited in the bonding curve
                                    //   Starts with 0 quote token and is changed on buying/selling tokens
                                    //   When the bonding curve is complete, it should become real_quote_threshold
    pub real_quote_threshold: u64,  // Real quote token threshold
    pub complete: bool              // Flag indicating whether the bonding curve is complete or not
}

impl PoolState {
    pub const MAX_SIZE: usize = std::mem::size_of::<Self>();    // Size of PoolState
    pub const PREFIX_SEED: &'static [u8] = b"pool";             // Seed of PoolState

    // This function calculates receivable amount on buying tokens
    // Params
    //   self - PoolState struct itself
    //   quote_amount - Amount of quote token to buy with
    // Return
    //   base_amount - Receivable amount of tokens
    pub fn compute_receivable_amount_on_buy(&mut self, quote_amount: u64) -> u64 {
        let base_amount =
            calculate_output_amount(quote_amount, self.virt_quote_reserves + self.real_quote_reserves, self.virt_base_reserves + self.real_base_reserves);
        base_amount
    }

    // This function calculates required amount of quote token on buying tokens
    // Params
    //   self - PoolState struct itself
    //   base_amount - Amount of base tokens to buy
    // Return
    //   quote_amount - Required amount of quote
    pub fn compute_required_amount_on_buy(&mut self, base_amount: u64) -> u64 {
        let quote_amount =
            calculate_input_amount(base_amount, self.virt_quote_reserves + self.real_quote_reserves, self.virt_base_reserves + self.real_base_reserves);
        quote_amount
    }

    // This function calculates receivable amount of quote tokens on selling base tokens
    // Params
    //   self - PoolState struct itself
    //   base_amount - Amount of base tokens to sell
    // Return
    //   quote_amount - Receivable amount of quote tokens
    pub fn compute_receivable_amount_on_sell(&mut self, base_amount: u64) -> u64 {
        let quote_amount =
            calculate_output_amount(base_amount, self.virt_base_reserves + self.real_base_reserves, self.virt_quote_reserves + self.real_quote_reserves);
        quote_amount
    }
}

// This function calculates output amount by using AMM formula
// Params
//   input_amount - Input amount
//   input_reserve - Input reserve
//   output_reserve - Output reserve
// Return
//   output_amount - Output amount
//     output_amount = output_reserve * input_amount / (input_reserve + input_amount)
fn calculate_output_amount(input_amount: u64, input_reserve: u64, output_reserve: u64) -> u64 {
    let output_amount = (output_reserve as u128)
        .checked_mul(input_amount as u128)
        .unwrap()
        .checked_div((input_reserve as u128) + (input_amount as u128))
        .unwrap();
    output_amount as u64
}

// This function calculates input amount by using AMM formula
// Params
//   output_amount - Output amount
//   input_reserve - Input reserve
//   output_reserve - Output reserve
// Return
//   input_amount - Input amount
//     input_amount = output_amount * input_reserve / (output_reserve - output_amount)
fn calculate_input_amount(output_amount: u64, input_reserve: u64, output_reserve: u64) -> u64 {
    let input_amount = (output_amount as u128)
        .checked_mul(input_reserve as u128)
        .unwrap()
        .checked_div((output_reserve as u128) - (output_amount as u128))
        .unwrap();
    input_amount as u64
}
