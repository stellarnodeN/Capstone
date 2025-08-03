import { box, randomBytes } from 'tweetnacl';
import { encode as encodeBase64, decode as decodeBase64 } from '@stablelib/base64';
import { create } from 'ipfs-http-client';
import { IPFSConfigManager } from './ipfs-config.js';

// Mock storage for development/testing
const mockStorage = new Map<string, any>();

export class EncryptionManager {
    private ipfs;
    private configManager: IPFSConfigManager;
    
    constructor() {
        this.configManager = new IPFSConfigManager();
        const config = this.configManager.getConfig();
        
        // For development/testing, use mock IPFS if no API key is available
        if (config.provider === 'custom' && !config.apiKey) {
            // Mock IPFS client for development
            this.ipfs = null;
        } else {
            // Initialize IPFS client with production configuration
            this.ipfs = create({
                host: config.endpoint || 'localhost',
                port: 5001,
                protocol: 'http'
            });
        }
    }

    /**
     * Encrypts data for a researcher and stores it on IPFS
     */
    async encryptAndStore(
        data: any,
        researcherPublicKey: Uint8Array
    ): Promise<{ 
        encrypted: Uint8Array,
        nonce: Uint8Array,
        cid: string 
    }> {
        // Convert data to JSON string then to Uint8Array
        const messageUint8 = new TextEncoder().encode(JSON.stringify(data));
        
        // Generate random nonce
        const nonce = randomBytes(box.nonceLength);
        
        // Generate ephemeral keypair for this encryption
        const ephemeralKeypair = box.keyPair();
        
        // Encrypt the message
        const encryptedMessage = box(
            messageUint8,
            nonce,
            researcherPublicKey,
            ephemeralKeypair.secretKey
        );

        // Prepare metadata with ephemeral public key
        const metadata = {
            version: '1.0',
            ephemeralPublicKey: encodeBase64(ephemeralKeypair.publicKey),
            nonce: encodeBase64(nonce),
            encryptedData: encodeBase64(encryptedMessage)
        };

        // Store on IPFS with retry logic
        const config = this.configManager.getConfig();
        
        // For development/testing, use mock IPFS if no API key is available
        if (config.provider === 'custom' && !config.apiKey) {
            // Mock IPFS implementation for development
            const mockCid = `mock-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
            
            // Store the original data in mock storage for decryption
            mockStorage.set(mockCid, data);
            
            return {
                encrypted: encryptedMessage,
                nonce,
                cid: mockCid
            };
        }
        
        let lastError: Error | null = null;
        
        for (let attempt = 1; attempt <= (config.retries || 3); attempt++) {
            try {
                const result = await this.ipfs.add(JSON.stringify(metadata));
                
                return {
                    encrypted: encryptedMessage,
                    nonce,
                    cid: result.path
                };
            } catch (error) {
                lastError = error instanceof Error ? error : new Error('Unknown IPFS error');
                
                if (attempt < (config.retries || 3)) {
                    // Wait before retry (exponential backoff)
                    await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
                }
            }
        }
        
        throw new Error(`Failed to store data on IPFS after ${config.retries || 3} attempts: ${lastError?.message}`);
    }

    /**
     * Stores consent form or study materials on IPFS
     */
    async storeDocument(
        document: Blob
    ): Promise<string> {
        const config = this.configManager.getConfig();
        
        // For development/testing, use mock IPFS if no API key is available
        if (config.provider === 'custom' && !config.apiKey) {
            // Mock IPFS implementation for development
            const mockCid = `mock-doc-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
            return mockCid;
        }
        
        let lastError: Error | null = null;
        
        for (let attempt = 1; attempt <= (config.retries || 3); attempt++) {
            try {
                const result = await this.ipfs.add(document);
                return result.path;
            } catch (error) {
                lastError = error instanceof Error ? error : new Error('Unknown IPFS error');
                
                if (attempt < (config.retries || 3)) {
                    await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
                }
            }
        }
        
        throw new Error(`Failed to store document on IPFS after ${config.retries || 3} attempts: ${lastError?.message}`);
    }
}

export class DecryptionManager {
    private ipfs;
    private configManager: IPFSConfigManager;
    
    constructor() {
        this.configManager = new IPFSConfigManager();
        const config = this.configManager.getConfig();
        
        // For development/testing, use mock IPFS if no API key is available
        if (config.provider === 'custom' && !config.apiKey) {
            // Mock IPFS client for development
            this.ipfs = null;
        } else {
            // Initialize IPFS client with production configuration
            this.ipfs = create({
                host: config.endpoint || 'localhost',
                port: 5001,
                protocol: 'http'
            });
        }
    }

    /**
     * Decrypts data using researcher's private key
     */
    async fetchAndDecrypt(
        cid: string,
        researcherPrivateKey: Uint8Array
    ): Promise<any> {
        const config = this.configManager.getConfig();
        
        // For development/testing, use mock IPFS if no API key is available
        if (config.provider === 'custom' && !config.apiKey) {
            // Mock decryption for development - retrieve original data from mock storage
            const originalData = mockStorage.get(cid);
            if (originalData) {
                return originalData;
            } else {
                // Fallback if data not found in mock storage
                throw new Error(`Mock data not found for CID: ${cid}`);
            }
        }
        
        let lastError: Error | null = null;
        
        for (let attempt = 1; attempt <= (config.retries || 3); attempt++) {
            try {
                // Fetch encrypted data from IPFS
                const chunks = [];
                for await (const chunk of this.ipfs.cat(cid)) {
                    chunks.push(chunk);
                }
                const data = JSON.parse(Buffer.concat(chunks).toString());

                // Decode components
                const encryptedData = decodeBase64(data.encryptedData);
                const nonce = decodeBase64(data.nonce);
                const ephemeralPublicKey = decodeBase64(data.ephemeralPublicKey);

                // Decrypt the message
                const decryptedMessage = box.open(
                    encryptedData,
                    nonce,
                    ephemeralPublicKey,
                    researcherPrivateKey
                );

                if (!decryptedMessage) {
                    throw new Error('Decryption failed');
                }

                // Parse and return the decrypted data
                return JSON.parse(new TextDecoder().decode(decryptedMessage));
                
            } catch (error) {
                lastError = error instanceof Error ? error : new Error('Unknown IPFS error');
                
                if (attempt < (config.retries || 3)) {
                    await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
                }
            }
        }
        
        throw new Error(`Failed to fetch and decrypt data after ${config.retries || 3} attempts: ${lastError?.message}`);
    }

    /**
     * Fetches stored document from IPFS
     */
    async fetchDocument(cid: string): Promise<Uint8Array> {
        const config = this.configManager.getConfig();
        
        // For development/testing, use mock IPFS if no API key is available
        if (config.provider === 'custom' && !config.apiKey) {
            // Mock document fetch for development
            return new TextEncoder().encode("mock-document-content");
        }
        
        let lastError: Error | null = null;
        
        for (let attempt = 1; attempt <= (config.retries || 3); attempt++) {
            try {
                const chunks = [];
                for await (const chunk of this.ipfs.cat(cid)) {
                    chunks.push(chunk);
                }
                return Buffer.concat(chunks);
            } catch (error) {
                lastError = error instanceof Error ? error : new Error('Unknown IPFS error');
                
                if (attempt < (config.retries || 3)) {
                    await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
                }
            }
        }
        
        throw new Error(`Failed to fetch document after ${config.retries || 3} attempts: ${lastError?.message}`);
    }
}
