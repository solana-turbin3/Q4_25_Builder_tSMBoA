use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::quest::*;
use crate::errors::*;

// Create quest campaign
#[derive(Accounts)]
#[instruction(campaign_counter: u64)]
pub struct CreateQuestCampaign<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + QuestCampaign::INIT_SPACE,
        seeds = [b"quest_campaign", creator.key().as_ref(), &campaign_counter.to_le_bytes()],
        bump
    )]
    pub quest_campaign: Account<'info, QuestCampaign>,
    
    #[account(
        init,
        payer = creator,
        token::mint = usdc_mint,
        token::authority = quest_campaign,
        seeds = [b"quest_vault", creator.key().as_ref(), &campaign_counter.to_le_bytes()],
        bump
    )]
    pub quest_vault: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = creator,
        space = 8 + QuestRewards::INIT_SPACE,
        seeds = [b"quest_rewards", creator.key().as_ref(), &campaign_counter.to_le_bytes()],
        bump
    )]
    pub quest_rewards: Account<'info, QuestRewards>,
    
    #[account(
        init_if_needed,
        payer = creator,
        space = 8 + QuestFactory::INIT_SPACE,
        seeds = [b"quest_factory"],
        bump
    )]
    pub quest_factory: Account<'info, QuestFactory>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    pub usdc_mint: Account<'info, token::Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateQuestCampaign<'info> {
    pub fn create_quest_campaign(
        &mut self,
        title: String,
        description: String,
        reward_pool: u64,
        max_participants: u32,
        quest_type: QuestType,
        campaign_counter: u64,
        bumps: &CreateQuestCampaignBumps,
    ) -> Result<()> {
        let quest_campaign = &mut self.quest_campaign;
        let quest_factory = &mut self.quest_factory;
        
        // Validate quest factory is active
        require!(quest_factory.is_active, QuestError::QuestFactoryNotActive);
        
        // Validate reward pool
        require!(reward_pool >= 1000000, QuestError::InvalidRewardAmount); // Min 1 USDC
        require!(reward_pool <= 1000000000, QuestError::InvalidRewardAmount); // Max 1000 USDC
        
        // Validate max participants
        require!(max_participants > 0, QuestError::InvalidCompletionCriteria);
        require!(max_participants <= 10000, QuestError::InvalidCompletionCriteria);
        
        // Calculate platform fee
        let platform_fee = calculate_platform_fee(reward_pool, quest_factory.platform_fee_bps);
        let net_reward_pool = reward_pool - platform_fee;
        
        // Transfer reward pool from creator to quest vault
        let transfer_instruction = Transfer {
            from: self.creator_token_account.to_account_info(),
            to: self.quest_vault.to_account_info(),
            authority: self.creator.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_instruction,
        );
        
        token::transfer(cpi_ctx, reward_pool)?;

        quest_campaign.set_inner(QuestCampaign {
            creator: self.creator.key(),
            title: title.clone(),
            description: description,
            quest_type: quest_type,
            reward_pool: net_reward_pool,
            max_participants: max_participants,
            current_participants: 0,
            start_time: Clock::get().unwrap().unix_timestamp,
            end_time: Clock::get().unwrap().unix_timestamp + (30 * 24 * 60 * 60), // 30 days
            status: QuestStatus::Active,
            completion_criteria: "Complete the quest requirements".to_string(),
            verification_method: "Manual verification".to_string(),
            is_active: true,
            campaign_counter,
            bump: bumps.quest_campaign,
        });

        
        
        // Initialize quest rewards
        let quest_rewards = &mut self.quest_rewards;
        quest_rewards.set_inner(QuestRewards {
            campaign: quest_campaign.key(),
            total_rewards_distributed: 0,
            total_participants_rewarded: 0,
            distribution_complete: false,
            bump: bumps.quest_rewards,
        });
        
        // Update quest factory
        quest_factory.total_campaigns += 1;
        quest_factory.total_rewards_distributed += platform_fee;
        
        msg!("Quest campaign created: Title: {}, Reward Pool: {}, Max Participants: {}", 
            title, net_reward_pool, max_participants);
        Ok(())
    }

}

// Participate in quest
#[derive(Accounts)]
#[instruction(campaign_counter: u64)]
pub struct ParticipateInQuest<'info> {
    #[account(
        mut,
        seeds = [b"quest_campaign", quest_campaign.creator.as_ref(), &campaign_counter.to_le_bytes()],
        bump = quest_campaign.bump
    )]
    pub quest_campaign: Account<'info, QuestCampaign>,
    
    #[account(
        init,
        payer = user,
        space = 8 + QuestParticipation::INIT_SPACE,
        seeds = [b"quest_participation", user.key().as_ref(), quest_campaign.key().as_ref()],
        bump
    )]
    pub quest_participation: Account<'info, QuestParticipation>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> ParticipateInQuest<'info> {
    pub fn participate_in_quest(&mut self, campaign_counter: u64, bumps: &ParticipateInQuestBumps) -> Result<()> {
        let quest_campaign = &mut self.quest_campaign;
        let quest_participation = &mut self.quest_participation;
        
        // Validate quest campaign is active
        require!(quest_campaign.is_active, QuestError::QuestCampaignNotActive);
        require!(quest_campaign.status == QuestStatus::Active, QuestError::QuestCampaignNotActive);
        
        // Validate quest campaign timing
        let current_time = Clock::get().unwrap().unix_timestamp;
        require!(current_time >= quest_campaign.start_time, QuestError::QuestCampaignNotStarted);
        require!(current_time <= quest_campaign.end_time, QuestError::QuestCampaignEnded);
        
        // Validate max participants
        require!(quest_campaign.current_participants < quest_campaign.max_participants, 
                QuestError::MaxParticipantsReached);

        quest_participation.set_inner(QuestParticipation {
            user: self.user.key(),
            campaign: quest_campaign.key(),
            participation_time: current_time,
            completion_status: CompletionStatus::Pending,
            completion_time: None,
            reward_amount: 0,
            verification_data: "".to_string(),
            is_verified: false,
            bump: bumps.quest_participation,
        });
        
        // Update quest campaign
        quest_campaign.current_participants += 1;
        
        msg!("User participated in quest: User: {}, Campaign: {}", 
            self.user.key(), quest_campaign.key());
        Ok(())
    }
}

// Complete quest
#[derive(Accounts)]
#[instruction(campaign_counter: u64)]
pub struct CompleteQuest<'info> {
    #[account(
        mut,
        seeds = [b"quest_campaign", quest_campaign.creator.as_ref(), &campaign_counter.to_le_bytes()],
        bump = quest_campaign.bump
    )]
    pub quest_campaign: Account<'info, QuestCampaign>,
    
    #[account(
        mut,
        seeds = [b"quest_participation", user.key().as_ref(), quest_campaign.key().as_ref()],
        bump = quest_participation.bump
    )]
    pub quest_participation: Account<'info, QuestParticipation>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

impl <'info> CompleteQuest<'info> {
    pub fn complete_quest(&mut self, campaign_counter: u64, bumps: &CompleteQuestBumps) -> Result<()> {
        let quest_campaign = &mut self.quest_campaign;
        let quest_participation = &mut self.quest_participation;
        
        // Validate quest campaign is active
        require!(quest_campaign.is_active, QuestError::QuestCampaignNotActive);
        require!(quest_campaign.status == QuestStatus::Active, QuestError::QuestCampaignNotActive);
        
        // Validate quest campaign timing
        let current_time = Clock::get().unwrap().unix_timestamp;
        require!(current_time >= quest_campaign.start_time, QuestError::QuestCampaignNotStarted);
        require!(current_time <= quest_campaign.end_time, QuestError::QuestCampaignEnded);
        
        // Validate user participation
        require!(quest_participation.user == self.user.key(), QuestError::UserNotParticipated);
        require!(quest_participation.completion_status == CompletionStatus::Pending, 
                QuestError::QuestAlreadyCompleted);
        
        // Calculate reward amount
        let reward_per_participant = quest_campaign.reward_pool / (quest_campaign.max_participants as u64);
        
        // Update quest participation
        quest_participation.completion_status = CompletionStatus::Completed;
        quest_participation.completion_time = Some(current_time);
        quest_participation.reward_amount = reward_per_participant;
        quest_participation.verification_data = "Quest completed successfully".to_string();
        quest_participation.is_verified = true;
        
        msg!("Quest completed: User: {}, Campaign: {}, Reward: {}", 
            self.user.key(), quest_campaign.key(), reward_per_participant);
        Ok(())
    }
}
// Distribute quest rewards
#[derive(Accounts)]
#[instruction(campaign_counter: u64)]
pub struct DistributeQuestRewards<'info> {
    #[account(
        mut,
        seeds = [b"quest_campaign", quest_campaign.creator.as_ref(), &campaign_counter.to_le_bytes()],
        bump = quest_campaign.bump
    )]
    pub quest_campaign: Account<'info, QuestCampaign>,
    
    #[account(
        mut,
        seeds = [b"quest_vault", quest_campaign.creator.as_ref(), &campaign_counter.to_le_bytes()],
        bump
    )]
    pub quest_vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"quest_rewards", quest_campaign.creator.as_ref(), &campaign_counter.to_le_bytes()],
        bump = quest_rewards.bump
    )]
    pub quest_rewards: Account<'info, QuestRewards>,
    
    #[account(
        mut,
        seeds = [b"quest_participation", creator.key().as_ref(), quest_campaign.key().as_ref()],
        bump = quest_participation.bump
    )]
    pub quest_participation: Account<'info, QuestParticipation>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}
impl<'info> DistributeQuestRewards<'info> {
    pub fn distribute_quest_rewards(&mut self, campaign_counter: u64, bumps: &DistributeQuestRewardsBumps) -> Result<()> {
        let quest_campaign = &mut self.quest_campaign;
        let quest_rewards = &mut self.quest_rewards;
        let quest_participation = &mut self.quest_participation;
        
        // Validate quest campaign has ended
        let current_time = Clock::get().unwrap().unix_timestamp;
        require!(current_time > quest_campaign.end_time, QuestError::RewardDistributionNotReady);
        
        // Validate quest participation is completed
        require!(quest_participation.completion_status == CompletionStatus::Completed, 
                QuestError::QuestNotCompleted);
        require!(quest_participation.is_verified, QuestError::QuestVerificationFailed);
        
        // Validate rewards not already distributed
        require!(!quest_rewards.distribution_complete, QuestError::QuestRewardsAlreadyDistributed);
        
        // Validate reward amount
        require!(quest_participation.reward_amount > 0, QuestError::InvalidRewardAmount);
        
        // Transfer reward to user
        let transfer_instruction = Transfer {
            from: self.quest_vault.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: quest_campaign.to_account_info(),
        };
        
        let campaign_counter_bytes = campaign_counter.to_le_bytes();
        let quest_campaign_bump = quest_campaign.bump;
        let seeds : &[&[&[u8]]] = &[&[b"quest_vault", quest_campaign.creator.as_ref(), &campaign_counter_bytes, &[quest_campaign_bump]]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_instruction,
            seeds,
        );
        
        token::transfer(cpi_ctx, quest_participation.reward_amount)?;
        
        // Update quest participation
        quest_participation.completion_status = CompletionStatus::Rewarded;
        
        // Update quest rewards
        quest_rewards.total_rewards_distributed += quest_participation.reward_amount;
        quest_rewards.total_participants_rewarded += 1;
        
        // Check if all rewards have been distributed
        if quest_rewards.total_participants_rewarded >= quest_campaign.current_participants {
            quest_rewards.distribution_complete = true;
            quest_campaign.status = QuestStatus::Completed;
        }
        
        msg!("Quest rewards distributed: User: {}, Amount: {}", 
            self.user_token_account.key(), quest_participation.reward_amount);
        Ok(())
    }

}

// Helper function to calculate platform fee
fn calculate_platform_fee(amount: u64, fee_bps: u16) -> u64 {
    (amount * fee_bps as u64) / 10000
}
