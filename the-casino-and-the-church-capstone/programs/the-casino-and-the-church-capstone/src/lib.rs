use anchor_lang::prelude::*;


pub mod state;
pub mod instructions;
pub mod utils;
pub mod errors;

use state::*;
use instructions::*;
use errors::*;

declare_id!("F2SPV6SKb6VFKdBueJbwbAHuv6Jvbz83ZzHckv5HZ8Bj");

#[program]
pub mod casino_nchurch {
    use super::*;

    // Casino Instructions
    pub fn initialize_casino(ctx: Context<InitializeCasino>) -> Result<()> {
        ctx.accounts.initialize_casino(&ctx.bumps)
    }

    // Slots Game (VRF-based)
    pub fn request_slots_game(ctx: Context<RequestSlotsGame>, bet_amount: u64, randomness_account: Pubkey) -> Result<()> {
        ctx.accounts.request_slots_game(bet_amount, randomness_account, &ctx.bumps)
    }

    pub fn settle_slots_game(ctx: Context<SettleSlotsGame>) -> Result<()> {
        ctx.accounts.settle_slots_game(&ctx.bumps)
    }

    // Roulette Game (VRF-based)
    pub fn request_roulette_game(ctx: Context<RequestRouletteGame>, bet_amount: u64, bet_type: RouletteBetType, numbers: Vec<u8>, randomness_account: Pubkey) -> Result<()> {
        ctx.accounts.request_roulette_game(bet_amount, bet_type, numbers, randomness_account, &ctx.bumps)
    }

    pub fn settle_roulette_game(ctx: Context<SettleRouletteGame>) -> Result<()> {
        ctx.accounts.settle_roulette_game(&ctx.bumps)
    }

    // Quest Instructions
    pub fn create_quest_campaign(ctx: Context<CreateQuestCampaign>, 
                                title: String, 
                                description: String, 
                                reward_pool: u64, 
                                max_participants: u32,
                                quest_type: QuestType,
                                campaign_counter: u64) -> Result<()> {
        ctx.accounts.create_quest_campaign(title, description, reward_pool, max_participants, quest_type, campaign_counter, &ctx.bumps)
    }

    pub fn participate_in_quest(ctx: Context<ParticipateInQuest>, campaign_counter: u64) -> Result<()> {
        ctx.accounts.participate_in_quest(campaign_counter, &ctx.bumps)
    }

    pub fn complete_quest(ctx: Context<CompleteQuest>, campaign_counter: u64) -> Result<()> {
        ctx.accounts.complete_quest(campaign_counter, &ctx.bumps)
    }

    pub fn distribute_quest_rewards(ctx: Context<DistributeQuestRewards>, campaign_counter: u64) -> Result<()> {
        ctx.accounts.distribute_quest_rewards(campaign_counter, &ctx.bumps)
    }

    // Liquidity Pool Instructions
    pub fn initialize_liquidity_pool(ctx: Context<InitializeLiquidityPool>) -> Result<()> {
        ctx.accounts.initialize_liquidity_pool(&ctx.bumps)
    }

    pub fn deposit_liquidity(ctx: Context<DepositLiquidity>, amount: u64) -> Result<()> {
        ctx.accounts.deposit_liquidity(amount, &ctx.bumps)
    }

    pub fn stake_lp_tokens(ctx: Context<StakeLPTokens>, amount: u64, staking_counter: u64) -> Result<()> {
        ctx.accounts.stake_lp_tokens(amount, staking_counter, &ctx.bumps)
    }

    pub fn claim_lp_rewards(ctx: Context<ClaimLPRewards>, staking_counter: u64) -> Result<()> {
        ctx.accounts.claim_lp_rewards(staking_counter, &ctx.bumps)
    }

    pub fn distribute_platform_fees(ctx: Context<DistributePlatformFees>, epoch: u64) -> Result<()> {
        ctx.accounts.distribute_platform_fees(epoch, &ctx.bumps)
    }
}
