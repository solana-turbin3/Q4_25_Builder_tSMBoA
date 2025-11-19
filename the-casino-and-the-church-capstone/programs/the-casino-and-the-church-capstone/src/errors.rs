use anchor_lang::prelude::*;

#[error_code]
pub enum CasinoError {
    #[msg("Casino is not active")]
    CasinoNotActive,
    
    #[msg("Invalid bet amount")]
    InvalidBetAmount,
    
    #[msg("Bet amount exceeds maximum allowed")]
    BetAmountTooHigh,
    
    #[msg("Bet amount below minimum required")]
    BetAmountTooLow,
    
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    #[msg("Game not found")]
    GameNotFound,
    
    #[msg("Game already completed")]
    GameAlreadyCompleted,
    
    #[msg("Invalid game state")]
    InvalidGameState,
    
    #[msg("VRF request failed")]
    VrfRequestFailed,
    
    #[msg("Invalid payout calculation")]
    InvalidPayoutCalculation,
    
    #[msg("House edge configuration invalid")]
    InvalidHouseEdgeConfig,
    
    #[msg("User statistics not found")]
    UserStatsNotFound,
    
    #[msg("Game type not supported")]
    GameTypeNotSupported,
    
    #[msg("Invalid roulette bet")]
    InvalidRouletteBet,
    
    #[msg("Invalid roulette numbers")]
    InvalidRouletteNumbers,
    
    #[msg("Invalid aviator cashout")]
    InvalidAviatorCashout,
    
    #[msg("Blackjack game not in progress")]
    BlackjackGameNotInProgress,
    
    #[msg("Invalid blackjack action")]
    InvalidBlackjackAction,
    
    #[msg("Slots game not properly initialized")]
    SlotsGameNotInitialized,
    
    #[msg("Invalid slots payline")]
    InvalidSlotsPayline,
}

#[error_code]
pub enum QuestError {
    #[msg("Quest campaign not found")]
    QuestCampaignNotFound,
    
    #[msg("Quest campaign not active")]
    QuestCampaignNotActive,
    
    #[msg("Quest campaign has ended")]
    QuestCampaignEnded,
    
    #[msg("Quest campaign not started")]
    QuestCampaignNotStarted,
    
    #[msg("Maximum participants reached")]
    MaxParticipantsReached,
    
    #[msg("User already participated")]
    UserAlreadyParticipated,
    
    #[msg("User not participated")]
    UserNotParticipated,
    
    #[msg("Quest not completed")]
    QuestNotCompleted,
    
    #[msg("Quest already completed")]
    QuestAlreadyCompleted,
    
    #[msg("Quest verification failed")]
    QuestVerificationFailed,
    
    #[msg("Invalid reward amount")]
    InvalidRewardAmount,
    
    #[msg("Insufficient reward pool")]
    InsufficientRewardPool,
    
    #[msg("Quest rewards already distributed")]
    QuestRewardsAlreadyDistributed,
    
    #[msg("Invalid quest type")]
    InvalidQuestType,
    
    #[msg("Quest factory not active")]
    QuestFactoryNotActive,
    
    #[msg("Invalid completion criteria")]
    InvalidCompletionCriteria,
    
    #[msg("Quest verification data invalid")]
    InvalidVerificationData,
    
    #[msg("Reward distribution not ready")]
    RewardDistributionNotReady,
}

#[error_code]
pub enum LiquidityError {
    #[msg("Liquidity pool not initialized")]
    LiquidityPoolNotInitialized,
    
    #[msg("Liquidity pool not active")]
    LiquidityPoolNotActive,
    
    #[msg("Insufficient LP tokens")]
    InsufficientLPTokens,
    
    #[msg("Invalid staking amount")]
    InvalidStakingAmount,
    
    #[msg("Staking period not valid")]
    InvalidStakingPeriod,
    
    #[msg("Staking not active")]
    StakingNotActive,
    
    #[msg("Staking period not ended")]
    StakingPeriodNotEnded,
    
    #[msg("No rewards to claim")]
    NoRewardsToClaim,
    
    #[msg("Fee distribution not ready")]
    FeeDistributionNotReady,
    
    #[msg("Fee distribution already completed")]
    FeeDistributionAlreadyCompleted,
    
    #[msg("Invalid LP token amount")]
    InvalidLPTokenAmount,
    
    #[msg("LP user stats not found")]
    LPUserStatsNotFound,
    
    #[msg("Governance proposal not found")]
    GovernanceProposalNotFound,
    
    #[msg("Governance proposal not active")]
    GovernanceProposalNotActive,
    
    #[msg("Governance proposal ended")]
    GovernanceProposalEnded,
    
    #[msg("User already voted")]
    UserAlreadyVoted,
    
    #[msg("Invalid vote choice")]
    InvalidVoteChoice,
    
    #[msg("Proposal not executable")]
    ProposalNotExecutable,
    
    #[msg("Insufficient voting power")]
    InsufficientVotingPower,
    
    #[msg("LP token mint not found")]
    LPTokenMintNotFound,
    
    #[msg("Invalid fee share configuration")]
    InvalidFeeShareConfig,
}
