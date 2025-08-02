import { DataSubmissionService, SurveyData } from '../dataSubmissionService';
import { Keypair, PublicKey } from '@solana/web3.js';

describe('DataSubmissionService', () => {
  let service: DataSubmissionService;
  let researcher: Keypair;
  let participantId: string;

  beforeEach(() => {
    service = new DataSubmissionService();
    researcher = Keypair.generate();
    participantId = 'test-participant-123';
  });

  describe('Survey Data Validation', () => {
    it('should validate correct survey data', () => {
      const validData: SurveyData = {
        participantId: participantId,
        studyId: 1,
        surveyResponses: {
          question1: 'Strongly agree',
          question2: 'Sometimes'
        },
        timestamp: Date.now()
      };

      expect(() => service.validateSurveyData(validData)).not.to.throw();
    });

    it('should reject invalid participant ID', () => {
      const invalidData: SurveyData = {
        participantId: '',
        studyId: 1,
        surveyResponses: { question1: 'test' },
        timestamp: Date.now()
      };

      expect(() => service.validateSurveyData(invalidData)).to.throw('Invalid participant ID');
    });

    it('should reject invalid study ID', () => {
      const invalidData: SurveyData = {
        participantId: participantId,
        studyId: 0,
        surveyResponses: { question1: 'test' },
        timestamp: Date.now()
      };

      expect(() => service.validateSurveyData(invalidData)).to.throw('Invalid study ID');
    });
  });

  describe('Sample Data Creation', () => {
    it('should create valid sample survey data', () => {
      const responses = {
        question1: 'Very satisfied',
        question2: 'Always',
        question3: 'Excellent'
      };

      const sampleData = service.createSampleSurveyData(participantId, 1, responses);

      expect(sampleData.participantId).to.equal(participantId);
      expect(sampleData.studyId).to.equal(1);
      expect(sampleData.surveyResponses).to.deep.equal(responses);
      expect(sampleData.timestamp).to.be.a('number');
      expect(sampleData.metadata).to.have.property('deviceType');
      expect(sampleData.metadata).to.have.property('completionTime');
    });
  });

  describe('Encryption and Decryption', () => {
    it('should encrypt and decrypt survey data successfully', async () => {
      const testData: SurveyData = {
        participantId: participantId,
        studyId: 1,
        surveyResponses: {
          question1: 'Strongly agree',
          question2: 'Sometimes',
          question3: 'Very satisfied'
        },
        timestamp: Date.now(),
        metadata: {
          deviceType: 'mobile',
          completionTime: 120
        }
      };

      // Encrypt the data
      const encryptedSubmission = await service.encryptSurveyData(
        testData,
        researcher.publicKey
      );

      expect(encryptedSubmission.encryptedDataHash).to.have.length(32);
      expect(encryptedSubmission.ipfsCid).to.be.a('string');
      expect(encryptedSubmission.ipfsCid.length).to.be.greaterThan(0);
      expect(encryptedSubmission.originalData).to.deep.equal(testData);

      // Decrypt the data
      const decryptedData = await service.decryptSurveyData(
        encryptedSubmission.ipfsCid,
        new Uint8Array(researcher.secretKey)
      );

      expect(decryptedData.participantId).to.equal(testData.participantId);
      expect(decryptedData.studyId).to.equal(testData.studyId);
      expect(decryptedData.surveyResponses).to.deep.equal(testData.surveyResponses);
      expect(decryptedData.metadata).to.deep.equal(testData.metadata);
    });

    it('should fail decryption with wrong private key', async () => {
      const testData: SurveyData = {
        participantId: participantId,
        studyId: 1,
        surveyResponses: { question1: 'test' },
        timestamp: Date.now()
      };

      // Encrypt with researcher's public key
      const encryptedSubmission = await service.encryptSurveyData(
        testData,
        researcher.publicKey
      );

      // Try to decrypt with wrong private key
      const wrongKeypair = Keypair.generate();
      
      await expect(
        service.decryptSurveyData(
          encryptedSubmission.ipfsCid,
          new Uint8Array(wrongKeypair.secretKey)
        )
      ).to.be.rejectedWith('Decryption failed');
    });
  });

  describe('Document Storage', () => {
    it('should store and fetch documents', async () => {
      const testContent = 'Test document content';
      const testBlob = new Blob([testContent], { type: 'text/plain' });

      // Store document
      const cid = await service.storeStudyDocument(testBlob);
      expect(cid).to.be.a('string');
      expect(cid.length).to.be.greaterThan(0);

      // Fetch document
      const fetchedDocument = await service.fetchStudyDocument(cid);
      const fetchedContent = new TextDecoder().decode(fetchedDocument);
      expect(fetchedContent).to.equal(testContent);
    });
  });

  describe('Error Handling', () => {
    it('should handle encryption errors gracefully', async () => {
      const invalidData = null as any;

      await expect(
        service.encryptSurveyData(invalidData, researcher.publicKey)
      ).to.be.rejectedWith('Encryption failed');
    });

    it('should handle decryption errors gracefully', async () => {
      await expect(
        service.decryptSurveyData('invalid-cid', new Uint8Array(32))
      ).to.be.rejectedWith('Decryption failed');
    });
  });
}); 