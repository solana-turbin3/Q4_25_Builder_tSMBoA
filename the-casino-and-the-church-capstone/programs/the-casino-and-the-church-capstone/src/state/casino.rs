use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum GameType {
    Slots,
    Roulette,
    Aviator,
    Blackjack,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum RouletteBetType {
    Straight,    // Single number
    Split,       // Two adjacent numbers
    Street,      // Three numbers in a row
    Corner,      // Four numbers forming a square
    Line,        // Six numbers in two adjacent rows
    Column,      // 12 numbers in a column
    Dozen,       // 12 numbers (1-12, 13-24, 25-36)
    Red,         // Red numbers
    Black,       // Black numbers
    Even,        // Even numbers
    Odd,         // Odd numbers
    Low,         // 1-18
    High,        // 19-36
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BlackjackAction {
    Hit,
    Stand,
    DoubleDown,
    Split,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum BlackjackHandStatus {
    Playing,
    Busted,
    Blackjack,
    Stand,
}

#[account]
#[derive(InitSpace)]
pub struct CasinoState {
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub total_games_played: u64,
    pub total_volume: u64,
    pub total_payouts: u64,
    pub house_edge_config: HouseEdgeConfig,
    pub is_active: bool,
    pub casino_state_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct HouseEdgeConfig {
    pub slots_rtp_bps: u16,      // 9500 = 95% RTP (5% house edge)
    pub roulette_rtp_bps: u16,   // 9730 = 97.3% RTP (2.7% house edge)
    pub aviator_rtp_bps: u16,    // 9600 = 96% RTP (4% house edge)
    pub blackjack_rtp_bps: u16,  // 9950 = 99.5% RTP (0.5% house edge)
    pub platform_fee_bps: u16,  // 200 = 2% platform fee
}

#[account]
#[derive(InitSpace)]
pub struct GameState {
    pub game_type: GameType,
    pub total_bets: u64,
    pub total_payouts: u64,
    pub total_games: u64,
    pub rtp_bps: u16,
    pub min_bet: u64,
    pub max_bet: u64,
    pub is_active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserStats {
    pub user: Pubkey,
    pub total_bets: u64,
    pub total_wins: u64,
    pub total_losses: u64,
    pub loyalty_points: u64,
    pub games_played: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct SlotsGame {
    pub user: Pubkey,
    pub bet_amount: u64,
    pub reels: [u8; 3], // 0-9 for each reel
    pub paylines: [bool; 5], // Which paylines are active
    pub payout: u64,
    pub is_complete: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct RouletteGame {
    pub user: Pubkey,
    pub bet_amount: u64,
    pub bet_type: RouletteBetType,
    #[max_len(37)]
    pub bet_numbers: Vec<u8>,
    pub winning_number: u8,
    pub payout: u64,
    pub is_complete: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct AviatorGame {
    pub user: Pubkey,
    pub bet_amount: u64,
    pub cashout_multiplier: f64,
    pub crash_multiplier: f64,
    pub payout: u64,
    pub is_complete: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct BlackjackGame {
    pub user: Pubkey,
    pub bet_amount: u64,
    #[max_len(13)]
    pub player_hand: Vec<u8>, // Card values (1-13)
    #[max_len(13)]
    pub dealer_hand: Vec<u8>,
    pub player_status: BlackjackHandStatus,
    pub dealer_status: BlackjackHandStatus,
    pub payout: u64,
    pub is_complete: bool,
    pub bump: u8,
}

// Constants for game logic
pub const SLOTS_SYMBOLS: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; // 10 different symbols
pub const SLOTS_PAYOUTS: [[u64; 3]; 10] = [
    [0, 0, 0],     // Symbol 0: No payout
    [2, 5, 10],    // Symbol 1: 2x, 5x, 10x
    [3, 8, 15],    // Symbol 2: 3x, 8x, 15x
    [5, 12, 25],   // Symbol 3: 5x, 12x, 25x
    [8, 20, 50],   // Symbol 4: 8x, 20x, 50x
    [12, 30, 75],  // Symbol 5: 12x, 30x, 75x
    [20, 50, 100], // Symbol 6: 20x, 50x, 100x
    [30, 75, 200], // Symbol 7: 30x, 75x, 200x
    [50, 125, 500], // Symbol 8: 50x, 125x, 500x
    [100, 250, 1000], // Symbol 9: 100x, 250x, 1000x
];

pub const ROULETTE_NUMBERS: [u8; 37] = [
    0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26
];

pub const ROULETTE_RED_NUMBERS: [u8; 18] = [
    1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36
];

pub const ROULETTE_BLACK_NUMBERS: [u8; 18] = [
    2, 4, 6, 8, 10, 11, 13, 15, 17, 20, 22, 24, 26, 28, 29, 31, 33, 35
];

// Blackjack constants
pub const BLACKJACK_VALUE: u8 = 21;
pub const DEALER_STAND_VALUE: u8 = 17;
pub const BLACKJACK_PAYOUT: u64 = 150; // 150% for blackjack
