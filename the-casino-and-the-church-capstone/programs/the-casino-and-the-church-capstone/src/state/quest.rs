use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum QuestType {
    Social,      // Follow, retweet, like
    Technical,   // Test protocols, perform on-chain actions
    Creative,    // Create content, memes
    Community,   // Join Discord, Telegram
    Custom,      // Custom quest requirements
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum CompletionStatus {
    Pending,
    Completed,
    Verified,
    Rewarded,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum QuestStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

#[account]
#[derive(InitSpace)]
pub struct QuestCampaign {
    pub creator: Pubkey,
    #[max_len(100)]
    pub title: String,
    #[max_len(500)]
    pub description: String,
    pub quest_type: QuestType,
    pub reward_pool: u64,
    pub max_participants: u32,
    pub current_participants: u32,
    pub start_time: i64,
    pub end_time: i64,
    pub status: QuestStatus,
    #[max_len(200)]
    pub completion_criteria: String,
    #[max_len(200)]
    pub verification_method: String,
    pub is_active: bool,
    pub campaign_counter:u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct QuestParticipation {
    pub user: Pubkey,
    pub campaign: Pubkey,
    pub participation_time: i64,
    pub completion_status: CompletionStatus,
    pub completion_time: Option<i64>,
    pub reward_amount: u64,
    #[max_len(500)]
    pub verification_data: String, // JSON string with verification data
    pub is_verified: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct QuestRewards {
    pub campaign: Pubkey,
    pub total_rewards_distributed: u64,
    pub total_participants_rewarded: u32,
    pub distribution_complete: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct QuestFactory {
    pub authority: Pubkey,
    pub total_campaigns: u32,
    pub total_rewards_distributed: u64,
    pub platform_fee_bps: u16, // Platform fee in basis points
    pub is_active: bool,
    pub bump: u8,
}

// Quest verification data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VerificationData {
    pub quest_type: QuestType,
    pub social_media_handle: Option<String>,
    pub transaction_hash: Option<String>,
    pub proof_of_work: Option<String>,
    pub custom_data: Option<String>,
}

// Quest completion criteria
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompletionCriteria {
    pub min_followers: Option<u32>,
    pub min_engagement: Option<u32>,
    pub required_actions: Vec<String>,
    pub verification_links: Vec<String>,
    pub custom_requirements: Option<String>,
}
