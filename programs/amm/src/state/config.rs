use anchor_lang::prelude::*;

#[account]
pub struct Config {
    pub seed: u64,                 // seed to be able to create different pools / configs
    pub authority: Option<Pubkey>, // If we want an authority to lock the config account
    pub mint_x: Pubkey,            // mint of token x
    pub mint_y: Pubkey,            // mint of token y
    pub fee: u16,                  // Swap fee in basis points
    pub locked: bool,              // If the pool is locked
    pub config_bump: u8,           // bump seed for config account
    pub lp_bump: u8,               // bump seed for lp token
}

impl Space for Config {
    const INIT_SPACE: usize = 8 + 8 + (1 + 32) + 32 + 32 + 2 + 1 + 1 + 1; // ? check today video
}
