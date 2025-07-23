# RecruSearch User Stories (Atomic)

## 1. Researcher Creates a Study (High Priority)

**User Story:** As a researcher, I want to create a new study entry, so that I can begin the process of recruiting participants.

**User Flow:**
- Researcher logs in with a Solana wallet.
- Navigates to the "Create Study" page.
- Enters basic study details (title, description).
- Submits the form to create the study shell.

**End-to-End Workflow:**
1. Researcher authenticates using wallet.
2. Fills out and submits the basic study creation form.
3. Study metadata is stored on-chain and appears in the study directory.

**Priority:** High

---

## 2. Researcher Sets Eligibility Criteria (High Priority)

**User Story:** As a researcher, I want to define eligibility criteria for my study, so that only qualified participants can join.

**User Flow:**
- After creating a study, researcher navigates to the eligibility section.
- Specifies rules (e.g., age, prior participation, wallet reputation).
- Saves the eligibility logic.

**End-to-End Workflow:**
1. Researcher accesses the eligibility settings for a study.
2. Defines and saves eligibility logic, which is stored on-chain.
3. System enforces these rules for all participant applications.

**Priority:** High

---

## 3. Researcher Defines Data Collection Fields (High Priority)

**User Story:** As a researcher, I want to specify the data fields/questions for my study, so that I can collect the information I need from participants.

**User Flow:**
- Researcher navigates to the data collection section of their study.
- Adds questions or data fields to be collected.
- Marks fields for client-side encryption as needed.
- Saves the data schema.

**End-to-End Workflow:**
1. Researcher defines data fields/questions.
2. Data schema is registered off-chain, with encryption flags.
3. Schema is linked to the study for participant submissions.

**Priority:** High

---

## 4. Researcher Publishes Study (High Priority)

**User Story:** As a researcher, I want to publish my study, so that it becomes visible to potential participants.

**User Flow:**
- After configuring study details, eligibility, and data fields, researcher clicks "Publish".
- Study is listed in the public directory.

**End-to-End Workflow:**
1. Researcher finalizes and publishes the study.
2. Study status is updated on-chain to "active".
3. Study appears in the public study directory.

**Priority:** High

---

## 5. Participant Browses and Searches Studies (High Priority)

**User Story:** As a participant, I want to browse and search for available studies, so that I can find research opportunities that interest me.

**User Flow:**
- Participant logs in with wallet.
- Navigates to the study directory.
- Browses or searches for studies by keyword, topic, or eligibility.

**End-to-End Workflow:**
1. Participant accesses the study directory.
2. System displays available studies with filters/search.
3. Participant selects a study to view details.

**Priority:** High

---

## 6. Participant Reviews Study Details and Consent (High Priority)

**User Story:** As a participant, I want to review a study's details, eligibility criteria, and consent form, so that I can make an informed decision about joining and know if I am eligible.

**User Flow:**
- Participant selects a study from the directory.
- Views study description, eligibility criteria, and consent form.
- System checks if participant meets eligibility criteria.
- If eligible, participant can proceed to provide consent; otherwise, access is denied.

**End-to-End Workflow:**
1. Participant views study details, eligibility, and consent form.
2. System evaluates participant's eligibility based on on-chain logic.
3. If eligible, participant can proceed to consent; if not, participation is blocked.

**Priority:** High

---

## 7. Participant Provides On-Chain Consent (High Priority)

**User Story:** As an eligible participant, I want to provide on-chain consent for a study, so that my participation is cryptographically verifiable.

**User Flow:**
- After passing eligibility, participant signs a transaction to provide consent.
- Receives a consent NFT/token in their wallet.

**End-to-End Workflow:**
1. System confirms participant eligibility.
2. Participant signs a transaction to mint a consent NFT/token.
3. Consent is recorded on-chain and linked to the participant's wallet.
4. Participant gains access to the study's data submission form.

**Priority:** High

---

## 8. Participant Submits Encrypted Data (High Priority)

**User Story:** As a participant, I want to submit my study responses in an encrypted format, so that only authorized researchers can access my data.

**User Flow:**
- Participant fills out the encrypted data form for the study.
- Data is encrypted client-side before submission.
- Submission is sent to off-chain storage, with a reference anchored on-chain.

**End-to-End Workflow:**
1. Participant completes the encrypted form.
2. Data is encrypted in-browser using the researcher's public key.
3. Encrypted data is uploaded to off-chain storage (e.g., Arweave/IPFS).
4. A reference hash is stored on-chain, linking the submission to the consent NFT/token.

**Priority:** High

---

## 9. Researcher Reviews and Downloads Encrypted Data (High Priority)

**User Story:** As a researcher, I want to review and download encrypted participant data, so that I can analyze results while maintaining privacy.

**User Flow:**
- Researcher views submissions for their study.
- Downloads encrypted data files.
- Decrypts data locally using their private key.

**End-to-End Workflow:**
1. Researcher logs in and navigates to their study dashboard.
2. Views a list of participant submissions (with pseudonymous IDs).
3. Downloads encrypted data.
4. Decrypts data locally for analysis.

**Priority:** High

---

## 10. Participant Receives Token-Based Incentive (Medium Priority)

**User Story:** As a participant, I want to receive a token or NFT reward after submitting valid data, so that I am incentivized to participate honestly.

**User Flow:**
- After successful data submission, participant receives a reward (SPL token or NFT) to their wallet.
- Reward is visible in their wallet and can be used or traded as specified by the study.

**End-to-End Workflow:**
1. Participant submits encrypted data.
2. System verifies submission and eligibility.
3. Smart contract issues reward token/NFT to participant's wallet.
4. Participant receives notification of reward.

**Priority:** Medium

---

## 11. Researcher Audits Consent and Data Provenance (Medium Priority)

**User Story:** As a researcher, I want to audit the consent and data provenance for my study, so that I can ensure compliance and reproducibility.

**User Flow:**
- Researcher accesses an audit dashboard for their study.
- Views on-chain records of all consent tokens and data submissions.
- Downloads audit logs for compliance or publication.

**End-to-End Workflow:**
1. Researcher logs in and selects a study.
2. Dashboard displays all consent and data provenance records.
3. Researcher can export logs for external review.

**Priority:** Medium

---

## 12. Participant Views Participation History (Low Priority)

**User Story:** As a participant, I want to view my history of studies joined and rewards earned, so that I can track my research contributions.

**User Flow:**
- Participant logs in with wallet.
- Navigates to "My Participation" dashboard.
- Sees a list of studies joined, consent tokens held, and rewards received.

**End-to-End Workflow:**
1. Participant connects wallet.
2. Dashboard queries on-chain and off-chain data for participation history.
3. Displays studies, consent NFTs, and rewards in a user-friendly format.

**Priority:** Low

---

## 13. Researcher Closes or Archives a Study (Low Priority)

**User Story:** As a researcher, I want to close or archive a study, so that it is no longer available for new participants.

**User Flow:**
- Researcher selects "Close Study" or "Archive Study" from the dashboard.
- Study is removed from the public directory.

**End-to-End Workflow:**
1. Researcher closes or archives the study.
2. Study status is updated on-chain to "closed" or "archived".
3. Study is no longer visible to participants.

**Priority:** Low 