import { Box25519Keypair, box, randomBytes } from 'tweetnacl';
import { encode as encodeBase64, decode as decodeBase64 } from '@stablelib/base64';
import { create } from 'ipfs-http-client';

export class EncryptionManager {
    private ipfs;
    
    constructor() {
        // Initialize IPFS client
        this.ipfs = create({
            host: 'ipfs.infura.io',
            port: 5001,
            protocol: 'https'
        });
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

        // Store on IPFS
        const result = await this.ipfs.add(JSON.stringify(metadata));
        
        return {
            encrypted: encryptedMessage,
            nonce,
            cid: result.path
        };
    }

    /**
     * Stores consent form or study materials on IPFS
     */
    async storeDocument(
        document: Blob | File
    ): Promise<string> {
        const result = await this.ipfs.add(document);
        return result.path;
    }
}

export class DecryptionManager {
    private ipfs;
    
    constructor() {
        this.ipfs = create({
            host: 'ipfs.infura.io',
            port: 5001,
            protocol: 'https'
        });
    }

    /**
     * Decrypts data using researcher's private key
     */
    async fetchAndDecrypt(
        cid: string,
        researcherPrivateKey: Uint8Array
    ): Promise<any> {
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
    }

    /**
     * Fetches stored document from IPFS
     */
    async fetchDocument(cid: string): Promise<Uint8Array> {
        const chunks = [];
        for await (const chunk of this.ipfs.cat(cid)) {
            chunks.push(chunk);
        }
        return Buffer.concat(chunks);
    }
}
