import { EncryptionManager, DecryptionManager } from '../utils/encryption';
import { Keypair, PublicKey } from '@solana/web3.js';
import { BN } from 'bn.js';

export interface SurveyData {
  participantId: string;
  studyId: number;
  surveyResponses: Record<string, any>;
  timestamp: number;
  metadata?: Record<string, any>;
}

export interface EncryptedSubmission {
  encryptedDataHash: number[];
  ipfsCid: string;
  originalData: SurveyData;
}

export class DataSubmissionService {
  private encryptionManager: EncryptionManager;
  private decryptionManager: DecryptionManager;

  constructor() {
    this.encryptionManager = new EncryptionManager();
    this.decryptionManager = new DecryptionManager();
  }

  /**
   * Encrypts survey data for a specific researcher and prepares it for blockchain submission
   */
  async encryptSurveyData(
    surveyData: SurveyData,
    researcherPublicKey: PublicKey
  ): Promise<EncryptedSubmission> {
    try {
      // Convert researcher public key to Uint8Array for encryption
      const researcherPublicKeyBytes = new Uint8Array(researcherPublicKey.toBytes());
      
      // Encrypt the data and store on IPFS
      const { encrypted, cid } = await this.encryptionManager.encryptAndStore(
        surveyData,
        researcherPublicKeyBytes
      );

      // Use first 32 bytes as hash for blockchain storage
      const encryptedDataHash = Array.from(encrypted.slice(0, 32));

      return {
        encryptedDataHash,
        ipfsCid: cid,
        originalData: surveyData
      };
    } catch (error) {
      console.error('Failed to encrypt survey data:', error);
      throw new Error(`Encryption failed: ${error.message}`);
    }
  }

  /**
   * Decrypts survey data using researcher's private key
   */
  async decryptSurveyData(
    ipfsCid: string,
    researcherPrivateKey: Uint8Array
  ): Promise<SurveyData> {
    try {
      const decryptedData = await this.decryptionManager.fetchAndDecrypt(
        ipfsCid,
        researcherPrivateKey
      );
      return decryptedData;
    } catch (error) {
      console.error('Failed to decrypt survey data:', error);
      throw new Error(`Decryption failed: ${error.message}`);
    }
  }

  /**
   * Stores study documents (consent forms, schemas) on IPFS
   */
  async storeStudyDocument(document: Blob): Promise<string> {
    try {
      const cid = await this.encryptionManager.storeDocument(document);
      return cid;
    } catch (error) {
      console.error('Failed to store study document:', error);
      throw new Error(`Document storage failed: ${error.message}`);
    }
  }

  /**
   * Fetches study documents from IPFS
   */
  async fetchStudyDocument(cid: string): Promise<Uint8Array> {
    try {
      const document = await this.decryptionManager.fetchDocument(cid);
      return document;
    } catch (error) {
      console.error('Failed to fetch study document:', error);
      throw new Error(`Document fetch failed: ${error.message}`);
    }
  }

  /**
   * Validates survey data before encryption
   */
  validateSurveyData(data: SurveyData): boolean {
    if (!data.participantId || typeof data.participantId !== 'string') {
      throw new Error('Invalid participant ID');
    }
    
    if (!data.studyId || typeof data.studyId !== 'number') {
      throw new Error('Invalid study ID');
    }
    
    if (!data.surveyResponses || typeof data.surveyResponses !== 'object') {
      throw new Error('Invalid survey responses');
    }
    
    if (!data.timestamp || typeof data.timestamp !== 'number') {
      throw new Error('Invalid timestamp');
    }

    return true;
  }

  /**
   * Creates a sample survey data structure for testing
   */
  createSampleSurveyData(
    participantId: string,
    studyId: number,
    responses: Record<string, any>
  ): SurveyData {
    return {
      participantId,
      studyId,
      surveyResponses: responses,
      timestamp: Date.now(),
      metadata: {
        deviceType: 'web',
        completionTime: Math.floor(Math.random() * 300) + 60, // 1-6 minutes
        userAgent: navigator.userAgent
      }
    };
  }
} 