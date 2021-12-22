use anchor_lang::prelude::*;

declare_id!("HV1tYVquon8bA4ZSbVLF361qrQ1vcnev2yhnbHyCPQWJ");

const PREFIX: &str = "prefix";
const ESCROW: &str = "escrow";

#[program]
pub mod unbalanced_transfer_poc {
    use super::*;
    use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

    pub fn initialize(
        ctx: Context<Initialize>,
        prefix_bump: u8,
        _escrow_bump: u8,
        amount: u64,
    ) -> ProgramResult {
        let prefix_account = &mut ctx.accounts.prefix_account;
        let escrow_account = &mut ctx.accounts.escrow_account;
        let payer = &mut ctx.accounts.payer;
        let system_program = &ctx.accounts.system_program;

        prefix_account.bump = prefix_bump;
        prefix_account.amount = amount;

        invoke_signed(
            &system_instruction::transfer(&payer.key(), &escrow_account.key, amount),
            &[
                payer.to_account_info(),
                escrow_account.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;

        Ok(())
    }

    pub fn close(ctx: Context<Close>, escrow_bump: u8) -> ProgramResult {
        let prefix_account = &mut ctx.accounts.prefix_account;
        let escrow_account = &mut ctx.accounts.escrow_account;
        let user = &mut ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;

        let user_key = user.key();

        let escrow_seeds = [
            PREFIX.as_bytes(),
            user_key.as_ref(),
            ESCROW.as_bytes(),
            &[escrow_bump],
        ];

        let prefix_account_info = prefix_account.to_account_info();
        let user_account_info = user.to_account_info();
        let curr_lamports = prefix_account_info.lamports();

        msg!(
            "User lamports before close prefix account: {:?}",
            user_account_info.lamports()
        );

        // Close one account (owned by system program)
        **prefix_account_info.lamports.borrow_mut() = 0;
        **user_account_info.lamports.borrow_mut() = user_account_info
            .lamports()
            .checked_add(curr_lamports)
            .ok_or(ErrorCode::NumericalOverflow)?;

        msg!(
            "User lamports before after prefix account: {:?}",
            user_account_info.lamports()
        );

        // Close PDA account
        invoke_signed(
            &system_instruction::transfer(
                &escrow_account.key(),
                &user.key(),
                escrow_account.lamports(),
            ),
            &[
                escrow_account.to_account_info(),
                user.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&escrow_seeds],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(prefix_bump: u8, escrow_bump: u8, amount: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [PREFIX.as_bytes(), payer.key().as_ref(), &amount.to_le_bytes()],
        bump = prefix_bump
    )]
    pub prefix_account: Account<'info, Prefix>,
    #[account(
        mut,
        seeds = [PREFIX.as_bytes(), payer.key().as_ref(), ESCROW.as_bytes()],
        bump = escrow_bump
    )]
    pub escrow_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(escrow_bump: u8)]
pub struct Close<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [PREFIX.as_bytes(), user.key().as_ref(), &prefix_account.amount.to_le_bytes()],
        bump = prefix_account.bump
    )]
    pub prefix_account: Account<'info, Prefix>,
    #[account(
        mut,
        seeds = [PREFIX.as_bytes(), user.key().as_ref(), ESCROW.as_bytes()],
        bump = escrow_bump
    )]
    pub escrow_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct Prefix {
    bump: u8,
    amount: u64,
}

#[error]
pub enum ErrorCode {
    #[msg("Numerical overflow")]
    NumericalOverflow,
}
