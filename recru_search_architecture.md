# RecruSearch Protocol Architecture Design
## Privacy-First Research Participation Protocol on Solana

---

## 1. Protocol Overview

RecruSearch is a decentralized, privacy-first research participation protocol built on Solana using Anchor. It enables pseudonymous, fraud-resistant data collection through encrypted off-chain storage and on-chain consent verification for sensitive academic and DeSci research studies.

### Core Value Proposition
- **Privacy-First:** Client-side encryption with off-chain storage
- **Fraud-Resistant:** Cryptographic verification and unique wallet-based participation
- **Incentive-Aligned:** Token-based rewards and NFT completion certificates
- **Compliance-Ready:** Verifiable consent and audit trails for institutional requirements

---

## 2. Program Structure Architecture

### 2.1 Core Programs

```mermaid
graph TB
    subgraph "RecruSearch Protocol"
        SP[Study Program]
        CNP[Consent NFT Program] 
        SP_SUB[Submission Program]
        RVP[Reward Vault Program]
        ZKP[ZK Verification Program]
    end
    
    subgraph "External Solana Programs"
        MPX[Metaplex Token Metadata]
        SPL[SPL Token Program]
        SYS[System Program]
        CLOCK[Clock Sysvar]
    end
    
    subgraph "Off-Chain Services"
        IPFS[IPFS Storage]
        ZK_GEN[ZK Proof Generation]
        ENCRYPT[Client-Side Encryption]
    end
    
    SP --> CNP
    CNP --> SP_SUB
    SP_SUB --> RVP
    SP --> ZKP
    CNP --> MPX
    RVP --> SPL
    SP --> CLOCK
    SP_SUB --> IPFS
    ZKP --> ZK_GEN
    SP_SUB --> ENCRYPT
```

### 2.2 Program Responsibilities

| Program | Primary Responsibilities | Key Instructions |
|---------|-------------------------|------------------|
| **Study Program** | Study lifecycle management, eligibility validation, time-based state transitions | `create_study`, `publish_study`, `close_study` |
| **Consent NFT Program** | Consent verification, eligibility proof validation, NFT minting | `mint_consent_nft`, `verify_eligibility` |
| **Submission Program** | Encrypted data tracking, submission validation, completion verification | `submit_encrypted_data`, `verify_submission` |
| **Reward Vault Program** | Token escrow, time-locked releases, reward distribution | `create_reward_vault`, `distribute_reward` |
| **ZK Verification Program** | Zero-knowledge proof validation, privacy-preserving eligibility | `verify_zk_proof`, `validate_eligibility` |

---

## 3. Account Structure Mapping

### 3.1 Primary Account Types

```mermaid
erDiagram
    StudyAccount {
        u64 study_id
        Pubkey researcher
        String title
        String description
        bytes32 consent_document_hash
        bytes32 eligibility_merkle_root
        bool requires_zk_proof
        i64 enrollment_start
        i64 enrollment_end
        i64 data_collection_end
        StudyStatus status
        u32 max_participants
        u64 reward_amount_per_participant
        u32 enrolled_count
        u32 completed_count
        Pubkey reward_vault
        i64 created_at
        u8 bump
    }
    
    ConsentNFT {
        u64 study_id
        Pubkey participant
        bytes32 consent_hash
        bytes32 eligibility_proof_hash
        i64 consent_timestamp
        bool eligibility_verified
        u8 bump
    }
    
    SubmissionAccount {
        u64 study_id
        Pubkey participant
        Pubkey consent_nft
        bytes32 encrypted_data_hash
        String ipfs_cid
        i64 submission_timestamp
        bool reward_claimed
        Option_Pubkey completion_nft
        u8 bump
    }
    
    RewardVault {
        Pubkey study_account
        Pubkey vault_token_account
        Pubkey reward_mint
        u64 total_deposited
        u32 participants_rewarded
        i64 created_at
        u8 bump
    }
    
    CompletionNFT {
        u64 study_id
        Pubkey participant
        Pubkey submission_account
        String study_metadata
        i64 completion_timestamp
        u8 bump
    }
    
    StudyAccount ||--o{ ConsentNFT : "enables"
    ConsentNFT ||--|| SubmissionAccount : "authorizes"
    StudyAccount ||--|| RewardVault : "funded_by"
    SubmissionAccount ||--o| CompletionNFT : "earns"
```

### 3.2 Program-Derived Addresses (PDAs)

| Account Type | PDA Seeds | Purpose |
|--------------|-----------|---------|
| **StudyAccount** | `["study", researcher_pubkey, study_id]` | Unique study identification per researcher |
| **ConsentNFT** | `["consent", participant_pubkey, study_id]` | One consent per participant per study |
| **SubmissionAccount** | `["submission", participant_pubkey, study_id]` | One submission per participant per study |
| **RewardVault** | `["vault", study_account_pubkey]` | Study-specific reward escrow |
| **CompletionNFT** | `["completion", participant_pubkey, study_id]` | Completion certificate per participant |

---

## 4. User Interaction Flows

### 4.1 Researcher Flow

```mermaid
sequenceDiagram
    participant R as Researcher
    participant W as Phantom Wallet
    participant SP as Study Program
    participant RVP as Reward Vault Program
    participant IPFS as IPFS Storage
    
    R->>W: Connect Wallet
    R->>IPFS: Upload consent document
    IPFS-->>R: Return document hash
    R->>SP: create_study(title, description, timeline, eligibility)
    SP-->>R: StudyAccount created
    R->>W: Approve token deposit
    R->>RVP: create_reward_vault(initial_deposit)
    RVP->>SPL: Transfer tokens to vault
    RVP-->>R: RewardVault created
    R->>SP: publish_study()
    SP-->>R: Study published and discoverable
```

### 4.2 Participant Flow

```mermaid
sequenceDiagram
    participant P as Participant
    participant W as Phantom Wallet
    participant ZK as ZK Generator
    participant CNP as Consent NFT Program
    participant SUB as Submission Program
    participant RVP as Reward Vault Program
    participant IPFS as IPFS Storage
    
    P->>W: Connect Wallet
    P->>ZK: Generate eligibility proof (optional)
    ZK-->>P: ZK proof
    P->>CNP: mint_consent_nft(study_id, zk_proof)
    CNP->>MPX: Mint NFT with metadata
    CNP-->>P: ConsentNFT minted
    P->>IPFS: Upload encrypted survey data
    IPFS-->>P: Return IPFS CID
    P->>SUB: submit_encrypted_data(data_hash, ipfs_cid)
    SUB-->>P: Submission recorded
    P->>RVP: distribute_reward()
    RVP->>SPL: Transfer reward tokens
    RVP->>MPX: Mint CompletionNFT
    RVP-->>P: Rewards distributed
```

### 4.3 Time-Based Study Management Flow

```mermaid
stateDiagram-v2
    [*] --> Draft: create_study()
    Draft --> Published: publish_study()
    Published --> Recruiting: enrollment_start timestamp
    Recruiting --> ActiveCollection: enrollment_end timestamp
    ActiveCollection --> Closed: data_collection_end timestamp
    Closed --> Archived: manual archive
    
    note right of Recruiting: Participants can mint ConsentNFT
    note right of ActiveCollection: Participants can submit data
    note right of Closed: Final reward distribution only
```

---

## 5. External Dependencies and Integrations

### 5.1 Blockchain Dependencies

```mermaid
graph LR
    subgraph "Solana Core"
        SPL[SPL Token Program]
        SYS[System Program]
        RENT[Rent Sysvar]
        CLOCK[Clock Sysvar]
    end
    
    subgraph "Metaplex Ecosystem"
        MPX[Token Metadata Program]
        BUBBLEGUM[Compressed NFTs]
        AUCTION[Auction House]
    end
    
    subgraph "External Protocols"
        PYTH[Pyth Price Oracles]
        SWITCHBOARD[Switchboard Oracles]
    end
    
    RecruSearch --> SPL
    RecruSearch --> MPX
    RecruSearch --> CLOCK
    RecruSearch --> SYS
```

### 5.2 Off-Chain Infrastructure

```mermaid
graph TB
    subgraph "Storage Layer"
        IPFS[IPFS Network]
        ARWEAVE[Arweave Storage]
        S3[AWS S3 Backup]
    end
    
    subgraph "Privacy Layer"
        ZK_SNARK[zk-SNARK Proofs]
        ENCRYPTION[AES-256 Encryption]
        TLS[TLS Communication]
    end
    
    subgraph "User Interface"
        PHANTOM[Phantom Wallet]
        WEB_APP[Web Application]
        MOBILE[Mobile App]
    end
    
    subgraph "Analytics & Monitoring"
        INDEXER[Custom Indexer]
        ANALYTICS[Usage Analytics]
        ALERTS[Alert System]
    end
    
    RecruSearch --> IPFS
    RecruSearch --> ZK_SNARK
    RecruSearch --> PHANTOM
    RecruSearch --> INDEXER
```

---

## 6. Advanced Technical Features

### 6.1 Zero-Knowledge Proof Integration

```mermaid
flowchart TD
    A[Participant Eligibility Check] --> B{Requires ZK Proof?}
    B -->|No| C[Direct Eligibility Validation]
    B -->|Yes| D[Generate ZK Proof Client-Side]
    D --> E[Submit Proof with Consent]
    E --> F[On-Chain Proof Verification]
    F --> G{Proof Valid?}
    G -->|Yes| H[Mint Consent NFT]
    G -->|No| I[Reject Participation]
    C --> H
    H --> J[Enable Study Participation]
```

### 6.2 Time-Locked Escrow System

```mermaid
gantt
    title Longitudinal Study Reward Schedule
    dateFormat X
    axisFormat %d
    
    section Study Timeline
    Enrollment Period    :a1, 0, 14
    Session 1 Window     :a2, 14, 21
    Session 2 Window     :a3, 35, 42
    Session 3 Window     :a4, 56, 63
    Final Rewards        :a5, 63, 70
    
    section Reward Distribution
    Initial Deposit      :b1, 0, 1
    Session 1 Rewards    :b2, 21, 22
    Session 2 Rewards    :b3, 42, 43
    Final Completion     :b4, 63, 64
```

### 6.3 Compressed State Management

```mermaid
graph TB
    subgraph "Traditional Approach"
        T1[Individual Account per Participant]
        T2[High Rent Costs]
        T3[Limited Scalability]
    end
    
    subgraph "Compressed Approach"
        C1[Merkle Tree Storage]
        C2[Proof-Based Verification]
        C3[Batch Operations]
        C4[Minimal Rent Costs]
    end
    
    T1 --> C1
    T2 --> C4
    T3 --> C3
    
    C1 --> C2
    C2 --> C3
```

---

## 7. Security Architecture

### 7.1 Attack Vector Mitigation

| Attack Vector | Mitigation Strategy | Implementation |
|---------------|-------------------|----------------|
| **Double Consent** | PDA uniqueness | One ConsentNFT per wallet per study |
| **Fake Submissions** | Consent NFT verification | Require valid ConsentNFT for submission |
| **Reward Gaming** | Escrow + verification | Time-locked rewards with completion proof |
| **Study Manipulation** | Immutable post-publish | Study parameters locked after publication |
| **Privacy Breach** | Client-side encryption | Data encrypted before off-chain storage |
| **Eligibility Fraud** | ZK proofs + Merkle trees | Cryptographic proof without data exposure |

### 7.2 Access Control Matrix

```mermaid
graph TB
    subgraph "Researcher Permissions"
        R1[Create Study]
        R2[Fund Reward Vault]
        R3[Close Study]
        R4[View Aggregate Data]
    end
    
    subgraph "Participant Permissions"
        P1[Mint Consent NFT]
        P2[Submit Data]
        P3[Claim Rewards]
        P4[View Own Data]
    end
    
    subgraph "Admin Permissions"
        A1[Deploy Programs]
        A2[Upgrade Contracts]
        A3[Emergency Pause]
        A4[System Monitoring]
    end
    
    subgraph "Restricted Actions"
        X1[View Raw Participant Data]
        X2[Modify Submissions]
        X3[Unauthorized Rewards]
        X4[Identity Exposure]
    end
```

---

## 8. Performance and Scalability

### 8.1 Transaction Cost Analysis

| Operation | Estimated SOL Cost | Account Changes | Compute Units |
|-----------|-------------------|-----------------|---------------|
| Create Study | ~0.002 SOL | +1 StudyAccount | ~15,000 CU |
| Mint Consent NFT | ~0.003 SOL | +1 ConsentNFT, +1 Metadata | ~25,000 CU |
| Submit Data | ~0.001 SOL | +1 SubmissionAccount | ~10,000 CU |
| Distribute Reward | ~0.002 SOL | Token transfer + NFT mint | ~20,000 CU |
| Batch Operations | ~0.001 SOL per participant | Merkle proof verification | ~5,000 CU |

### 8.2 Scalability Metrics

```mermaid
graph LR
    subgraph "Current Capacity"
        C1[1000 studies/day]
        C2[10,000 participants/study]
        C3[<$0.01 per transaction]
    end
    
    subgraph "With Compression"
        S1[10,000 studies/day]
        S2[100,000 participants/study]
        S3[<$0.001 per transaction]
    end
    
    C1 --> S1
    C2 --> S2
    C3 --> S3
```

---

## 9. Compliance and Audit Framework

### 9.1 Regulatory Requirements

```mermaid
flowchart TB
    A[Research Study] --> B{IRB Approval Required?}
    B -->|Yes| C[Generate Compliance Report]
    B -->|No| D[Standard Privacy Checks]
    C --> E[Verifiable Consent Trail]
    D --> E
    E --> F[Audit Log Generation]
    F --> G[Data Provenance Report]
    G --> H[Institutional Submission]
```

### 9.2 Audit Trail Components

| Component | Data Captured | Verification Method |
|-----------|---------------|-------------------|
| **Consent Records** | Timestamp, wallet, study, consent hash | On-chain NFT metadata |
| **Eligibility Verification** | ZK proof hash, validation result | Cryptographic verification |
| **Data Submissions** | IPFS hash, submission time, participant | Immutable on-chain record |
| **Reward Distribution** | Amount, recipient, completion proof | SPL token transaction history |
| **Study Lifecycle** | State changes, timestamps, authority | Event log with signatures |

---

## 10. Deployment and Monitoring

### 10.1 Deployment Architecture

```mermaid
graph TB
    subgraph "Development"
        DEV_NET[Devnet Deployment]
        UNIT_TESTS[Unit Tests]
        INTEGRATION[Integration Tests]
    end
    
    subgraph "Staging"
        TESTNET[Testnet Deployment]
        LOAD_TESTS[Load Testing]
        SECURITY_AUDIT[Security Audit]
    end
    
    subgraph "Production"
        MAINNET[Mainnet Deployment]
        MONITORING[Real-time Monitoring]
        INCIDENT[Incident Response]
    end
    
    DEV_NET --> TESTNET
    TESTNET --> MAINNET
    UNIT_TESTS --> LOAD_TESTS
    LOAD_TESTS --> MONITORING
```

### 10.2 Monitoring Dashboard Metrics

| Metric Category | Key Indicators | Alert Thresholds |
|----------------|----------------|------------------|
| **Program Health** | Transaction success rate, CU usage | <95% success, >80% CU limit |
| **User Activity** | Daily active studies, new participants | <50% baseline activity |
| **Financial** | Reward distribution, vault balances | Unexpected balance changes |
| **Security** | Failed transactions, unauthorized access | Any security event |
| **Performance** | Transaction latency, confirmation time | >5s latency, >1min confirmation |

---

## 11. Future Enhancements

### 11.1 Roadmap Features

```mermaid
timeline
    title RecruSearch Development Roadmap
    
    section Q1 2024
        MVP Launch : Core study creation
                   : Consent NFT minting
                   : Basic reward distribution
    
    section Q2 2024
        ZK Integration : Zero-knowledge proofs
                      : Advanced eligibility
                      : Privacy enhancements
    
    section Q3 2024
        State Compression : Merkle tree implementation
                         : Batch operations
                         : Cost optimization
    
    section Q4 2024
        DAO Governance : Decentralized study review
                      : Community validation
                      : Protocol upgrades
    
    section 2025
        Institutional : University partnerships
                     : Clinical trial integration
                     : Regulatory compliance
```

### 11.2 Potential Protocol Extensions

| Extension | Description | Technical Requirements |
|-----------|-------------|----------------------|
| **Multi-Chain Support** | Expand to other blockchains | Cross-chain bridge integration |
| **AI Data Analysis** | On-chain ML model integration | Confidential computing protocols |
| **Reputation System** | Participant and researcher ratings | Decentralized identity integration |
| **Study Marketplace** | Discover and fund research | Automated matching algorithms |
| **Real-Time Analytics** | Live study progress tracking | Event streaming infrastructure |

---

## Conclusion

RecruSearch represents a novel approach to research participation that prioritizes privacy, security, and user sovereignty while maintaining the rigor and compliance requirements of academic and clinical research. The architecture leverages Solana's high-performance blockchain capabilities, advanced cryptographic techniques, and user-friendly Web3 interfaces to create a comprehensive platform for ethical, efficient, and transparent research data collection.

The modular design ensures scalability and extensibility while the privacy-first approach addresses critical concerns in sensitive research domains. By combining on-chain verification with off-chain encrypted storage, RecruSearch establishes a new standard for research integrity in the Web3 era. 