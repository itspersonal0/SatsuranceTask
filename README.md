# ICP Staking Pool Canister

A decentralized staking pool canister built on the Internet Computer Protocol (ICP) using Rust SDK. This canister allows users to stake ICP tokens with different lock periods and earn proportional rewards.

## Features

- **Multiple Lock Periods**: Users can stake for 90, 180, or 360 days
- **Multiple Independent Deposits**: Users can make multiple stakes with different lock times
- **Proportional Rewards**: Rewards are distributed proportionally based on stake amounts
- **Slashing Mechanism**: Pool can be slashed proportionally with funds sent to specified receiver
- **Time-locked Withdrawals**: Funds can only be withdrawn after the lock period expires

## Prerequisites

- [DFX SDK](https://internetcomputer.org/docs/current/developer-docs/setup/install/) (version 0.15.0 or later)
- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (for frontend development, optional)

## Installation

1. **Clone the repository:**
```bash
git clone <your-repo-url>
cd staking_pool
```

2. **Install dependencies:**
```bash
# Install Rust target for WebAssembly
rustup target add wasm32-unknown-unknown
```

## Quick Start

### 1. Start the local replica
```bash
dfx start --clean --background
```

### 2. Deploy the canister
```bash
dfx deploy
```

### 3. Interact with the canister

**Make a deposit (stake 1000 ICP for 90 days):**
```bash
dfx canister call staking_pool_backend deposit '(record { amount = 1000; lock_period_days = 90 })'
```

**Check your stakes:**
```bash
dfx canister call staking_pool_backend get_my_stakes
```

**Check pool information:**
```bash
dfx canister call staking_pool_backend get_pool_info
```

## API Reference

### Update Methods

#### `deposit(request: DepositRequest) -> Result<String, String>`
Allows users to deposit funds into the staking pool.

**Parameters:**
- `amount`: Amount to stake (in ICP)
- `lock_period_days`: Lock period (90, 180, or 360 days)

**Example:**
```bash
dfx canister call staking_pool_backend deposit '(record { amount = 1000; lock_period_days = 180 })'
```

#### `withdraw(request: WithdrawRequest) -> Result<String, String>`
Allows users to withdraw funds after the lock period expires.

**Parameters:**
- `stake_index`: Index of the stake to withdraw

**Example:**
```bash
dfx canister call staking_pool_backend withdraw '(record { stake_index = 0 })'
```

#### `reward_pool(amount: u64) -> Result<String, String>`
Distributes rewards proportionally to all stakers.

**Parameters:**
- `amount`: Total reward amount to distribute

**Example:**
```bash
dfx canister call staking_pool_backend reward_pool '(500)'
```

#### `slash_pool(amount: u64, receiver: Principal) -> Result<String, String>`
Slashes the pool proportionally and sends funds to a receiver.

**Parameters:**
- `amount`: Amount to slash from the pool
- `receiver`: Principal to receive the slashed funds

**Example:**
```bash
dfx canister call staking_pool_backend slash_pool '(200, principal "rdmx6-jaaaa-aaaah-qcaiq-cai")'
```

### Query Methods

#### `get_my_stakes() -> Option<UserStakes>`
Returns the caller's stake information.

```bash
dfx canister call staking_pool_backend get_my_stakes
```

#### `get_user_stakes(user: Principal) -> Option<UserStakes>`
Returns stake information for a specific user.

```bash
dfx canister call staking_pool_backend get_user_stakes '(principal "rdmx6-jaaaa-aaaah-qcaiq-cai")'
```

#### `get_pool_info() -> (u64, usize)`
Returns total pool amount and number of stakers.

```bash
dfx canister call staking_pool_backend get_pool_info
```

#### `get_current_timestamp() -> u64`
Returns the current timestamp in seconds.

```bash
dfx canister call staking_pool_backend get_current_timestamp
```

## Data Structures

### StakeInfo
```rust
pub struct StakeInfo {
    pub amount: u64,           // Staked amount
    pub lock_period_days: u32, // Lock period in days
    pub stake_time: u64,       // Timestamp when staked
    pub unlock_time: u64,      // Timestamp when unlocked
}
```

### UserStakes
```rust
pub struct UserStakes {
    pub stakes: Vec<StakeInfo>, // List of user's stakes
    pub total_staked: u64,      // Total amount staked by user
}
```

## Testing

### Manual Testing Examples

1. **Test Multiple Deposits:**
```bash
# First deposit
dfx canister call staking_pool_backend deposit '(record { amount = 1000; lock_period_days = 90 })'

# Second deposit with different lock period
dfx canister call staking_pool_backend deposit '(record { amount = 2000; lock_period_days = 180 })'

# Check stakes
dfx canister call staking_pool_backend get_my_stakes
```

2. **Test Reward Distribution:**
```bash
# Add rewards
dfx canister call staking_pool_backend reward_pool '(300)'

# Check updated stakes
dfx canister call staking_pool_backend get_my_stakes
```

3. **Test Pool Slashing:**
```bash
# Slash pool
dfx canister call staking_pool_backend slash_pool '(100, principal "rdmx6-jaaaa-aaaah-qcaiq-cai")'

# Check updated stakes
dfx canister call staking_pool_backend get_my_stakes
```

4. **Test Withdrawal (after lock period):**
```bash
# This will fail if lock period hasn't passed
dfx canister call staking_pool_backend withdraw '(record { stake_index = 0 })'
```

## Candid UI

After deployment, access the Candid UI at:
```
http://127.0.0.1:4943/?canisterId=<CANDID_UI_ID>&id=<CANISTER_ID>
```

The URL will be displayed after successful deployment.

## Project Structure

```
staking_pool/
├── src/
│   └── staking_pool_backend/
│       ├── src/
│       │   └── lib.rs                 # Main canister logic
│       ├── Cargo.toml                 # Rust dependencies
│       └── staking_pool_backend.did   # Candid interface
├── dfx.json                           # DFX configuration
├── Cargo.toml                         # Workspace configuration
└── README.md                          # This file
```

## Error Handling

The canister includes comprehensive error handling for:
- Invalid lock periods
- Zero amounts
- Invalid stake indices
- Premature withdrawals
- Empty pools
- Insufficient funds for slashing

## Security Considerations

- All state modifications are protected by caller authentication
- Time-based locks prevent premature withdrawals
- Proportional calculations prevent unfair distribution
- Input validation prevents invalid operations

## Development

### Building
```bash
dfx build
```

### Deploying to Mainnet
```bash
dfx deploy --network ic
```

### Stopping Local Replica
```bash
dfx stop
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Support

For issues and questions:
- Create an issue in the repository
- Check the [Internet Computer documentation](https://internetcomputer.org/docs/)
- Visit the [DFINITY Developer Forum](https://forum.dfinity.org/)
