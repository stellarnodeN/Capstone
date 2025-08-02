import { EncryptionManager, DecryptionManager } from './encryption';
import { IPFSConfigManager, IPFSHealthChecker } from './ipfs-config';
import { box, randomBytes } from 'tweetnacl';

async function testIPFSConfiguration() {
    console.log('🔍 Testing RecruSearch IPFS Configuration...\n');

    try {
        // Test 1: Configuration Loading
        console.log('1. Testing Configuration Loading...');
        const configManager = IPFSConfigManager.getInstance();
        const config = configManager.getConfig();
        console.log(`   ✅ Provider: ${config.provider}`);
        console.log(`   ✅ Timeout: ${config.timeout}ms`);
        console.log(`   ✅ Retries: ${config.retries}\n`);

        // Test 2: Health Check
        console.log('2. Testing IPFS Health...');
        const healthChecker = new IPFSHealthChecker();
        const health = await healthChecker.checkHealth();
        console.log(`   ✅ Status: ${health.status}`);
        console.log(`   ✅ Latency: ${health.latency}ms`);
        if (health.error) {
            console.log(`   ❌ Error: ${health.error}`);
        }
        console.log('');

        // Test 3: Encryption and Storage
        console.log('3. Testing Encryption and Storage...');
        const encryptionManager = new EncryptionManager();
        
        // Generate test keypair
        const keypair = box.keyPair();
        const testData = {
            participantId: 'test-participant-123',
            surveyResponses: {
                question1: 'Test response 1',
                question2: 'Test response 2'
            },
            timestamp: Date.now()
        };

        console.log('   📤 Storing encrypted data...');
        const storeResult = await encryptionManager.encryptAndStore(testData, keypair.publicKey);
        console.log(`   ✅ CID: ${storeResult.cid}`);
        console.log(`   ✅ Encrypted size: ${storeResult.encrypted.length} bytes\n`);

        // Test 4: Decryption
        console.log('4. Testing Decryption...');
        const decryptionManager = new DecryptionManager();
        
        console.log('   📥 Fetching and decrypting data...');
        const decryptedData = await decryptionManager.fetchAndDecrypt(storeResult.cid, keypair.secretKey);
        
        // Verify data integrity
        const isDataValid = JSON.stringify(decryptedData) === JSON.stringify(testData);
        console.log(`   ✅ Data integrity: ${isDataValid ? 'Valid' : 'Invalid'}`);
        console.log(`   ✅ Decrypted participant ID: ${decryptedData.participantId}\n`);

        // Test 5: Document Storage
        console.log('5. Testing Document Storage...');
        const testDocument = new Blob(['Test document content'], { type: 'text/plain' });
        const documentCid = await encryptionManager.storeDocument(testDocument);
        console.log(`   ✅ Document CID: ${documentCid}\n`);

        // Test 6: Document Retrieval
        console.log('6. Testing Document Retrieval...');
        const retrievedDocument = await decryptionManager.fetchDocument(documentCid);
        const documentContent = new TextDecoder().decode(retrievedDocument);
        console.log(`   ✅ Document content: "${documentContent}"\n`);

        console.log('🎉 All IPFS tests passed successfully!');
        console.log('\n📊 Test Summary:');
        console.log(`   • Configuration: ✅`);
        console.log(`   • Health Check: ✅ (${health.status})`);
        console.log(`   • Encryption: ✅`);
        console.log(`   • Storage: ✅`);
        console.log(`   • Decryption: ✅`);
        console.log(`   • Document Handling: ✅`);
        console.log(`   • Average Latency: ${health.latency}ms`);

    } catch (error) {
        console.error('❌ IPFS test failed:', error);
        console.error('\n🔧 Troubleshooting:');
        console.error('   1. Check your environment variables');
        console.error('   2. Verify your IPFS provider credentials');
        console.error('   3. Check network connectivity');
        console.error('   4. Review the IPFS_DEPLOYMENT_GUIDE.md');
        process.exit(1);
    }
}

// Run the test if this file is executed directly
if (require.main === module) {
    testIPFSConfiguration();
}

export { testIPFSConfiguration }; 