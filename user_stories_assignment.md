# RecruSearch User Stories - Assignment Format

## Researcher Stories

**R1** – As a researcher, I want to enter the study title, description, and duration so that participants understand the purpose and scope.

**R2** – As a researcher, I want to upload or link to an off-chain consent document so that participants can review and approve it.

**R3** – As a researcher, I want to set eligibility conditions (e.g., age, diagnosis) so that only qualified participants can join.

**R4** – As a researcher, I want to deposit SPL tokens or NFTs into a study-specific reward vault so that participants can be compensated automatically.

**R5** – As a researcher, I want to publish the study to the blockchain so that it becomes available to participants via the front end.

**R6** – As a researcher, I want to track how many Consent NFTs have been issued so I can see how many people have joined.

**R7** – As a researcher, I want to see how many participants have completed the survey so I can gauge engagement.

**R8** – As a researcher, I want to export verifiable records (e.g., Consent NFT metadata or logs) to meet reproducibility or audit requirements.

**R9** – As a researcher, I want to set enrollment start and end dates so that participant recruitment happens within controlled time windows.

**R10** – As a researcher, I want to set data collection deadlines so that my study automatically closes after the specified period.

**R11** – As a researcher, I want to define data collection fields and questions so that I can gather the specific information needed for my research.

**R12** – As a researcher, I want to review and download encrypted participant data so that I can analyze results while maintaining privacy.

**R13** – As a researcher, I want to close or archive a study so that it is no longer available for new participants.

**R14** – As a researcher, I want to extend enrollment periods through contract updates so that I can adapt to changing research needs.



## Participant Stories

**P1** – As a participant, I want to connect my Phantom wallet without needing to provide my name or personal information so that I remain anonymous.

**P2** – As a participant, I want to read the consent form linked to the study so that I can understand my rights and what is expected.

**P3** – As a participant, I want to approve the consent form so that I can indicate my willingness to participate.

**P4** – As a participant, I want to prove that I meet the study's eligibility requirements without revealing personal data using zero-knowledge proofs.

**P5** – As a participant, I want to mint a Consent NFT after accepting the form and meeting eligibility so I can access the study.

**P6** – As a participant, I want to access the survey or task linked to the study so I can complete my participation.

**P7** – As a participant, I want to submit my response securely and privately so that my sensitive answers are protected through client-side encryption.

**P8** – As a participant, I want my submission to be linked to my Consent NFT so it can be verified that I participated legitimately.

**P9** – As a participant, I want to receive the study reward (SPL token or Completion NFT) after completing the task.

**P10** – As a participant, I want the system to ensure I can't join or claim rewards more than once for the same study.

**P11** – As a participant, I want to browse and search for available studies so that I can find research opportunities that interest me.

**P12** – As a participant, I want to review a study's details, eligibility criteria, and consent form so that I can make an informed decision about joining.

**P13** – As a participant, I want to view my participation history so that I can track my research contributions and earned rewards.

**P14** – As a participant, I want to receive rewards automatically through smart contracts so that compensation is guaranteed upon completion.

**P15** – As a participant, I want my wallet to serve as my pseudonymous identity so that I can participate across multiple studies while maintaining privacy.



## Developer Stories

**D1** – As the developer, I want to deploy the smart contracts for study, consent NFT, and reward vault so that the protocol logic can run on-chain.

**D2** – As the developer, I want to use Program Derived Addresses (PDAs) to store and link study-specific data securely and deterministically.

**D3** – As the developer, I want to create an NFT minting mechanism that stores consent metadata and links it to a specific study.

**D4** – As the developer, I want to build a vault to hold study rewards and release them only when participation is verified.

**D5** – As the developer, I want to write validation logic that prevents any wallet from minting more than one Consent NFT per study.

**D6** – As the developer, I want to enable researchers to query issued Consent NFTs by study so they can track who has joined.

**D7** – As the developer, I want to connect encrypted survey submissions to the Consent NFT on-chain so each participation can be verified.

**D8** – As the developer, I want to implement anti-Sybil measures so that wallets can't replay eligibility proofs or circumvent one-time participation logic.

**D9** – As the developer, I want to simulate full study creation, participant onboarding, and reward distribution to confirm expected behavior.

**D10** – As the developer, I want to log all key actions (consent, submission, reward) so the protocol state can be independently verified.

**D11** – As the developer, I want to integrate Phantom wallet connection using `@solana/wallet-adapter-react` so users can authenticate with their Web3 identity.

**D12** – As the developer, I want to implement time-based study state transitions using Solana's Clock sysvar so studies automatically move from recruiting to active to closed.

**D13** – As the developer, I want to create ZK eligibility verification logic that validates participant qualifications without exposing personal data.

**D14** – As the developer, I want to implement Merkle tree batch operations for efficient consent verification and reward distribution at scale.



**D16** – As the developer, I want to create automated state management that transitions studies based on enrollment deadlines and data collection periods.

**D17** – As the developer, I want to implement client-side encryption for survey responses using the researcher's public key.

**D18** – As the developer, I want to integrate with IPFS/Arweave for off-chain encrypted data storage while maintaining on-chain references.

**D19** – As the developer, I want to create event emission systems that notify users of state changes and transaction confirmations.

**D20** – As the developer, I want to implement proper account rent optimization and lifecycle management for efficient Solana resource usage. 