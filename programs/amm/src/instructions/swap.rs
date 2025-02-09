use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{errors::AmmError, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        seeds = [b"lp".config.key.as_ref()],
        bump = config.lp_bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Account<'info, Mint>,
    #[account(
    init_if_needed,
    payer = user,
    associated_token::mint = mint_x,
    associated_token::authority = user,
    )]
    pub user_x: Account<'info, TokenAccount>,
    #[account(
    init_if_needed,
    payer = user,
    associated_token::mint = mint_y,
    associated_token::authority = user,
    )]
    pub user_y: Account<'info, TokenAccount>,

    #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
    has_one = mint_x,
    has_one = mint_y,
    seeds = [b"config",config.seed.to_le_bytes().as_ref()],
    bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
impl<'info> Swap<'info> {
    pub fn swap(&mut self, is_x: bool, amount_in: u64, min_amount_out: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount_in > 0, AmmError::InvalidAmount);

        let mut curve = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            self.config.fee,
            None,
        )
        .map_err(AmmError::from)?;

        let p = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };

        let res = curve
            .swap(p, amount_in, min_amount_out)
            .map_err(AmmError::from)?;

        require!(res.deposit != 0, AmmError::InvalidAmount);
        require!(res.withdraw != 0, AmmError::InvalidAmount);

        // deposit tokens
        self.deposit_token(is_x, res.deposit)?;
        // withdraw tokens
        self.withdraw_token(is_x, res.withdraw)?;
        // transfer fee

        Ok(())
    }

    pub fn deposit_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
            ),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_account = Transfer {
            from: from.to_account_info(), // user_x or user_y
            to: to.to_account_info(),     // vault_x or vault_y
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw_token(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            ),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_account = Transfer {
            from: from.to_account_info(), // user_x or user_y
            to: to.to_account_info(),     // vault_x or vault_y
            authority: self.config.to_account_info(),
        };

        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}
