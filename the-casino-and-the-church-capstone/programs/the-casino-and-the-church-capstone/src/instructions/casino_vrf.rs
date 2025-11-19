use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use crate::state::casino::*;
use crate::utils::*;
use crate::errors::*;


// Initialize casino
#[derive(Accounts)]
pub struct InitializeCasino<'info> {
    #[account(
        init,
        payer = authority,
        space = CasinoState::DISCRIMINATOR.len() + CasinoState::INIT_SPACE,
        seeds = [b"casino_state"],
        bump
    )]
    pub casino_state: Account<'info, CasinoState>,
    
    #[account(
        init,
        payer = authority,
        associated_token::mint = usdc_mint,
        associated_token::authority = casino_state,
        associated_token::token_program = token_program,
    )]
    pub casino_vault: InterfaceAccount<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub usdc_mint: InterfaceAccount<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl <'info> InitializeCasino<'info> {
    pub fn initialize_casino(&mut self, bumps: &InitializeCasinoBumps) -> Result<()> {
        self.casino_state.set_inner(CasinoState {
            authority: self.authority.key(),
            vault: self.casino_vault.key(),
            total_games_played: 0,
            total_volume: 0,
            total_payouts: 0,
            house_edge_config: HouseEdgeConfig {
                slots_rtp_bps: 9500,      // 95% RTP
                roulette_rtp_bps: 9730,   // 97.3% RTP
                aviator_rtp_bps: 9600,    // 96% RTP
                blackjack_rtp_bps: 9950,   // 99.5% RTP
                platform_fee_bps: 200,    // 2% platform fee
            },
            is_active: true,
            casino_state_bump: bumps.casino_state
        });
        
        msg!("Casino initialized successfully");
        Ok(())
    }
}

// Request slots game (Phase 1: Request randomness)
#[derive(Accounts)]
pub struct RequestSlotsGame<'info> {
    #[account(
        mut,
        seeds = [b"casino_state"],
        bump = casino_state.casino_state_bump
    )]
    pub casino_state: Account<'info, CasinoState>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = casino_state,
        associated_token::token_program = token_program,
    )]
    pub casino_vault: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init,
        payer = user,
        space = crate::utils::vrf::VrfGameState::DISCRIMINATOR.len() + crate::utils::vrf::VrfGameState::INIT_SPACE,
        seeds = [b"vrf_game_state", user.key().as_ref()],
        bump
    )]
    pub vrf_game_state: Account<'info, crate::utils::vrf::VrfGameState>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = UserStats::DISCRIMINATOR.len() + UserStats::INIT_SPACE,
        seeds = [b"user_stats", user.key().as_ref()],
        bump
    )]
    pub user_stats: Account<'info, UserStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: AccountInfo<'info>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// Settle slots game (Phase 2: Reveal randomness and determine outcome)
#[derive(Accounts)]
pub struct SettleSlotsGame<'info> {
    #[account(
        mut,
        seeds = [b"casino_state"],
        bump = casino_state.casino_state_bump
    )]
    pub casino_state: Account<'info, CasinoState>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = casino_state,
        associated_token::token_program = token_program,
    )]
    pub casino_vault: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"vrf_game_state", user.key().as_ref()],
        bump = vrf_game_state.bump
    )]
    pub vrf_game_state: Account<'info, crate::utils::vrf::VrfGameState>,

    #[account(
        mut,
        seeds = [b"user_stats", user.key().as_ref()],
        bump = user_stats.bump
    )]
    pub user_stats: Account<'info, UserStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: AccountInfo<'info>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl <'info> RequestSlotsGame<'info> {
    pub fn request_slots_game(&mut self, bet_amount: u64, randomness_account: Pubkey, bumps: &RequestSlotsGameBumps) -> Result<()> {
        let casino_state = &mut self.casino_state;
        let vrf_game_state = &mut self.vrf_game_state;
        let user_stats = &mut self.user_stats;
        
        // Validate casino is active
        require!(casino_state.is_active, CasinoError::CasinoNotActive);
        
        // Validate bet amount
        validate_bet_amount(bet_amount, 1000, 1000000)?; // Min 1 USDC, Max 1000 USDC
        
        // Generate deterministic seed for test RNG
        let clock = Clock::get()?;
        let user_key = self.user.key();
        let user_bytes = user_key.to_bytes();
        let mut user_prefix = [0u8; 8];
        user_prefix.copy_from_slice(&user_bytes[..8]);
        let seed = clock.slot ^ u64::from_le_bytes(user_prefix) ^ bet_amount;
        
        // Transfer bet amount from user to casino vault (take collateral on request)
        let transfer_instruction = TransferChecked {
            from: self.user_token_account.to_account_info(),
            to: self.casino_vault.to_account_info(),
            mint: self.usdc_mint.to_account_info(),
            authority: self.user.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_instruction,
        );
        
        transfer_checked(cpi_ctx, bet_amount, self.usdc_mint.decimals)?;
        
        // Initialize VRF game state
        vrf_game_state.set_inner(crate::utils::vrf::VrfGameState {
            user: self.user.key(),
            game_type: GameType::Slots,
            bet_amount,
            randomness_account,
            commit_slot: clock.slot,
            game_phase: crate::utils::vrf::GamePhase::Requested,
            game_data: seed.to_le_bytes().to_vec(), // Store seed for settlement
            payout: 0,
            is_complete: false,
            bump: bumps.vrf_game_state,
        });
        
        // Update user stats
        user_stats.user = self.user.key();
        user_stats.total_bets += bet_amount;
        user_stats.games_played += 1;
        user_stats.bump = bumps.user_stats;
        
        // Update casino stats
        casino_state.total_games_played += 1;
        casino_state.total_volume += bet_amount;
        
        msg!("Slots game requested: Bet: {}, Randomness Account: {}", bet_amount, randomness_account);
        Ok(())
    }
}

impl <'info> SettleSlotsGame<'info> {
    pub fn settle_slots_game(&mut self, bumps: &SettleSlotsGameBumps) -> Result<()> {
        let casino_state = &mut self.casino_state;
        let vrf_game_state = &mut self.vrf_game_state;
        let user_stats = &mut self.user_stats;
        let _ = bumps;
        
        // Validate game state
        require!(
            vrf_game_state.game_phase == crate::utils::vrf::GamePhase::Requested,
            CasinoError::InvalidGameState
        );
        
        // require!(
        //     vrf_game_state.randomness_account == self.randomness_account_data.key(),
        //     CasinoError::VrfRequestFailed
        // );
        
        // // // Parse and validate Switchboard randomness data
        // let randomness_data = crate::utils::vrf::parse_randomness_data(&self.randomness_account_data)?;
        
        // // Validate randomness timing
        // crate::utils::vrf::validate_randomness_timing(&randomness_data, vrf_game_state.commit_slot)?;
        
        // // Get revealed randomness
        // let randomness_bytes = crate::utils::vrf::get_revealed_randomness(&randomness_data)?;

        // msg!("Randomness bytes: {:?}", randomness_bytes);
        
        // Generate reels using test RNG seed stored in game_data
        require!(vrf_game_state.game_data.len() >= 8, CasinoError::InvalidGameState);
        let mut seed_bytes = [0u8; 8];
        seed_bytes.copy_from_slice(&vrf_game_state.game_data[..8]);
        let mut seed = u64::from_le_bytes(seed_bytes);
        let mut reels = [0u8; 3];
        for reel in reels.iter_mut() {
            seed = seed.wrapping_add(1);
            *reel = generate_random_number(seed, 10) as u8;
        }
        
        // Calculate payout
        let payout = calculate_slots_payout(reels, vrf_game_state.bet_amount);
        
        // Apply house edge
        let rtp_multiplier = casino_state.house_edge_config.slots_rtp_bps as f64 / 10000.0;
        let final_payout = (payout as f64 * rtp_multiplier) as u64;
        
        // Update VRF game state
        vrf_game_state.game_data = reels.to_vec();
        vrf_game_state.payout = final_payout;
        vrf_game_state.game_phase = crate::utils::vrf::GamePhase::Settled;
        vrf_game_state.is_complete = true;
        
        // Update user stats
        if final_payout > 0 {
            user_stats.total_wins += final_payout;
            user_stats.loyalty_points += vrf_game_state.bet_amount / 100;
        } else {
            user_stats.total_losses += vrf_game_state.bet_amount;
        }
        
        // Update casino stats
        casino_state.total_payouts += final_payout;
        
        // Transfer payout to user if they won
        if final_payout > 0 {
            let payout_instruction = TransferChecked {
                from: self.casino_vault.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: casino_state.to_account_info(),
                mint: self.usdc_mint.to_account_info(),
            };
            
            let seeds : &[&[&[u8]]] = &[&[b"casino_state", &[self.casino_state.casino_state_bump]]];
            let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), payout_instruction, seeds);
            transfer_checked(cpi_ctx, final_payout, self.usdc_mint.decimals)?;
        }
        Ok(())
    }
}

// Request roulette game (Phase 1: Request randomness)
#[derive(Accounts)]
pub struct RequestRouletteGame<'info> {
    #[account(
        mut,
        seeds = [b"casino_state"],
        bump = casino_state.casino_state_bump
    )]
    pub casino_state: Account<'info, CasinoState>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = casino_state,
        associated_token::token_program = token_program,
    )]
    pub casino_vault: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init,
        payer = user,
        space = crate::utils::vrf::VrfGameState::DISCRIMINATOR.len() + crate::utils::vrf::VrfGameState::INIT_SPACE,
        seeds = [b"vrf_game_state", user.key().as_ref()],
        bump
    )]
    pub vrf_game_state: Account<'info, crate::utils::vrf::VrfGameState>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = UserStats::DISCRIMINATOR.len() + UserStats::INIT_SPACE,
        seeds = [b"user_stats", user.key().as_ref()],
        bump
    )]
    pub user_stats: Account<'info, UserStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: AccountInfo<'info>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Settle roulette game (Phase 2: Reveal randomness and determine outcome)
#[derive(Accounts)]
pub struct SettleRouletteGame<'info> {
    #[account(
        mut,
        seeds = [b"casino_state"],
        bump = casino_state.casino_state_bump
    )]
    pub casino_state: Account<'info, CasinoState>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = casino_state,
        associated_token::token_program = token_program,
    )]
    pub casino_vault: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"vrf_game_state", user.key().as_ref()],
        bump = vrf_game_state.bump
    )]
    pub vrf_game_state: Account<'info, crate::utils::vrf::VrfGameState>,

    #[account(
        mut,
        seeds = [b"user_stats", user.key().as_ref()],
        bump = user_stats.bump
    )]
    pub user_stats: Account<'info, UserStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub usdc_mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: AccountInfo<'info>,
    
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl <'info> RequestRouletteGame<'info> {
    pub fn request_roulette_game(
        &mut self, 
        bet_amount: u64, 
        bet_type: RouletteBetType, 
        numbers: Vec<u8>,
        randomness_account: Pubkey,
        bumps: &RequestRouletteGameBumps
    ) -> Result<()> {
        let casino_state = &mut self.casino_state;
        let vrf_game_state = &mut self.vrf_game_state;
        let user_stats = &mut self.user_stats;
        
        // Validate casino is active
        require!(casino_state.is_active, CasinoError::CasinoNotActive);
        
        // Validate bet amount
        validate_bet_amount(bet_amount, 1000, 1000000)?;
        
        // Validate bet numbers based on bet type
        match bet_type {
            RouletteBetType::Straight => {
                require!(numbers.len() == 1, CasinoError::InvalidRouletteNumbers);
                require!(numbers[0] <= 36, CasinoError::InvalidRouletteNumbers);
            },
            RouletteBetType::Split => {
                require!(numbers.len() == 2, CasinoError::InvalidRouletteNumbers);
            },
            RouletteBetType::Street => {
                require!(numbers.len() == 3, CasinoError::InvalidRouletteNumbers);
            },
            RouletteBetType::Corner => {
                require!(numbers.len() == 4, CasinoError::InvalidRouletteNumbers);
            },
            RouletteBetType::Line => {
                require!(numbers.len() == 6, CasinoError::InvalidRouletteNumbers);
            },
            RouletteBetType::Column => {
                require!(numbers.len() == 12, CasinoError::InvalidRouletteNumbers);
            },
            RouletteBetType::Dozen => {
                require!(numbers.len() == 12, CasinoError::InvalidRouletteNumbers);
            },
            _ => {
                require!(numbers.is_empty(), CasinoError::InvalidRouletteNumbers);
            },
        }
        
        // // Parse and validate Switchboard randomness data
        // let randomness_data = crate::utils::vrf::parse_randomness_data(&self.randomness_account_data)?;
        let clock = Clock::get()?;
        
        // Transfer bet amount from user to casino vault
        let transfer_instruction = TransferChecked {
            from: self.user_token_account.to_account_info(),
            to: self.casino_vault.to_account_info(),
            authority: self.user.to_account_info(),
            mint: self.usdc_mint.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_instruction,
        );
        
        transfer_checked(cpi_ctx, bet_amount, self.usdc_mint.decimals)?;
        
        // Store bet information in game data
        let mut game_data = Vec::new();
        game_data.push(bet_type.clone() as u8);
        game_data.extend_from_slice(&numbers);
        
        // Generate deterministic seed for test RNG
        let user_key = self.user.key();
        let user_bytes = user_key.to_bytes();
        let mut user_prefix = [0u8; 8];
        user_prefix.copy_from_slice(&user_bytes[..8]);
        let mut numbers_hash = 0u64;
        for (i, number) in numbers.iter().enumerate() {
            numbers_hash ^= (*number as u64) << ((i % 8) * 8);
        }
        let seed = clock.slot ^ u64::from_le_bytes(user_prefix) ^ bet_amount ^ numbers_hash;
        game_data.extend_from_slice(&seed.to_le_bytes());
        
        // Initialize VRF game state
        vrf_game_state.set_inner(crate::utils::vrf::VrfGameState {
            user: self.user.key(),
            game_type: GameType::Roulette,
            bet_amount,
            randomness_account,
            commit_slot: clock.slot,
            game_phase: crate::utils::vrf::GamePhase::Requested,
            game_data,
            payout: 0,
            is_complete: false,
            bump: bumps.vrf_game_state,
        });
        
        // Update user stats
        user_stats.user = self.user.key();
        user_stats.total_bets += bet_amount;
        user_stats.games_played += 1;
        user_stats.bump = bumps.user_stats;
        
        // Update casino stats
        casino_state.total_games_played += 1;
        casino_state.total_volume += bet_amount;
        
        msg!("Roulette game requested successfully");
        Ok(())
    }
}

impl <'info> SettleRouletteGame<'info> {
    pub fn settle_roulette_game(&mut self, bumps: &SettleRouletteGameBumps) -> Result<()> {
        let casino_state = &mut self.casino_state;
        let vrf_game_state = &mut self.vrf_game_state;
        let user_stats = &mut self.user_stats;
        let _ = bumps;
        
        // Validate game state
        require!(
            vrf_game_state.game_phase == crate::utils::vrf::GamePhase::Requested,
            CasinoError::InvalidGameState
        );
        
        // require!(
        //     vrf_game_state.randomness_account == self.randomness_account_data.key(),
        //     CasinoError::VrfRequestFailed
        // );
        
        // // Parse and validate Switchboard randomness data
        // let randomness_data = crate::utils::vrf::parse_randomness_data(&self.randomness_account_data)?;
        
        // // Validate randomness timing
        // crate::utils::vrf::validate_randomness_timing(&randomness_data, vrf_game_state.commit_slot)?;
        
        // // Get revealed randomness
        // let randomness_bytes = crate::utils::vrf::get_revealed_randomness(&randomness_data)?;
        
        // Generate game-specific randomness
        require!(vrf_game_state.game_data.len() >= 9, CasinoError::InvalidGameState);
        let seed_index = vrf_game_state
            .game_data
            .len()
            .checked_sub(8)
            .ok_or(CasinoError::InvalidGameState)?;
        let _bet_numbers = vrf_game_state.game_data[1..seed_index].to_vec();
        let mut seed_bytes = [0u8; 8];
        seed_bytes.copy_from_slice(&vrf_game_state.game_data[seed_index..]);
        let seed = u64::from_le_bytes(seed_bytes);
        let winning_number = generate_random_number(seed, 37) as u8;
        
        // Parse stored bet information
        let bet_type_byte = vrf_game_state.game_data[0];
        let bet_type = match bet_type_byte {
            0 => RouletteBetType::Straight,
            1 => RouletteBetType::Split,
            2 => RouletteBetType::Street,
            3 => RouletteBetType::Corner,
            4 => RouletteBetType::Line,
            5 => RouletteBetType::Column,
            6 => RouletteBetType::Dozen,
            7 => RouletteBetType::Red,
            8 => RouletteBetType::Black,
            9 => RouletteBetType::Even,
            10 => RouletteBetType::Odd,
            11 => RouletteBetType::Low,
            12 => RouletteBetType::High,
            _ => return Err(CasinoError::InvalidRouletteBet.into()),
        };
        // let bet_numbers = vrf_game_state.game_data[1..].to_vec();
        
        // Calculate payout
        let payout = calculate_roulette_payout(bet_type.clone(), vrf_game_state.bet_amount, winning_number);
        
        // Apply house edge
        let rtp_multiplier = casino_state.house_edge_config.roulette_rtp_bps as f64 / 10000.0;
        let final_payout = (payout as f64 * rtp_multiplier) as u64;
        
        // Update VRF game state
        vrf_game_state.game_data = vec![winning_number];
        vrf_game_state.payout = final_payout;
        vrf_game_state.game_phase = crate::utils::vrf::GamePhase::Settled;
        vrf_game_state.is_complete = true;
        
        // Update user stats
        if final_payout > 0 {
            user_stats.total_wins += final_payout;
            user_stats.loyalty_points += vrf_game_state.bet_amount / 100;
        } else {
            user_stats.total_losses += vrf_game_state.bet_amount;
        }
        
        // Update casino stats
        casino_state.total_payouts += final_payout;
        
        // Transfer payout to user if they won
        if final_payout > 0 {
            let payout_instruction = TransferChecked {
                from: self.casino_vault.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: casino_state.to_account_info(),
                mint: self.usdc_mint.to_account_info(),
            };
            
            let seeds : &[&[&[u8]]] = &[&[b"casino_state", &[self.casino_state.casino_state_bump]]];
            let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), payout_instruction, seeds);
            
            transfer_checked(cpi_ctx, final_payout, self.usdc_mint.decimals)?;
        }
        
        msg!("Roulette game settled: Winning Number: {}, Payout: {}", winning_number, final_payout);
        Ok(())
    }
}
