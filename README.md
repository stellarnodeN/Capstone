# RecruSearch 🔬🔒
### Privacy-First Research Participation Protocol on Solana

[![Solana](https://img.shields.io/badge/Solana-Network-purple)](https://solana.com)
[![Anchor](https://img.shields.io/badge/Anchor-Framework-blue)](https://anchor-lang.com)
[![Rust](https://img.shields.io/badge/Rust-Programming-orange)](https://www.rust-lang.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-Frontend-blue)](https://www.typescriptlang.org)

> A decentralized, privacy-first research participation protocol that enables pseudonymous, fraud-resistant data collection through encrypted off-chain storage and on-chain consent verification.

## 🌟 Core Features

### 🔒 Privacy-First Design
- **Client-Side Encryption**: Survey data encrypted before off-chain storage
- **Zero-Knowledge Proofs**: Eligibility verification without revealing personal data
- **Pseudonymous Participation**: Wallet-based identity without PII collection
- **Verifiable Consent**: Cryptographic proof of informed consent via NFTs

### 🏆 Dual NFT System
- **Consent NFTs**: Proof of informed consent and study entry authorization
- **Completion NFTs**: Certificates of participation and reward claim verification
- **Anti-Sybil Protection**: One NFT per wallet per study enforcement

### ⏰ Time-Bound Management
- **Automated Lifecycle**: Smart contract-driven study state transitions
- **Enrollment Windows**: Configurable start/end dates for participant recruitment
- **Data Collection Deadlines**: Automatic study closure with grace periods

### 💰 Incentive-Aligned Economics
- **Token-Based Rewards**: SPL token distribution upon completion
- **Escrow Security**: Time-locked reward vaults with conditional release
- **Fair Distribution**: Automatic reward calculation and distribution

## 🏗️ Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Researchers   │    │  Participants   │    │   Developers    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RecruSearch Protocol                         │
├─────────────────────────────────────────────────────────────────┤
│  Study Program  │ Consent NFT │ Submission │ Reward Vault │ ZK  │
│                 │   Program   │  Program   │   Program    │Prog │
└─────────────────────────────────────────────────────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Solana L1     │    │   Off-Chain     │    │   External      │
│   • SPL Tokens  │    │   • IPFS        │    │   • Metaplex    │
│   • Clock       │    │   • Encryption  │    │   • Oracles     │
│   • System      │    │   • ZK Proofs   │    │   • Wallets     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📋 Project Structure

```
RecruSearch/
├── 📁 recru_search/                    # Anchor project root
│   ├── 📁 programs/recru_search/       # Smart contract source
│   │   ├── 📁 src/
│   │   │   ├── 📁 instructions/        # Program instructions
│   │   │   │   ├── create_study.rs
│   │   │   │   ├── mint_consent_nft.rs
│   │   │   │   ├── submit_encrypted_data.rs
│   │   │   │   ├── distribute_reward.rs
│   │   │   │   └── ...
│   │   │   ├── 📁 state/              # Account structures
│   │   │   │   ├── study.rs
│   │   │   │   ├── consent_nft.rs
│   │   │   │   ├── submission.rs
│   │   │   │   └── ...
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   └── constants.rs
│   │   └── Cargo.toml
│   ├── 📁 tests/                      # Integration tests
│   └── 📁 migrations/                 # Deployment scripts
├── 📄 recru_search_architecture.md    # Comprehensive architecture
├── 📄 capstone_idea.md               # Project concept & market analysis
├── 📄 user_stories.md                # User requirements & stories
└── 📄 README.md                      # This file
```

## 🚀 Quick Start

### Prerequisites
- [Rust](https://rustlang.org/tools/install) 1.70+
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) 1.16+
- [Anchor Framework](https://anchor-lang.com/docs/installation) 0.28+
- [Node.js](https://nodejs.org/) 18+
- [Phantom Wallet](https://phantom.app/) or compatible Solana wallet

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/stellarnodeN/Capstone.git
   cd Capstone/recru_search
   ```

2. **Install dependencies**
   ```bash
   npm install
   # or
   yarn install
   ```

3. **Configure Solana for development**
   ```bash
   solana config set --url devnet
   solana-keygen new --outfile ~/.config/solana/id.json
   solana airdrop 2
   ```

4. **Build the program**
   ```bash
   anchor build
   ```

5. **Deploy to devnet**
   ```bash
   anchor deploy --provider.cluster devnet
   ```

6. **Run tests**
   ```bash
   anchor test
   ```

## 📖 Usage Examples

### For Researchers
```typescript
// Create a new research study
await program.methods
  .createStudy(
    studyId,
    "PTSD Recovery Study",
    "Anonymous survey on recovery methods",
    enrollmentStart,
    enrollmentEnd,
    dataCollectionEnd,
    maxParticipants,
    rewardAmount
  )
  .accounts({
    studyAccount,
    researcher: researcherWallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([researcherWallet])
  .rpc();
```

### For Participants
```typescript
// Mint consent NFT and join study
await program.methods
  .mintConsentNft(studyId, eligibilityProof, metadata)
  .accounts({
    consentNft,
    participant: participantWallet.publicKey,
    studyAccount,
    tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
  })
  .signers([participantWallet])
  .rpc();
```

## 🔧 Configuration

### Environment Variables
```bash
# .env
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json
SOLANA_NETWORK=devnet
```

### Anchor.toml
```toml
[features]
seeds = false
skip-lint = false

[programs.devnet]
recru_search = "YourProgramIdHere"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"
```

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
anchor test

# Run specific test file
anchor test --skip-deploy tests/recru_search.ts

# Run with verbose logging
anchor test --skip-deploy -- --verbose
```

## 🔒 Security Features

| Security Aspect | Implementation |
|-----------------|----------------|
| **Anti-Sybil** | PDA uniqueness ensures one consent per wallet per study |
| **Data Privacy** | Client-side encryption before off-chain storage |
| **Consent Verification** | Cryptographic NFT-based consent tracking |
| **Eligibility Fraud** | Zero-knowledge proofs for privacy-preserving verification |
| **Reward Gaming** | Time-locked escrow with completion verification |

## 💡 Advanced Features

### Zero-Knowledge Proof Integration
- Privacy-preserving eligibility verification
- Age, location, or condition proof without data exposure
- Compatible with Circom/snarkjs ecosystem

### State Compression
- Merkle tree-based participant management
- Reduced storage costs for large studies
- Batch operations for efficient processing

### Time-Locked Escrow
- Multi-session longitudinal studies
- Incremental reward release
- Automated completion verification

## 🗺️ Roadmap

- **Q1 2024**: MVP Launch with core features
- **Q2 2024**: Zero-knowledge proof integration
- **Q3 2024**: State compression and batch operations
- **Q4 2024**: DAO governance and community validation
- **2025**: University partnerships and clinical trial integration

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Architecture Documentation**: [recru_search_architecture.md](recru_search_architecture.md)
- **Project Concept**: [capstone_idea.md](capstone_idea.md)
- **User Stories**: [user_stories.md](user_stories.md)
- **Solana Documentation**: https://docs.solana.com
- **Anchor Framework**: https://anchor-lang.com

## 🙏 Acknowledgments

- Built with [Anchor Framework](https://anchor-lang.com)
- Powered by [Solana](https://solana.com) blockchain
- Privacy-preserving cryptography inspired by [Ethereum](https://ethereum.org) ZK research
- NFT standards from [Metaplex](https://metaplex.com)

---

**RecruSearch** - Advancing ethical research through blockchain innovation 🔬⚡ 