// Devnet deployment script for RecruSearch
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RecruSearch } from "../target/types/recru_search.js";
import { PublicKey } from "@solana/web3.js";

module.exports = async function (provider: anchor.AnchorProvider) {
  // Configure client to use the provider
  anchor.setProvider(provider);
  
  // Get the program
  const program = anchor.workspace.RecruSearch as Program<RecruSearch>;
  const programId = program.programId;
  
  console.log("🚀 Deploying RecruSearch to Devnet...");
  console.log("Program ID:", programId.toString());
  
  try {
    // Initialize protocol with default parameters
    const [adminState] = PublicKey.findProgramAddressSync(
      [Buffer.from("admin")],
      programId
    );
    
    console.log("📋 Initializing protocol...");
    console.log("Admin State PDA:", adminState.toString());
    
    // Note: Protocol initialization will be done by the admin after deployment
    // This is just a placeholder for the deployment script
    
    console.log("✅ RecruSearch deployed successfully!");
    console.log("🔗 View on Solana Explorer: https://explorer.solana.com/address/" + programId.toString() + "?cluster=devnet");
    
  } catch (error) {
    console.error("❌ Deployment failed:", error);
    throw error;
  }
};
