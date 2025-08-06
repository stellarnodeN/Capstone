import { IPFSHTTPClient } from 'ipfs-http-client';

export interface IPFSConfig {
  provider: 'pinata' | 'infura' | 'custom';
  apiKey?: string;
  apiSecret?: string;
  endpoint?: string;
  gateway?: string;
  retries?: number;
  timeout?: number;
}

export class IPFSConfigManager {
  private config: IPFSConfig;

  constructor() {
    this.config = {
      provider: (process.env.IPFS_PROVIDER as 'pinata' | 'infura' | 'custom') || 'custom',
      apiKey: process.env.IPFS_API_KEY,
      apiSecret: process.env.IPFS_API_SECRET,
      endpoint: process.env.IPFS_ENDPOINT || 'http://localhost:5001',
      gateway: process.env.IPFS_GATEWAY || 'https://gateway.pinata.cloud',
      retries: parseInt(process.env.IPFS_RETRIES || '3'),
      timeout: parseInt(process.env.IPFS_TIMEOUT || '30000')
    };
  }

  getConfig(): IPFSConfig {
    return this.config;
  }

  setConfig(config: IPFSConfig) {
    this.config = config;
  }

  async uploadMetadata(metadata: any): Promise<string> {
    const config = this.getConfig();
    
    if (config.provider === 'pinata' && config.apiKey) {
      return this.uploadToPinata(metadata, config);
    } else if (config.provider === 'infura' && config.apiKey) {
      return this.uploadToInfura(metadata, config);
    } else {
      // For development/testing, return a mock IPFS hash
      console.log('Using mock IPFS upload for development');
      return 'ipfs://bafkreibmockmetadatahashforconsentnft';
    }
  }

  private async uploadToPinata(metadata: any, config: IPFSConfig): Promise<string> {
    try {
      const response = await fetch('https://api.pinata.cloud/pinning/pinJSONToIPFS', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'pinata_api_key': config.apiKey!,
          'pinata_secret_api_key': config.apiSecret!
        },
        body: JSON.stringify({
          pinataMetadata: {
            name: metadata.name || 'RecruSearch NFT Metadata'
          },
          pinataContent: metadata
        })
      });

      if (!response.ok) {
        throw new Error(`Pinata upload failed: ${response.statusText}`);
      }

      const result = await response.json() as { IpfsHash: string };
      return `ipfs://${result.IpfsHash}`;
    } catch (error) {
      console.error('Error uploading to Pinata:', error);
      throw error;
    }
  }

  private async uploadToInfura(metadata: any, config: IPFSConfig): Promise<string> {
    try {
      const response = await fetch('https://ipfs.infura.io:5001/api/v0/add', {
        method: 'POST',
        headers: {
          'Authorization': `Basic ${btoa(`${config.apiKey}:${config.apiSecret}`)}`
        },
        body: JSON.stringify(metadata)
      });

      if (!response.ok) {
        throw new Error(`Infura upload failed: ${response.statusText}`);
      }

      const result = await response.json() as { Hash: string };
      return `ipfs://${result.Hash}`;
    } catch (error) {
      console.error('Error uploading to Infura:', error);
      throw error;
    }
  }
} 