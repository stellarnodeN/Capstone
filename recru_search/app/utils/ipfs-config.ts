export interface IPFSConfig {
    provider: 'web3storage' | 'nftstorage' | 'pinata' | 'custom';
    host?: string;
    port?: number;
    protocol?: 'http' | 'https';
    apiKey?: string;
    timeout?: number;
    retries?: number;
}

export interface IPFSProviderConfig {
    pinata: {
        apiKey: string;
        secretApiKey: string;
        gateway: string;
    };
    web3storage: {
        apiToken: string;
        endpoint: string;
    };
    nftstorage: {
        apiToken: string;
        endpoint: string;
    };
    custom: {
        host: string;
        port: number;
        protocol: 'http' | 'https';
        apiKey?: string;
    };
}

export class IPFSConfigManager {
    private static instance: IPFSConfigManager;
    private config: IPFSConfig;

    private constructor() {
        this.config = this.loadConfig();
    }

    public static getInstance(): IPFSConfigManager {
        if (!IPFSConfigManager.instance) {
            IPFSConfigManager.instance = new IPFSConfigManager();
        }
        return IPFSConfigManager.instance;
    }

    private loadConfig(): IPFSConfig {
        const env = process.env.NODE_ENV || 'development';
        
        if (env === 'production') {
            return this.loadProductionConfig();
        } else if (env === 'staging') {
            return this.loadStagingConfig();
        } else {
            return this.loadDevelopmentConfig();
        }
    }

    private loadProductionConfig(): IPFSConfig {
        // Production configuration - use free IPFS providers
        const provider = process.env.IPFS_PROVIDER as IPFSConfig['provider'] || 'web3storage';
        
        switch (provider) {
            case 'pinata':
                return {
                    provider: 'pinata',
                    apiKey: process.env.PINATA_API_KEY!,
                    timeout: 30000,
                    retries: 3
                };
            
            case 'web3storage':
                return {
                    provider: 'web3storage',
                    apiKey: process.env.WEB3STORAGE_API_TOKEN!,
                    timeout: 30000,
                    retries: 3
                };
            
            case 'nftstorage':
                return {
                    provider: 'nftstorage',
                    apiKey: process.env.NFTSTORAGE_API_TOKEN!,
                    timeout: 30000,
                    retries: 3
                };
            
            case 'custom':
                return {
                    provider: 'custom',
                    host: process.env.IPFS_HOST || 'localhost',
                    port: parseInt(process.env.IPFS_PORT || '5001'),
                    protocol: (process.env.IPFS_PROTOCOL as 'http' | 'https') || 'http',
                    apiKey: process.env.IPFS_API_KEY,
                    timeout: 30000,
                    retries: 3
                };
            
            default:
                throw new Error(`Unsupported IPFS provider: ${provider}`);
        }
    }

    private loadStagingConfig(): IPFSConfig {
        // Staging configuration - use free providers
        return {
            provider: 'web3storage',
            apiKey: process.env.WEB3STORAGE_API_TOKEN || 'test-token',
            timeout: 15000,
            retries: 2
        };
    }

    private loadDevelopmentConfig(): IPFSConfig {
        // Development configuration - use public gateway for testing
        return {
            provider: 'custom',
            host: 'dweb.link',
            port: 443,
            protocol: 'https',
            timeout: 10000,
            retries: 1
        };
    }

    public getConfig(): IPFSConfig {
        return this.config;
    }

    public updateConfig(newConfig: Partial<IPFSConfig>): void {
        this.config = { ...this.config, ...newConfig };
    }

    public getConnectionConfig() {
        const config = this.getConfig();
        
        switch (config.provider) {
            case 'pinata':
                return {
                    host: 'api.pinata.cloud',
                    port: 443,
                    protocol: 'https' as const,
                    headers: {
                        'pinata_api_key': config.apiKey!
                    }
                };
            
            case 'web3storage':
                return {
                    host: 'api.web3.storage',
                    port: 443,
                    protocol: 'https' as const,
                    headers: {
                        'Authorization': `Bearer ${config.apiKey}`
                    }
                };
            
            case 'nftstorage':
                return {
                    host: 'api.nft.storage',
                    port: 443,
                    protocol: 'https' as const,
                    headers: {
                        'Authorization': `Bearer ${config.apiKey}`
                    }
                };
            
            case 'custom':
                return {
                    host: config.host!,
                    port: config.port!,
                    protocol: config.protocol!,
                    headers: config.apiKey ? {
                        'Authorization': `Bearer ${config.apiKey}`
                    } : undefined
                };
            
            default:
                throw new Error(`Unsupported provider: ${config.provider}`);
        }
    }
}

export class IPFSHealthChecker {
    private configManager: IPFSConfigManager;

    constructor() {
        this.configManager = IPFSConfigManager.getInstance();
    }

    async checkHealth(): Promise<{
        status: 'healthy' | 'degraded' | 'unhealthy';
        latency: number;
        error?: string;
    }> {
        const startTime = Date.now();
        
        try {
            const config = this.configManager.getConnectionConfig();
            const testData = { test: 'health-check', timestamp: Date.now() };
            
            // Create a temporary IPFS client for health check
            const { create } = await import('ipfs-http-client');
            const ipfs = create(config);
            
            // Try to add a small test file
            const result = await ipfs.add(JSON.stringify(testData));
            
            // Try to read it back
            const chunks = [];
            for await (const chunk of ipfs.cat(result.path)) {
                chunks.push(chunk);
            }
            
            const latency = Date.now() - startTime;
            
            return {
                status: latency < 5000 ? 'healthy' : 'degraded',
                latency
            };
            
        } catch (error) {
            return {
                status: 'unhealthy',
                latency: Date.now() - startTime,
                error: error instanceof Error ? error.message : 'Unknown error'
            };
        }
    }
} 