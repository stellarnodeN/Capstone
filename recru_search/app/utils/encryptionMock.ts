import { box, randomBytes } from 'tweetnacl';
import { encode as encodeBase64, decode as decodeBase64 } from '@stablelib/base64';

// In-memory storage for testing
const mockIPFSStorage = new Map<string, string>();

export class EncryptionManager {
    /**
     * Encrypts data for a researcher and stores it in mock IPFS
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

        // Store in mock IPFS (generate a fake CID)
        const cid = `QmMock${Date.now()}${Math.random().toString(36).substr(2, 9)}`;
        mockIPFSStorage.set(cid, JSON.stringify(metadata));
        
        return {
            encrypted: encryptedMessage,
            nonce,
            cid
        };
    }

    /**
     * Stores consent form or study materials in mock IPFS
     */
    async storeDocument(document: Blob): Promise<string> {
        const cid = `QmMockDoc${Date.now()}${Math.random().toString(36).substr(2, 9)}`;
        const content = await document.text();
        mockIPFSStorage.set(cid, content);
        return cid;
    }
}

export class DecryptionManager {
    /**
     * Decrypts data using researcher's private key
     */
    async fetchAndDecrypt(
        cid: string,
        researcherPrivateKey: Uint8Array
    ): Promise<any> {
        // Fetch encrypted data from mock IPFS
        const dataString = mockIPFSStorage.get(cid);
        if (!dataString) {
            throw new Error('Data not found in mock IPFS');
        }

        const data = JSON.parse(dataString);

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
    }

    /**
     * Fetches stored document from mock IPFS
     */
    async fetchDocument(cid: string): Promise<Uint8Array> {
        const content = mockIPFSStorage.get(cid);
        if (!content) {
            throw new Error('Document not found in mock IPFS');
        }
        return new TextEncoder().encode(content);
    }
}

// Export the storage for testing purposes
export const getMockIPFSStorage = () => mockIPFSStorage;
export const clearMockIPFSStorage = () => mockIPFSStorage.clear(); 