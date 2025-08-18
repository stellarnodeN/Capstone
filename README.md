# RecruSearch

> **Decentralized Research Participation Platform on Solana**  
> A blockchain-based protocol enabling researchers to create studies and participants to earn rewards through NFT-gated research participation.

---

RecruSearch is a comprehensive decentralized research recruitment and data collection platform built on Solana using the Anchor framework. The platform enables researchers to create, manage, and execute research studies through smart contracts while providing participants with transparent enrollment processes, on-chain consent management, and automated reward distribution.

The core architecture features NFT-gated participation where participants receive Consent NFTs upon enrollment and Completion NFTs upon successful study completion. The system implements configurable eligibility criteria including age, gender, and location requirements with on-chain validation. Research data is securely stored on IPFS with cryptographic hashing, ensuring data integrity and privacy while maintaining decentralization.

Built with Rust smart contracts and comprehensive TypeScript testing, RecruSearch supports a complete study lifecycle from Draft to Published, Active, and Closed states with automated transitions. The platform integrates MPL Core for NFT operations and SPL Token for reward distribution. Security features include immutable consent records with revocation capabilities, role-based access control, and complete transaction audit trails for compliance and transparency.

RecruSearch serves academic research, clinical trials, market research, social science experiments, and decentralized research communities, providing a robust foundation for blockchain-based research participation with automated incentive structures and transparent governance.

## Core Functions

- **initialize_protocol()** - Sets up protocol parameters and admin state
- **create_study()** - Creates new research studies with configurable parameters
- **set_eligibility_criteria()** - Defines participant requirements and constraints
- **create_reward_vault()** - Establishes token vaults for study rewards
- **publish_study()** - Transitions studies from Draft to Published state
- **mint_consent_nft()** - Enrolls participants and mints consent NFTs
- **submit_data()** - Collects and stores research data on IPFS
- **mint_completion_nft()** - Rewards participants with completion NFTs
- **distribute_reward()** - Transfers tokens to eligible participants
- **close_study()** - Finalizes studies and archives data

## Architecture

The platform uses Program Derived Addresses (PDAs) for account management, MPL Core for NFT operations, and IPFS for decentralized data storage. Smart contracts handle all business logic including eligibility validation, consent management, and automated reward distribution.

## Getting Started

1. Install dependencies: `pnpm install`
2. Build the program: `anchor build`
3. Run tests: `anchor test`
4. Deploy to localnet: `anchor deploy`

## User Flow & Interactions

### Researcher Workflow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RESEARCHER WORKFLOW                               │
└─────────────────────────────────────────────────────────────────────────────┘

    [START]
        │
        ▼
    ┌─────────────────┐
    │ Initialize      │
    │ Protocol        │
    └─────────────────┘
        │
        ▼
    ┌─────────────────┐
    │ Create Study    │
    └─────────────────┘
        │
        ├─► Define Study Parameters
        ├─► Set Enrollment Periods  
        └─► Configure Reward Amount
        │
        ▼
    ┌─────────────────┐
    │ Set Eligibility │
    │ Criteria        │
    └─────────────────┘
        │
        ├─► Age Requirements
        ├─► Gender Criteria
        └─► Location Constraints
        │
        ▼
    ┌─────────────────┐
    │ Create Reward   │
    │ Vault           │
    └─────────────────┘
        │
        ├─► Deposit Initial Tokens
        └─► Verify Sufficient Balance
        │
        ▼
    ┌─────────────────┐
    │ Publish Study   │
    └─────────────────┘
        │
        ├─► Study Becomes Visible
        └─► Accepting Participants
        │
        ▼
    ┌─────────────────┐
    │ Monitor         │
    │ Enrollments     │
    └─────────────────┘
        │
        ├─► Track Enrollment Count
        └─► Monitor Study Progress
        │
        ▼
    ┌─────────────────┐
    │ Review          │
    │ Submissions     │
    └─────────────────┘
        │
        ├─► Verify Data Quality
        └─► Check Completion Status
        │
        ▼
    ┌─────────────────┐
    │ Distribute      │
    │ Rewards         │
    └─────────────────┘
        │
        ├─► Automated Distribution
        └─► Track Reward Metrics
        │
        ▼
    ┌─────────────────┐
    │ Close Study     │
    └─────────────────┘
        │
        ├─► No New Enrollments
        └─► Finalize Study Data
        │
        ▼
    [END]
```

### Participant Workflow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PARTICIPANT WORKFLOW                               │
└─────────────────────────────────────────────────────────────────────────────┘

    [START]
        │
        ▼
    ┌─────────────────┐
    │ Browse Studies  │
    └─────────────────┘
        │
        ├─► View Study Details
        ├─► Read Requirements
        └─► Check Reward Amount
        │
        ▼
    ┌─────────────────┐
    │ Check           │
    │ Eligibility     │
    └─────────────────┘
        │
        ├─► Verify Age Range
        ├─► Confirm Gender Match
        ├─► Validate Location
        └─► Eligibility Confirmed
        │
        ▼
    ┌─────────────────┐
    │ Enroll & Mint   │
    │ Consent NFT     │
    └─────────────────┘
        │
        ├─► Provide Personal Info
        ├─► Sign Consent Agreement
        ├─► Receive Consent NFT
        └─► Study Enrollment Complete
        │
        ▼
    ┌─────────────────┐
    │ Submit Research │
    │ Data            │
    └─────────────────┘
        │
        ├─► Complete Survey/Study
        ├─► Encrypt Data
        ├─► Upload to IPFS
        └─► Submit Data Hash
        │
        ▼
    ┌─────────────────┐
    │ Mint Completion │
    │ NFT             │
    └─────────────────┘
        │
        ├─► Data Verification
        ├─► Quality Check Passed
        └─► Receive Completion NFT
        │
        ▼
    ┌─────────────────┐
    │ Claim Token     │
    │ Rewards         │
    └─────────────────┘
        │
        ├─► Wait 24h Cooldown
        ├─► Claim Tokens
        └─► Reward Distribution
        │
        ▼
    [END]
```

### Study Lifecycle States

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           STUDY LIFECYCLE FLOW                              │
└─────────────────────────────────────────────────────────────────────────────┘

    [START] 
        │
        ▼
    ┌─────────────┐
    │   DRAFT     │ ◄─── Study Creation Phase
    └─────────────┘
        │
        │ • Study Created
        │ • Eligibility Criteria Set
        │ • Reward Vault Created
        │ • Not Visible to Participants
        │
        │ publish_study()
        ▼
    ┌─────────────┐
    │ PUBLISHED   │ ◄─── Enrollment Phase
    └─────────────┘
        │
        │ • Accepting Enrollments
        │ • Participants Can Join
        │ • Consent NFTs Minted
        │ • Enrollment Period Active
        │
        │ transition_study_state()
        ▼
    ┌─────────────┐
    │   ACTIVE    │ ◄─── Data Collection Phase
    └─────────────┘
        │
        │ • Data Collection Active
        │ • Participants Submit Data
        │ • Completion NFTs Minted
        │ • Rewards Distributed
        │
        │ close_study()
        ▼
    ┌─────────────┐
    │   CLOSED    │ ◄─── Completion Phase
    └─────────────┘
        │
        │ • No New Enrollments
        │ • Data Collection Ended
        │ • Final Rewards Processed
        │ • Study Archived
        │
        ▼
    [END]

┌─────────────────────────────────────────────────────────────────────────────┐
│                           STATE TRANSITIONS                                 │
└─────────────────────────────────────────────────────────────────────────────┘

Draft ────── publish_study() ──────► Published
Published ── transition_study_state() ──► Active  
Active ───── close_study() ──────► Closed
```


#### AdminAccount
```rust
pub struct AdminAccount {
    pub protocol_admin: Pubkey,           // Protocol administrator
    pub protocol_fee_bps: u16,            // Protocol fee in basis points
    pub min_study_duration: u64,          // Minimum study duration
    pub max_study_duration: u64,          // Maximum study duration
    pub total_studies: u64,               // Total studies created
    pub total_participants: u64,          // Total participants enrolled
    pub total_rewards_distributed: u64,   // Total rewards distributed
    pub bump: u8,                         // PDA bump seed
}
```

#### StudyAccount
```rust
pub struct StudyAccount {
    pub study_id: u64,                    // Unique study identifier
    pub title: String,                    // Study title (max 100 chars)
    pub description: String,              // Study description (max 500 chars)
    pub researcher: Pubkey,               // Researcher's public key
    pub enrollment_start: i64,            // Enrollment start timestamp
    pub enrollment_end: i64,              // Enrollment end timestamp
    pub data_collection_end: i64,         // Data collection end timestamp
    pub max_participants: u32,            // Maximum participants allowed
    pub enrolled_count: u32,              // Current enrollment count
    pub reward_amount_per_participant: u64, // Reward per participant
    pub status: StudyStatus,              // Current study status
    pub completed_count: u32,             // Completed submissions count
    pub total_rewards_distributed: u64,   // Total rewards distributed
    pub created_at: i64,                  // Creation timestamp
    pub has_eligibility_criteria: bool,   // Eligibility criteria flag
    pub eligibility_criteria: Vec<u8>,    // Serialized eligibility data
    pub bump: u8,                         // PDA bump seed
}
```

#### ConsentAccount
```rust
pub struct ConsentAccount {
    pub study: Pubkey,                    // Associated study
    pub participant: Pubkey,              // Participant's public key
    pub eligibility_proof: Vec<u8>,       // Eligibility verification data
    pub timestamp: i64,                   // Consent timestamp
    pub is_revoked: bool,                 // Revocation status
    pub revocation_timestamp: Option<i64>, // Revocation timestamp
    pub nft_mint: Option<Pubkey>,         // Consent NFT mint address
    pub bump: u8,                         // PDA bump seed
}
```

#### SubmissionAccount
```rust
pub struct SubmissionAccount {
    pub study: Pubkey,                    // Associated study
    pub participant: Pubkey,              // Participant's public key
    pub encrypted_data_hash: [u8; 32],    // Hash of encrypted data
    pub ipfs_cid: String,                 // IPFS CID for data storage
    pub submission_timestamp: i64,        // Submission timestamp
    pub is_verified: bool,                // Verification status
    pub reward_distributed: bool,         // Reward distribution status
    pub completion_nft_mint: Option<Pubkey>, // Completion NFT mint
    pub bump: u8,                         // PDA bump seed
}
```

## Security Features

### Access Control
- **Researcher Authorization**: Only study creators can modify their studies
- **Participant Verification**: Consent accounts verify enrollment status
- **State Validation**: Study state transitions enforce business logic
- **PDA Security**: Program-derived addresses prevent unauthorized access

### Data Protection
- **Encrypted Submissions**: Data hashes ensure integrity
- **IPFS Storage**: Decentralized, censorship-resistant storage
- **NFT Verification**: Consent and completion NFTs prevent fraud
- **Time-based Constraints**: Enrollment and collection periods enforce deadlines

### NFT System

### Consent NFTs
- **Purpose**: Proof of study enrollment and consent
- **Attributes**: Study ID, title, consent date, researcher info
- **Metadata**: Stored on IPFS with dynamic attributes
- **Lifecycle**: Minted on enrollment, burned on revocation

### Completion NFTs
- **Purpose**: Achievement token for study completion
- **Attributes**: Study details, completion date, achievement type
- **Reward**: Unlocks token distribution eligibility
- **Permanent**: Cannot be revoked once minted

## Token Economics

### Reward Structure
- **Researcher Funding**: Initial deposit into reward vault
- **Participant Rewards**: Configurable per-participant amounts
- **Protocol Fees**: 2.5% default fee on study creation
- **Vault Management**: Automated token distribution

### Token Flow
```
Researcher → Reward Vault → Participant
     ↓              ↓           ↓
  Initial      Distribution   Completion
  Deposit      Automation     NFT + Tokens
```

### Installation
```bash
# Clone repository
git clone <repository-url>
cd RecruSearch

# Install dependencies
pnpm install

# Build program
anchor build

# Run tests
anchor test
```

### Configuration
```toml
# Anchor.toml
[provider]
cluster = "localnet"  # or "devnet", "mainnet"
wallet = "./programs/recru_search/Turbin3-wallet.json"

[programs.localnet]
recru_search = "HL4vrf5EV4eeaWyDLdzRgdjxxLiPfxiBvpWqjtKBPBNR"
mpl_core = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
```


### Key Dependencies
```toml
# Rust dependencies
mpl-core = "0.10.1"               # Metaplex NFT standard
anchor-spl = "0.31.1"             # SPL token integration
anchor-lang = "0.31.1"            # Anchor framework

# TypeScript dependencies
@coral-xyz/anchor = "^0.31.1"     # Anchor client
@solana/web3.js = "^1.98.2"       # Solana web3 client
@metaplex-foundation/mpl-core     # MPL Core client
```


## 📄 License

This project is licensed under the ISC License. See the LICENSE file for details.

*RecruSearch: Decentralizing Research Recruitment on Solana*
