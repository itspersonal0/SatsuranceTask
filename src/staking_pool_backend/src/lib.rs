use candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::*;  // Keep this for interaction with the ICP environment
use std::collections::HashMap;
use std::cell::RefCell;
use sha2::{Digest, Sha256};  // For creating subaccounts with SHA256

type Subaccount = [u8; 32];  // Defining a type for Subaccount
type AccountIdentifier = String;

const ICP_FEE: u64 = 10_000;  // Minimum fee for depositing

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StakeInfo {
    pub amount: u64,
    pub lock_period_days: u32,
    pub stake_time: u64,
    pub unlock_time: u64,
    pub subaccount: Subaccount,
    pub account_id: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UserStakes {
    pub stakes: Vec<StakeInfo>,
    pub total_staked: u64,
}

#[derive(CandidType, Deserialize)]
pub struct DepositRequest {
    pub amount: u64,
    pub lock_period_days: u32,
}

#[derive(CandidType, Deserialize)]
pub struct WithdrawRequest {
    pub stake_index: usize,
}

thread_local! {
    static STAKES: RefCell<HashMap<Principal, UserStakes>> = RefCell::new(HashMap::new());
    static TOTAL_POOL_AMOUNT: RefCell<u64> = RefCell::new(0);
    static NEXT_SUBACCOUNT_NONCE: RefCell<u64> = RefCell::new(1);
    static AUTHORIZED_PRINCIPALS: RefCell<Vec<Principal>> = RefCell::new(Vec::new());
    static CANISTER_BALANCE: RefCell<u64> = RefCell::new(1_000_000_000_000); // Example balance for testing
}

#[init]
fn init() {
    let caller = ic_cdk::caller();
    AUTHORIZED_PRINCIPALS.with(|auth| {
        auth.borrow_mut().push(caller);
    });
}

fn get_current_time() -> u64 {
    ic_cdk::api::time() / 1_000_000_000  // Convert from nanoseconds to seconds
}

fn is_authorized(caller: &Principal) -> bool {
    AUTHORIZED_PRINCIPALS.with(|auth| {
        auth.borrow().contains(caller)
    })
}

fn generate_subaccount(caller: &Principal) -> Subaccount {
    let nonce = NEXT_SUBACCOUNT_NONCE.with(|n| {
        let current = *n.borrow();
        *n.borrow_mut() = current + 1;
        current
    });
    
    let mut hasher = Sha256::new();
    hasher.update(caller.as_slice());
    hasher.update(nonce.to_be_bytes());
    hasher.update(b"staking_pool");  // Add a unique identifier to the hash
    
    let hash = hasher.finalize();
    let mut subaccount = [0u8; 32];
    subaccount.copy_from_slice(&hash[..32]);  // Only take the first 32 bytes
    
    subaccount
}

fn get_account_identifier(subaccount: &Subaccount) -> AccountIdentifier {
    format!("account_{}", hex::encode(subaccount))  // Convert subaccount to string
}

#[update]
async fn deposit(request: DepositRequest) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Validate lock period
    if ![90, 180, 360].contains(&request.lock_period_days) {
        return Err("Invalid lock period. Must be 90, 180, or 360 days".to_string());
    }
    
    // Validate amount
    if request.amount < ICP_FEE {
        return Err(format!("Amount must be at least {} e8s to cover fees", ICP_FEE));
    }
    
    // Check canister balance (simulated)
    let available_balance = CANISTER_BALANCE.with(|balance| *balance.borrow());
    if request.amount > available_balance {
        return Err("Insufficient canister balance for deposit".to_string());
    }
    
    // Generate unique subaccount for this stake
    let stake_subaccount = generate_subaccount(&caller);
    let account_id = get_account_identifier(&stake_subaccount);
    
    // Simulate transfer (in real implementation, this would be actual ICP transfer)
    CANISTER_BALANCE.with(|balance| {
        *balance.borrow_mut() -= request.amount;
    });
    
    let current_time = get_current_time();
    let unlock_time = current_time + (request.lock_period_days as u64 * 24 * 60 * 60);
    
    let stake_info = StakeInfo {
        amount: request.amount,
        lock_period_days: request.lock_period_days,
        stake_time: current_time,
        unlock_time,
        subaccount: stake_subaccount,
        account_id: account_id.clone(),
    };
    
    // Update state
    STAKES.with(|stakes| {
        let mut stakes_map = stakes.borrow_mut();
        let user_stakes = stakes_map.entry(caller).or_insert(UserStakes {
            stakes: Vec::new(),
            total_staked: 0,
        });
        
        user_stakes.stakes.push(stake_info);
        user_stakes.total_staked += request.amount;
    });
    
    TOTAL_POOL_AMOUNT.with(|total| {
        *total.borrow_mut() += request.amount;
    });
    
    Ok(format!(
        "Successfully deposited {} e8s for {} days. Account: {}", 
        request.amount, request.lock_period_days, account_id
    ))
}

#[update]
async fn withdraw(request: WithdrawRequest) -> Result<String, String> {
    let caller = ic_cdk::caller();
    let current_time = get_current_time();
    
    let (amount, _subaccount) = STAKES.with(|stakes| {
        let mut stakes_map = stakes.borrow_mut();
        
        match stakes_map.get_mut(&caller) {
            Some(user_stakes) => {
                if request.stake_index >= user_stakes.stakes.len() {
                    return Err("Invalid stake index".to_string());
                }
                
                let stake = &user_stakes.stakes[request.stake_index];
                
                if current_time < stake.unlock_time {
                    let remaining_time = stake.unlock_time - current_time;
                    return Err(format!(
                        "Stake is still locked. Remaining time: {} seconds", 
                        remaining_time
                    ));
                }
                
                let amount = stake.amount;
                let subaccount = stake.subaccount;
                
                // Remove stake and update totals
                user_stakes.stakes.remove(request.stake_index);
                user_stakes.total_staked -= amount;
                
                TOTAL_POOL_AMOUNT.with(|total| {
                    *total.borrow_mut() -= amount;
                });
                
                Ok((amount, subaccount))
            }
            None => Err("No stakes found for user".to_string()),
        }
    })?;
    
    let transfer_amount = amount.saturating_sub(ICP_FEE);
    
    if transfer_amount == 0 {
        return Err("Insufficient amount to cover transfer fee".to_string());
    }
    
    CANISTER_BALANCE.with(|balance| {
        *balance.borrow_mut() += transfer_amount;
    });
    
    Ok(format!(
        "Successfully withdrew {} e8s (fee: {} e8s)", 
        transfer_amount, ICP_FEE
    ))
}

#[query]
fn get_user_stakes(user: Principal) -> Option<UserStakes> {
    STAKES.with(|stakes| {
        stakes.borrow().get(&user).cloned()
    })
}

#[query]
fn get_my_stakes() -> Option<UserStakes> {
    let caller = ic_cdk::caller();
    get_user_stakes(caller)
}

#[query]
fn get_pool_info() -> (u64, usize, usize) {
    let total_amount = TOTAL_POOL_AMOUNT.with(|total| *total.borrow());
    let total_stakers = STAKES.with(|stakes| stakes.borrow().len());
    let total_stakes = STAKES.with(|stakes| {
        stakes.borrow().values().map(|user_stakes| user_stakes.stakes.len()).sum()
    });
    (total_amount, total_stakers, total_stakes)
}

// Export Candid interface (remove ic_ledger_types and export_candid)
