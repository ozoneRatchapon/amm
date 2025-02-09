use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;

declare_id!("D6FyucvDAm3LxvqYzfncuqHM3hSU3wvZWEQkjw3htXKf");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
