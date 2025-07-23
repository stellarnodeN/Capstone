# RecruSearch NFT Metadata Examples

This document shows the Metaplex-compatible JSON metadata structure that RecruSearch NFTs will generate following the standards from [Metaplex Token Creation Guide](https://developers.metaplex.com/guides/javascript/how-to-create-a-solana-token).

## Consent NFT Metadata Structure

### Example JSON for Consent NFT:
```json
{
  "name": "RecruSearch Consent - Mental Health Privacy Study",
  "symbol": "RSCONSENT",
  "description": "Proof of informed consent for participation in Mental Health research study. Granted pseudonymous access with Zero-Knowledge privacy protection.",
  "image": "https://arweave.net/AbCdEf123456789_consent_nft_image",
  "attributes": [
    {
      "trait_type": "Study ID",
      "value": "123"
    },
    {
      "trait_type": "Consent Date",
      "value": "2024-12-15"
    },
    {
      "trait_type": "Study Type",
      "value": "Mental Health"
    },
    {
      "trait_type": "Privacy Level",
      "value": "Zero-Knowledge"
    },
    {
      "trait_type": "Has ZK Proof",
      "value": "true"
    },
    {
      "trait_type": "Platform",
      "value": "RecruSearch"
    }
  ],
  "properties": {
    "files": [
      {
        "uri": "https://arweave.net/AbCdEf123456789_consent_nft_image",
        "type": "image/png"
      }
    ],
    "category": "image"
  }
}
```

## Completion NFT Metadata Structure

### Example JSON for Completion Certificate NFT:
```json
{
  "name": "RecruSearch Certificate - Mental Health Privacy Study",
  "symbol": "RSCOMPL",
  "description": "Certificate of completed participation in Mental Health research study. This NFT represents verified contribution to privacy-first scientific research.",
  "image": "https://arweave.net/XyZ987654321_completion_certificate",
  "attributes": [
    {
      "trait_type": "Study ID",
      "value": "123"
    },
    {
      "trait_type": "Completion Date",
      "value": "2024-12-30"
    },
    {
      "trait_type": "Study Type",
      "value": "Mental Health"
    },
    {
      "trait_type": "Study Duration",
      "value": "30 days"
    },
    {
      "trait_type": "Reward Amount",
      "value": "0.001 SOL"
    },
    {
      "trait_type": "Certificate Type",
      "value": "Research Participation"
    },
    {
      "trait_type": "Platform",
      "value": "RecruSearch"
    },
    {
      "trait_type": "Privacy Preserved",
      "value": "True"
    }
  ],
  "properties": {
    "files": [
      {
        "uri": "https://arweave.net/XyZ987654321_completion_certificate",
        "type": "image/png"
      }
    ],
    "category": "image"
  },
  "collection": {
    "name": "RecruSearch Certificates",
    "family": "RecruSearch"
  }
}
```

## Implementation in Frontend

When building the frontend, use the Metaplex packages to upload these JSON files to IPFS/Arweave:

### Required Packages:
```bash
npm i @metaplex-foundation/umi
npm i @metaplex-foundation/umi-bundle-defaults
npm i @metaplex-foundation/mpl-token-metadata
npm i @metaplex-foundation/umi-uploader-irys
npm i @metaplex-foundation/mpl-toolbox
```

### Upload Process:
1. **Generate image** for the NFT (consent form icon or certificate design)
2. **Upload image to IPFS/Arweave** using Metaplex Irys uploader
3. **Generate metadata JSON** using our smart contract helper functions
4. **Upload metadata JSON to IPFS/Arweave**
5. **Pass metadata URI** to our `mint_consent_nft` or `distribute_reward` instructions

### Smart Contract Integration:
Our Rust smart contracts provide helper functions:
- `ConsentNFTAccount::generate_metadata_json()` - Generates consent NFT metadata
- `CompletionNFTAccount::generate_metadata_json()` - Generates completion NFT metadata

These functions create the exact JSON structure that should be uploaded to IPFS/Arweave.

## Benefits of This Approach

### ✅ **Wallet Compatibility**
- NFTs display properly in Phantom, Solflare, and other Solana wallets
- Rich metadata with images and attributes visible in wallet UI
- Standard format recognized by all Solana wallet providers

### ✅ **Marketplace Compatibility** 
- NFTs can be listed on Magic Eden, OpenSea (Solana), and other marketplaces
- Proper trait filtering and searching capabilities
- Collection grouping for related study NFTs

### ✅ **Developer Experience**
- Standard JSON format easy to work with in frontend
- Rich attributes for filtering and analytics
- Extensible structure for future enhancements

### ✅ **Research Value**
- Professional-looking certificates that participants value
- Clear study identification and participation proof
- Audit trail of research contributions

## Future Enhancements

### **Collection Support**
Group all RecruSearch NFTs under verified collections for brand recognition.

### **Royalties**
Add royalty support for protocol fees or researcher attribution.

### **Enhanced Attributes**
Add more granular traits like:
- Research institution
- IRB approval number
- Data sensitivity level
- Participant demographics (anonymized)

This metadata structure ensures our NFTs follow Metaplex standards while providing meaningful value to research participants and maintaining compatibility with the broader Solana ecosystem. 