use anchor_lang::prelude::*;
use crate::state::casino::*;



// Mathematical utility functions for casino games
pub fn calculate_rtp(house_edge_bps: u16) -> u16 {
    10000 - house_edge_bps
}

pub fn calculate_house_edge(rtp_bps: u16) -> u16 {
    10000 - rtp_bps
}

pub fn calculate_platform_fee(amount: u64, fee_bps: u16) -> u64 {
    (amount * fee_bps as u64) / 10000
}

pub fn calculate_lp_fee_share(total_fees: u64, lp_share_bps: u16) -> u64 {
    (total_fees * lp_share_bps as u64) / 10000
}

pub fn calculate_staking_rewards(staked_amount: u64, apr_bps: u16, time_elapsed: i64) -> u64 {
    let seconds_per_year = 365 * 24 * 60 * 60;
    let time_elapsed_seconds = time_elapsed as u64;
    
    if time_elapsed_seconds >= seconds_per_year {
        (staked_amount * apr_bps as u64) / 10000
    } else {
        (staked_amount * apr_bps as u64 * time_elapsed_seconds) / (10000 * seconds_per_year)
    }
}

pub fn calculate_compound_interest(principal: u64, rate_bps: u16, periods: u32) -> u64 {
    let rate = rate_bps as f64 / 10000.0;
    let compound_factor = (1.0 + rate).powi(periods as i32);
    (principal as f64 * compound_factor) as u64
}

// Probability calculations for games
pub fn calculate_slots_probability(symbol: u8, count: u8) -> f64 {
    match count {
        1 => 0.3,  // 30% chance for single symbol
        2 => 0.1,  // 10% chance for pair
        3 => 0.01, // 1% chance for triple
        _ => 0.0,
    }
}

pub fn calculate_roulette_probability(bet_type: RouletteBetType) -> f64 {
    match bet_type {
        RouletteBetType::Straight => 1.0 / 37.0,
        RouletteBetType::Split => 2.0 / 37.0,
        RouletteBetType::Street => 3.0 / 37.0,
        RouletteBetType::Corner => 4.0 / 37.0,
        RouletteBetType::Line => 6.0 / 37.0,
        RouletteBetType::Column => 12.0 / 37.0,
        RouletteBetType::Dozen => 12.0 / 37.0,
        RouletteBetType::Red => 18.0 / 37.0,
        RouletteBetType::Black => 18.0 / 37.0,
        RouletteBetType::Even => 18.0 / 37.0,
        RouletteBetType::Odd => 18.0 / 37.0,
        RouletteBetType::Low => 18.0 / 37.0,
        RouletteBetType::High => 18.0 / 37.0,
    }
}

pub fn calculate_blackjack_probability(hand_value: u8) -> f64 {
    match hand_value {
        21 => 0.048,  // 4.8% chance of blackjack
        20 => 0.1,    // 10% chance of 20
        19 => 0.1,    // 10% chance of 19
        18 => 0.1,    // 10% chance of 18
        17 => 0.1,    // 10% chance of 17
        _ => 0.0,
    }
}

// VRF utility functions
pub fn generate_random_number(seed: u64, max: u64) -> u64 {
    // Simple PRNG for testing - in production, use Switchboard VRF
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x % max
}

pub fn generate_random_float(seed: u64) -> f64 {
    let random_int = generate_random_number(seed, 1000000);
    random_int as f64 / 1000000.0
}

// Validation functions
pub fn validate_bet_amount(amount: u64, min_bet: u64, max_bet: u64) -> Result<()> {
    require!(amount >= min_bet, crate::errors::CasinoError::BetAmountTooLow);
    require!(amount <= max_bet, crate::errors::CasinoError::BetAmountTooHigh);
    Ok(())
}

pub fn validate_rtp_config(rtp_bps: u16) -> Result<()> {
    require!(rtp_bps >= 8000, crate::errors::CasinoError::InvalidHouseEdgeConfig); // Min 80% RTP
    require!(rtp_bps <= 9950, crate::errors::CasinoError::InvalidHouseEdgeConfig); // Max 99.5% RTP
    Ok(())
}

pub fn validate_fee_config(fee_bps: u16) -> Result<()> {
    require!(fee_bps <= 1000, crate::errors::CasinoError::InvalidHouseEdgeConfig); // Max 10% fee
    Ok(())
}
