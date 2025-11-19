use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer, MintTo};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::liquidity::*;
use crate::errors::*;

// Initialize liquidity pool
#[derive(Accounts)]
pub struct InitializeLiquidityPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + LiquidityPool::INIT_SPACE,
        seeds = [b"liquidity_pool"],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = liquidity_pool,
        seeds = [b"lp_token_mint"],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint,
        token::authority = liquidity_pool,
        seeds = [b"lp_vault"],
        bump
    )]
    pub lp_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub usdc_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> InitializeLiquidityPool<'info> {
    pub fn initialize_liquidity_pool(&mut self, bumps: &InitializeLiquidityPoolBumps) -> Result<()> {
        let liquidity_pool = &mut self.liquidity_pool;

        liquidity_pool.set_inner(LiquidityPool {
            authority: self.authority.key(),
            lp_token_mint: self.lp_token_mint.key(),
            total_liquidity: 0,
            lp_token_supply: 0,
            platform_fee_share_bps: DEFAULT_PLATFORM_FEE_SHARE_BPS,
            staking_rewards_apr: DEFAULT_STAKING_REWARDS_APR,
            total_fees_distributed: 0,
            total_staking_rewards: 0,
            status: LPStatus::Active,
            bump: bumps.liquidity_pool,
            lp_vault_bump: bumps.lp_vault,
        });
        
        msg!("Liquidity pool initialized successfully");
        Ok(())
    }
}

// Deposit liquidity
#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(
        mut,
        seeds = [b"liquidity_pool"],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        mut,
        seeds = [b"lp_token_mint"],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"lp_vault"],
        bump
    )]
    pub lp_vault: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_token_mint,
        associated_token::authority = user,
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + LPUserStats::INIT_SPACE,
        seeds = [b"lp_user_stats", user.key().as_ref()],
        bump
    )]
    pub lp_user_stats: Account<'info, LPUserStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> DepositLiquidity<'info> {
    pub fn deposit_liquidity(&mut self, amount: u64, bumps: &DepositLiquidityBumps) -> Result<()> {
        let liquidity_pool = &mut self.liquidity_pool;
        let lp_user_stats = &mut self.lp_user_stats;
        
        // Validate liquidity pool is active
        require!(liquidity_pool.status == LPStatus::Active, LiquidityError::LiquidityPoolNotActive);
        
        // Validate deposit amount
        require!(amount >= 1000000, LiquidityError::InvalidLPTokenAmount); // Min 1 USDC
        
        // Transfer USDC from user to LP vault
        let transfer_instruction = Transfer {
            from: self.user_token_account.to_account_info(),
            to: self.lp_vault.to_account_info(),
            authority: self.user.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, amount)?;
        
        // Calculate LP tokens to mint (1:1 ratio for simplicity)
        let lp_tokens_to_mint = amount;
        
        // Mint LP tokens to user
        let mint_instruction = MintTo {
            mint: self.lp_token_mint.to_account_info(),
            to: self.user_lp_token_account.to_account_info(),
            authority: liquidity_pool.to_account_info(),
        };
        
        let liquidity_pool_bump = liquidity_pool.bump;
        let seeds : &[&[&[u8]]] = &[&[b"lp_vault", &[liquidity_pool_bump]]];
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), mint_instruction, seeds);
        
        token::mint_to(cpi_ctx, lp_tokens_to_mint)?;
        
        // Update liquidity pool
        liquidity_pool.total_liquidity += amount;
        liquidity_pool.lp_token_supply += lp_tokens_to_mint;
        
        // Update user stats
        lp_user_stats.user = self.user.key();
        lp_user_stats.total_lp_tokens += lp_tokens_to_mint;
        lp_user_stats.bump = bumps.lp_user_stats;
        
        msg!("Liquidity deposited: User: {}, Amount: {}, LP Tokens: {}", 
            self.user.key(), amount, lp_tokens_to_mint);
        Ok(())
    }
}

// Stake LP tokens
#[derive(Accounts)]
#[instruction(staking_counter: u64)]
pub struct StakeLPTokens<'info> {
    #[account(
        mut,
        seeds = [b"liquidity_pool"],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        init,
        payer = user,
        space = 8 + LPStaking::INIT_SPACE,
        seeds = [b"lp_staking", user.key().as_ref(), &staking_counter.to_le_bytes()],
        bump
    )]
    pub lp_staking: Account<'info, LPStaking>,
    
    #[account(
        mut,
        seeds = [b"lp_user_stats", user.key().as_ref()],
        bump = lp_user_stats.bump
    )]
    pub lp_user_stats: Account<'info, LPUserStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> StakeLPTokens<'info> {
    pub fn stake_lp_tokens(&mut self, amount: u64, staking_counter: u64, bumps: &StakeLPTokensBumps) -> Result<()> {
        let liquidity_pool = &mut self.liquidity_pool;
        let lp_staking = &mut self.lp_staking;
        let lp_user_stats = &mut self.lp_user_stats;
        
        // Validate liquidity pool is active
        require!(liquidity_pool.status == LPStatus::Active, LiquidityError::LiquidityPoolNotActive);
        
        // Validate staking amount
        require!(amount >= MIN_STAKING_AMOUNT, LiquidityError::InvalidStakingAmount);
        
        // Transfer LP tokens from user to staking account
        let transfer_instruction = Transfer {
            from: self.user_lp_token_account.to_account_info(),
            to: lp_staking.to_account_info(),
            authority: self.user.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, amount)?;
        
        // Calculate staking period (default to medium term)
        let staking_period = StakingPeriod::Medium;
        let staking_duration = match staking_period {
            StakingPeriod::Short => 30 * 24 * 60 * 60,   // 30 days
            StakingPeriod::Medium => 90 * 24 * 60 * 60,  // 90 days
            StakingPeriod::Long => 180 * 24 * 60 * 60,   // 180 days
            StakingPeriod::Ultra => 365 * 24 * 60 * 60,  // 365 days
        };
        
        let current_time = Clock::get().unwrap().unix_timestamp;

        lp_staking.set_inner(LPStaking {
            user: self.user.key(),
            lp_tokens_staked: amount,
            staking_period: staking_period,
            staking_start_time: current_time,
            staking_end_time: current_time + staking_duration,
            rewards_earned: 0,
            last_claim_time: current_time,
            is_active: true,
            bump: bumps.lp_staking,
        });
        
        // Initialize staking
        
        
        // Update user stats
        lp_user_stats.total_staked += amount;
        lp_user_stats.staking_count += 1;
        
        msg!("LP tokens staked successfully");
        Ok(())
    }
}

// Claim LP rewards
#[derive(Accounts)]
#[instruction(staking_counter: u64)]
pub struct ClaimLPRewards<'info> {
    #[account(
        mut,
        seeds = [b"liquidity_pool"],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        mut,
        seeds = [b"lp_vault"],
        bump = liquidity_pool.lp_vault_bump
    )]
    pub lp_vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"lp_staking", user.key().as_ref(), &staking_counter.to_le_bytes()],
        bump = lp_staking.bump
    )]
    pub lp_staking: Account<'info, LPStaking>,
    
    #[account(
        mut,
        seeds = [b"lp_user_stats", user.key().as_ref()],
        bump = lp_user_stats.bump
    )]
    pub lp_user_stats: Account<'info, LPUserStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimLPRewards<'info> {
    pub fn claim_lp_rewards(&mut self, staking_counter: u64, bumps: &ClaimLPRewardsBumps) -> Result<()> {
        let liquidity_pool = &mut self.liquidity_pool;
        let lp_staking = &mut self.lp_staking;
        let lp_user_stats = &mut self.lp_user_stats;
        
        // Validate staking is active
        require!(lp_staking.is_active, LiquidityError::StakingNotActive);
        
        // Calculate rewards
        let current_time = Clock::get().unwrap().unix_timestamp;
        let time_elapsed = current_time - lp_staking.last_claim_time;
        
        let staking_multiplier = match lp_staking.staking_period {
            StakingPeriod::Short => SHORT_STAKING_MULTIPLIER,
            StakingPeriod::Medium => MEDIUM_STAKING_MULTIPLIER,
            StakingPeriod::Long => LONG_STAKING_MULTIPLIER,
            StakingPeriod::Ultra => ULTRA_STAKING_MULTIPLIER,
        };
        
        let base_apr = liquidity_pool.staking_rewards_apr;
        let adjusted_apr = (base_apr * staking_multiplier) / 100;
        
        let rewards = calculate_staking_rewards(
            lp_staking.lp_tokens_staked,
            adjusted_apr,
            time_elapsed
        );
        
        require!(rewards > 0, LiquidityError::NoRewardsToClaim);
        
        // Transfer rewards to user
        let transfer_instruction = Transfer {
            from: self.lp_vault.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: liquidity_pool.to_account_info(),
        };
        
        let liquidity_pool_bump = liquidity_pool.bump;
        let seeds : &[&[&[u8]]] = &[&[b"lp_vault", &[liquidity_pool_bump]]];
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_instruction, seeds);
        
        token::transfer(cpi_ctx, rewards)?;
        
        // Update staking
        lp_staking.rewards_earned += rewards;
        lp_staking.last_claim_time = current_time;
        
        // Update user stats
        lp_user_stats.total_rewards_claimed += rewards;
        
        // Update liquidity pool
        liquidity_pool.total_staking_rewards += rewards;
        
        msg!("LP rewards claimed: User: {}, Amount: {}", 
            self.user.key(), rewards);
        Ok(())
    }
}

// Distribute platform fees
#[derive(Accounts)]
#[instruction(epoch: u64)]
pub struct DistributePlatformFees<'info> {
    #[account(
        mut,
        seeds = [b"liquidity_pool"],
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + FeeDistribution::INIT_SPACE,
        seeds = [b"fee_distribution", epoch.to_le_bytes().as_ref()],
        bump
    )]
    pub fee_distribution: Account<'info, FeeDistribution>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> DistributePlatformFees<'info> {
    pub fn distribute_platform_fees(&mut self, epoch: u64, bumps: &DistributePlatformFeesBumps) -> Result<()> {
        let liquidity_pool = &mut self.liquidity_pool;
        let fee_distribution = &mut self.fee_distribution;
        
        // Validate liquidity pool is active
        require!(liquidity_pool.status == LPStatus::Active, LiquidityError::LiquidityPoolNotActive);
        
        // Calculate platform fees to distribute (placeholder - in production, this would come from actual platform fees)
        let total_platform_fees = 1000000; // 1 USDC placeholder
        
        // Calculate LP fee share
        let lp_fee_share = calculate_lp_fee_share(total_platform_fees, liquidity_pool.platform_fee_share_bps);
        let platform_fee_share = total_platform_fees - lp_fee_share;

        fee_distribution.set_inner(FeeDistribution {
            epoch: (Clock::get().unwrap().unix_timestamp / FEE_DISTRIBUTION_EPOCH) as u64,
            total_platform_fees: total_platform_fees,
            lp_fee_share: lp_fee_share,
            platform_fee_share: platform_fee_share,
            distribution_complete: false,
            bump: bumps.fee_distribution,
        });
        
        // Initialize fee distribution
        
        // Update liquidity pool
        liquidity_pool.total_fees_distributed += lp_fee_share;
        
        msg!("Platform fees distributed: Total: {}, LP Share: {}, Platform Share: {}", 
            total_platform_fees, lp_fee_share, platform_fee_share);
        Ok(())
    }
}

// Helper function to calculate staking rewards
fn calculate_staking_rewards(staked_amount: u64, apr_bps: u16, time_elapsed: i64) -> u64 {
    let seconds_per_year = 365 * 24 * 60 * 60;
    let time_elapsed_seconds = time_elapsed as u64;
    
    if time_elapsed_seconds >= seconds_per_year {
        (staked_amount * apr_bps as u64) / 10000
    } else {
        (staked_amount * apr_bps as u64 * time_elapsed_seconds) / (10000 * seconds_per_year)
    }
}

// Helper function to calculate LP fee share
fn calculate_lp_fee_share(total_fees: u64, lp_share_bps: u16) -> u64 {
    (total_fees * lp_share_bps as u64) / 10000
}
