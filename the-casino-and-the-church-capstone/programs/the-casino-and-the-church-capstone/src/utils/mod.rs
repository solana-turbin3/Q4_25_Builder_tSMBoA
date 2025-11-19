use anchor_lang::prelude::*;
use crate::state::casino::*;

pub mod math;
pub mod vrf;

pub use math::*;
pub use vrf::*;

// Utility functions for casino games
pub fn calculate_slots_payout(reels: [u8; 3], bet_amount: u64) -> u64 {
    let mut payout = 0u64;
    
    // Check each payline
    for i in 0..3 {
        let symbol = reels[i];
        if symbol < 10 {
            // Check for 3-of-a-kind
            if reels[0] == symbol && reels[1] == symbol && reels[2] == symbol {
                payout += bet_amount * SLOTS_PAYOUTS[symbol as usize][2] / 100;
            }
            // Check for 2-of-a-kind
            else if (reels[0] == symbol && reels[1] == symbol) || 
                    (reels[1] == symbol && reels[2] == symbol) ||
                    (reels[0] == symbol && reels[2] == symbol) {
                payout += bet_amount * SLOTS_PAYOUTS[symbol as usize][1] / 100;
            }
            // Check for single symbol
            else if reels[0] == symbol || reels[1] == symbol || reels[2] == symbol {
                payout += bet_amount * SLOTS_PAYOUTS[symbol as usize][0] / 100;
            }
        }
    }
    
    payout
}

pub fn calculate_roulette_payout(bet_type: RouletteBetType, bet_amount: u64, winning_number: u8) -> u64 {
    match bet_type {
        RouletteBetType::Straight => {
            // 35:1 payout
            bet_amount * 35
        },
        RouletteBetType::Split => {
            // 17:1 payout
            bet_amount * 17
        },
        RouletteBetType::Street => {
            // 11:1 payout
            bet_amount * 11
        },
        RouletteBetType::Corner => {
            // 8:1 payout
            bet_amount * 8
        },
        RouletteBetType::Line => {
            // 5:1 payout
            bet_amount * 5
        },
        RouletteBetType::Column => {
            // 2:1 payout
            bet_amount * 2
        },
        RouletteBetType::Dozen => {
            // 2:1 payout
            bet_amount * 2
        },
        RouletteBetType::Red => {
            if ROULETTE_RED_NUMBERS.contains(&winning_number) {
                bet_amount
            } else {
                0
            }
        },
        RouletteBetType::Black => {
            if ROULETTE_BLACK_NUMBERS.contains(&winning_number) {
                bet_amount
            } else {
                0
            }
        },
        RouletteBetType::Even => {
            if winning_number != 0 && winning_number % 2 == 0 {
                bet_amount
            } else {
                0
            }
        },
        RouletteBetType::Odd => {
            if winning_number % 2 == 1 {
                bet_amount
            } else {
                0
            }
        },
        RouletteBetType::Low => {
            if winning_number >= 1 && winning_number <= 18 {
                bet_amount
            } else {
                0
            }
        },
        RouletteBetType::High => {
            if winning_number >= 19 && winning_number <= 36 {
                bet_amount
            } else {
                0
            }
        },
    }
}

pub fn calculate_aviator_payout(cashout_multiplier: f64, crash_multiplier: f64, bet_amount: u64) -> u64 {
    if cashout_multiplier <= crash_multiplier {
        // Player cashed out before crash
        (bet_amount as f64 * cashout_multiplier) as u64
    } else {
        // Player didn't cash out in time
        0
    }
}

pub fn calculate_blackjack_payout(player_hand: &Vec<u8>, dealer_hand: &Vec<u8>, bet_amount: u64) -> u64 {
    let player_value = calculate_hand_value(player_hand);
    let dealer_value = calculate_hand_value(dealer_hand);
    
    if player_value > BLACKJACK_VALUE {
        // Player busted
        0
    } else if dealer_value > BLACKJACK_VALUE {
        // Dealer busted, player wins
        bet_amount
    } else if player_value > dealer_value {
        // Player wins
        bet_amount
    } else if player_value == dealer_value {
        // Push
        bet_amount
    } else {
        // Dealer wins
        0
    }
}

pub fn calculate_hand_value(hand: &Vec<u8>) -> u8 {
    let mut value = 0u8;
    let mut aces = 0u8;
    
    for card in hand {
        match card {
            1 => {
                aces += 1;
                value += 11;
            },
            2..=10 => value += card,
            11..=13 => value += 10,
            _ => {}
        }
    }
    
    // Adjust for aces
    while value > BLACKJACK_VALUE && aces > 0 {
        value -= 10;
        aces -= 1;
    }
    
    value
}

pub fn is_blackjack(hand: &Vec<u8>) -> bool {
    hand.len() == 2 && calculate_hand_value(hand) == BLACKJACK_VALUE
}

pub fn should_dealer_hit(dealer_hand: &Vec<u8>) -> bool {
    let value = calculate_hand_value(dealer_hand);
    value < DEALER_STAND_VALUE
}
