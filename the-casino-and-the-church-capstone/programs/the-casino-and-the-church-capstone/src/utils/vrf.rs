use anchor_lang::prelude::*;
use switchboard_on_demand::accounts::RandomnessAccountData;
use anchor_lang::solana_program::hash::hash;

// VRF integration with Switchboard On-Demand
// Following Switchboard's two-phase pattern: Request -> Settle

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum GamePhase {
    Requested,  // Randomness requested, waiting for reveal
    Settled,    // Game completed with revealed randomness
}

// VRF account for storing game state with randomness
#[account]
#[derive(InitSpace)]
pub struct VrfGameState {
    pub user: Pubkey,
    pub game_type: crate::state::casino::GameType,
    pub bet_amount: u64,
    pub randomness_account: Pubkey,  // Reference to Switchboard randomness account
    pub commit_slot: u64,            // Slot when randomness was committed
    pub game_phase: GamePhase,
    #[max_len(100)]
    pub game_data: Vec<u8>,          // Game-specific data (reels, cards, etc.)
    pub payout: u64,
    pub is_complete: bool,
    pub bump: u8,
}


pub fn generate_mock_randomness() -> [u8; 32] {
    // Combine current slot + some entropy from timestamp
    let clock = Clock::get().unwrap();
    let seed_bytes = clock.slot.to_le_bytes();

    // Hash it for pseudo-random output
    let hash_result = hash(&seed_bytes);
    hash_result.to_bytes()
}

// VRF utility functions for Switchboard integration

/// Parse and validate Switchboard randomness account data
pub fn parse_randomness_data(
    randomness_account_data: &AccountInfo,
) -> Result<RandomnessAccountData> {
    let data = randomness_account_data.data.borrow();
    let parsed = RandomnessAccountData::parse(data)
        .map_err(|_| crate::errors::CasinoError::VrfRequestFailed)?;
    Ok(parsed.clone())
}

/// Validate that randomness is ready to be revealed
pub fn validate_randomness_timing(
    randomness_data: &RandomnessAccountData,
    expected_commit_slot: u64,
) -> Result<()> {
    let clock = Clock::get()?;
    
    // Ensure randomness is from the expected slot
    require!(
        randomness_data.seed_slot == expected_commit_slot,
        crate::errors::CasinoError::VrfRequestFailed
    );
    
    // Ensure randomness is ready to be revealed (current slot > seed_slot)
    require!(
        clock.slot > randomness_data.seed_slot,
        crate::errors::CasinoError::VrfRequestFailed
    );
    
    Ok(())
}

/// Get revealed random value from Switchboard
pub fn get_revealed_randomness(
    randomness_data: &RandomnessAccountData,
) -> Result<Vec<u8>> {
    let clock = Clock::get()?;
    let random_bytes: [u8; 32] = randomness_data
        .get_value(clock.slot)
        .map_err(|_| crate::errors::CasinoError::VrfRequestFailed)?;
    Ok(random_bytes.to_vec())
}

/// Generate game-specific random values from Switchboard randomness
pub fn generate_game_randomness(
    game_type: crate::state::casino::GameType,
    randomness_bytes: &[u8],
) -> Result<Vec<u8>> {
    match game_type {
        crate::state::casino::GameType::Slots => {
            // Generate 3 random numbers for slots (0-9)
            let mut reels = Vec::new();
            for i in 0..3 {
                let random_byte = randomness_bytes[i % randomness_bytes.len()];
                reels.push(random_byte % 10);
            }
            Ok(reels)
        },
        crate::state::casino::GameType::Roulette => {
            // Generate random number 0-36 for roulette
            let random_byte = randomness_bytes[0];
            let number = random_byte % 37;
            Ok(vec![number])
        },
        crate::state::casino::GameType::Aviator => {
            // Generate crash multiplier (1.0x - 100.0x)
            let random_bytes: [u8; 4] = [
                randomness_bytes[0],
                randomness_bytes[1],
                randomness_bytes[2],
                randomness_bytes[3],
            ];
            let random_value = u32::from_le_bytes(random_bytes);
            let multiplier = 1.0 + (random_value % 9900) as f64 / 100.0;
            let multiplier_bytes = multiplier.to_le_bytes();
            Ok(multiplier_bytes.to_vec())
        },
        crate::state::casino::GameType::Blackjack => {
            // Generate multiple card values (1-13) for blackjack
            let mut cards = Vec::new();
            for i in 0..4 { // Generate 4 cards initially
                let random_byte = randomness_bytes[i % randomness_bytes.len()];
                cards.push((random_byte % 13) + 1);
            }
            Ok(cards)
        },
    }
}

/// Transfer funds with optional seeds for PDA authority
pub fn transfer_funds<'a>(
    system_program: AccountInfo<'a>,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    amount: u64,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let transfer_accounts = anchor_lang::system_program::Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
    };

    let transfer_ctx = match seeds {
        Some(seeds) => CpiContext::new_with_signer(system_program, transfer_accounts, seeds),
        None => CpiContext::new(system_program, transfer_accounts),
    };

    anchor_lang::system_program::transfer(transfer_ctx, amount)
}

// VRF configuration constants
pub const VRF_TIMEOUT_SLOTS: u64 = 150; // ~1 minute timeout for VRF requests
pub const MAX_VRF_REQUESTS: u32 = 1000; // Maximum concurrent VRF requests
