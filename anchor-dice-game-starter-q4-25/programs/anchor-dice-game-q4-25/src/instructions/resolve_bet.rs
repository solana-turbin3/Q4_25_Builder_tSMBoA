use anchor_lang::prelude::*;
use anchor_instruction_sysvar::Ed25519InstructionSignatures;
use anchor_lang::system_program::{Transfer, transfer};
use solana_program::ed25519_program;
use solana_program::hash::hash;
use solana_program::sysvar::instructions::{load_current_index_checked, load_instruction_at_checked};
use crate::Bet;
use crate::errors::DiceError;

const HOUSE_EDGE_BPS: u16 = 150;

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(
        mut,
        address = bet.player
    )]
    /// CHECK: Player. Checked in bet account
    pub player: UncheckedAccount<'info>,
    pub house: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        close = player,
        seeds = [b"bet", vault.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump
    )]
    pub bet: Account<'info, Bet>,
    #[account(
        address = sysvar::instructions::ID @ DiceError::InstructionSysvarNotFound,
    )]
    /// CHECK: Sysvar instruction. Unknown type
    pub instruction_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn resolve_bet(&mut self, sig: &[u8], bumps: &ResolveBetBumps) -> Result<()> {
        let _hash = hash(sig).to_bytes();

        let mut hash_16 = [0; 16];
        hash_16.copy_from_slice(&_hash[0..16]);
        let lower = u128::from_le_bytes(hash_16);

        hash_16.copy_from_slice(&_hash[16..32]);
        let upper = u128::from_le_bytes(hash_16);

        let roll = lower
            .wrapping_add(upper)
            .wrapping_rem(100) as u8 + 1;

        if self.bet.roll > roll {
            let bps = 10000;
            let payout = (self.bet.amount as u128)
                .checked_mul((bps - HOUSE_EDGE_BPS) as u128).unwrap()
                .checked_div(self.bet.roll as u128).unwrap()
                .checked_div(bps as u128).unwrap() as u64;

            let signer_seeds: [&[&[u8]]; 1] =
                [&[b"vault", &self.house.key().to_bytes(), &[bumps.vault]]];

            let transfer_ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.vault.to_account_info(),
                    to: self.player.to_account_info(),
                },
                &signer_seeds
            );

            transfer(transfer_ctx, payout)?;
        }

        Ok(())
    }

    pub fn verify_ed25519_signature(&self, sig: &[u8]) -> Result<()> {
        let cur_index = load_current_index_checked(&self.instruction_sysvar.to_account_info())? as usize;
        let ix = load_instruction_at_checked(cur_index-1, &self.instruction_sysvar.to_account_info())?;

        require_keys_eq!(ix.program_id, ed25519_program::ID, DiceError::Ed25519Program);

        require_eq!(ix.accounts.len(), 0, DiceError::Ed25519Accounts);
        
        let signatures = Ed25519InstructionSignatures::unpack(ix.data.as_slice()).unwrap().0;
        
        require_eq!(signatures.len(), 1, DiceError::Ed25519DataLength);
        let signature = signatures.get(0).ok_or(DiceError::Ed25519Signature)?;

        require!(&signature.is_verifiable, DiceError::Ed25519Signature);

        require_keys_eq!(signature.public_key.unwrap(), self.house.key(), DiceError::Ed25519Pubkey);

        require!(signature.signature.unwrap().eq(sig), DiceError::Ed25519Signature);

        require!(signature.message.as_ref().unwrap().eq(self.bet.to_slice().as_slice()), DiceError::Ed25519Message);

        Ok(())
    }
}