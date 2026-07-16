#![allow(unused)]
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VaultError {
    InvalidTier        = 1,
    BelowMinDeposit    = 2,
    LockNotExpired     = 3,
    DepositCapExceeded = 5,
}