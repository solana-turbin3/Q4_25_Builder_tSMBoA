use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum StakingPeriod {
    Short,   // 30 days
    Medium,  // 90 days
    Long,    // 180 days
    Ultra,   // 365 days
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum LPStatus {
    Active,
    Paused,
    Closed,
}

#[account]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub authority: Pubkey,
    pub lp_token_mint: Pubkey,
    pub total_liquidity: u64,
    pub lp_token_supply: u64,
    pub platform_fee_share_bps: u16, // % of platform fees to LPs
    pub staking_rewards_apr: u16,     // Annual percentage rate for staking
    pub total_fees_distributed: u64,
    pub total_staking_rewards: u64,
    pub status: LPStatus,
    pub bump: u8,
    pub lp_vault_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct LPStaking {
    pub user: Pubkey,
    pub lp_tokens_staked: u64,
    pub staking_period: StakingPeriod,
    pub staking_start_time: i64,
    pub staking_end_time: i64,
    pub rewards_earned: u64,
    pub last_claim_time: i64,
    pub is_active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct LPUserStats {
    pub user: Pubkey,
    pub total_lp_tokens: u64,
    pub total_staked: u64,
    pub total_rewards_claimed: u64,
    pub total_fees_earned: u64,
    pub staking_count: u32,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct FeeDistribution {
    pub epoch: u64,
    pub total_platform_fees: u64,
    pub lp_fee_share: u64,
    pub platform_fee_share: u64,
    pub distribution_complete: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct LPGovernance {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    #[max_len(100)]
    pub title: String,
    #[max_len(500)]
    pub description: String,
    pub proposal_type: ProposalType,
    pub votes_for: u64,
    pub votes_against: u64,
    pub total_votes: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub is_executed: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum ProposalType {
    HouseEdgeChange,
    PlatformFeeChange,
    LPFeeShareChange,
    NewGameAddition,
    QuestPolicyChange,
    EmergencyPause,
}

#[account]
#[derive(InitSpace)]
pub struct LPGovernanceVote {
    pub user: Pubkey,
    pub proposal: Pubkey,
    pub vote_weight: u64,
    pub vote_choice: VoteChoice,
    pub vote_time: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

// LP Token economics constants
pub const LP_TOKEN_DECIMALS: u8 = 6;
pub const MIN_STAKING_AMOUNT: u64 = 1000; // Minimum LP tokens to stake
pub const MAX_STAKING_PERIOD: i64 = 365 * 24 * 60 * 60; // 1 year in seconds

// Staking period multipliers
pub const SHORT_STAKING_MULTIPLIER: u16 = 100;   // 1x
pub const MEDIUM_STAKING_MULTIPLIER: u16 = 150;  // 1.5x
pub const LONG_STAKING_MULTIPLIER: u16 = 200;    // 2x
pub const ULTRA_STAKING_MULTIPLIER: u16 = 300;   // 3x

// Fee distribution constants
pub const DEFAULT_PLATFORM_FEE_SHARE_BPS: u16 = 3000; // 30% to LPs
pub const DEFAULT_STAKING_REWARDS_APR: u16 = 1200;     // 12% APR
pub const FEE_DISTRIBUTION_EPOCH: i64 = 7 * 24 * 60 * 60; // 7 days
