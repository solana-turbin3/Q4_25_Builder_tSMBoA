import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmmStarterQ425 } from "../target/types/anchor_amm_starter_q4_25";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  getMint,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccount
} from "@solana/spl-token";
import { expect } from "chai";
import fs from "fs";

describe("anchor-amm-starter-q4-25", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorAmmStarterQ425 as Program<AnchorAmmStarterQ425>;
  
  // Test accounts
  let user: Keypair;
  let mintX: PublicKey;
  let mintY: PublicKey;
  let mintLp: PublicKey;
  let config: PublicKey;
  let vaultX: PublicKey;
  let vaultY: PublicKey;
  let userTokenX: PublicKey;
  let userTokenY: PublicKey;
  let userTokenLp: PublicKey;
  
  // Test parameters
  const SEED = new anchor.BN(Math.floor(Math.random() * 100));
  const FEE = 400; // 4% fee in basis points 
  
  function calculateDepositAmounts(vaultX: number, vaultY: number, lpSupply: number, amount: number) {
    if (lpSupply === 0) {
      return { x: amount, y: amount };
    }
    const ratioX = vaultX / lpSupply;
    const ratioY = vaultY / lpSupply;
    return {
      x: Math.floor(amount * ratioX),
      y: Math.floor(amount * ratioY)
    };
  }
  
  function calculateWithdrawAmounts(vaultX: number, vaultY: number, lpSupply: number, burnAmount: number) {
    const precision = 1_000_000;
    const ratio = ((lpSupply - burnAmount) * precision) / lpSupply;
    const withdrawX = vaultX - Math.floor(vaultX * ratio / precision);
    const withdrawY = vaultY - Math.floor(vaultY * ratio / precision);
    return {
      x: withdrawX,
      y: withdrawY
    };
  }
  
  function calculateSwapOutput(inputAmount: number, inputReserve: number, outputReserve: number, fee: number) {
    const feeAmount = (inputAmount * fee) / 10000;
    const inputAmountAfterFee = inputAmount - feeAmount;
    const numerator = inputAmountAfterFee * outputReserve;
    const denominator = inputReserve + inputAmountAfterFee;
    return Math.floor(numerator / denominator);
  }

  const AUTHORITY = null;

  before(async () => {
    // Use keypair from file for testing
    user = Keypair.fromSecretKey(new Uint8Array(JSON.parse(fs.readFileSync('~/.config/solana/id.json', 'utf8'))));
    
    // Create token mints
    mintX = await createMint(
      provider.connection,
      user,
      user.publicKey,
      null,
      6
    );
    
    mintY = await createMint(
      provider.connection,
      user,
      user.publicKey,
      null,
      6
    );
    
    // Create user token accounts
    userTokenX = getAssociatedTokenAddressSync(mintX, user.publicKey);
    userTokenY = getAssociatedTokenAddressSync(mintY, user.publicKey);
    
    // Create user token accounts
    await createAssociatedTokenAccount(provider.connection, user, mintX, user.publicKey);
    await createAssociatedTokenAccount(provider.connection, user, mintY, user.publicKey);
    
    // Mint tokens to user
    await mintTo(
      provider.connection,
      user,
      mintX,
      userTokenX,
      user,
      1000000 * 10**7 // 100M tokens
    );
    
    await mintTo(
      provider.connection,
      user,
      mintY,
      userTokenY,
      user,
      1000000 * 10**7 // 100M tokens
    );
    
    // Derive PDAs for config and lp mint
    [config] = PublicKey.findProgramAddressSync(
      [Buffer.from("config"), SEED.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    [mintLp] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), config.toBuffer()],
      program.programId
    );

    // Get associated token addresses for vaults and user token lp
    vaultX = await getAssociatedTokenAddress(mintX, config, true);
    vaultY = await getAssociatedTokenAddress(mintY, config, true);
    userTokenLp = await getAssociatedTokenAddress(mintLp, user.publicKey);
  });

  it("Initialize AMM pool and create LP mint", async () => {
    const tx = await program.methods
      .initialize(SEED, FEE, AUTHORITY)
      .accountsStrict({
        initializer: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        mintLp: mintLp,
        vaultX: vaultX,
        vaultY: vaultY,
        config: config,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    
    console.log("AMM pool initialized and LP mint created successfully", tx);
    
    // Verify config account
    const configAccount = await program.account.config.fetch(config);
    expect(configAccount.seed.toString()).to.equal(SEED.toString());
    expect(configAccount.fee).to.equal(FEE);
    expect(configAccount.authority).to.be.null;
    expect(configAccount.mintX.toString()).to.equal(mintX.toString());
    expect(configAccount.mintY.toString()).to.equal(mintY.toString());
    expect(configAccount.locked).to.be.false;
    
    // Verify LP mint was created
    const lpMintInfo = await getMint(provider.connection, mintLp);
    expect(lpMintInfo.decimals).to.equal(6);
    expect(lpMintInfo.supply.toString()).to.equal("0");
  });

  it("Deposit liquidity to empty pool", async () => {
    const depositAmount = 1000 * 10**7; // 1000M tokens
    const maxX = depositAmount;
    const maxY = depositAmount;
    
    // Get initial balances
    const initialUserX = await getAccount(provider.connection, userTokenX);
    const initialUserY = await getAccount(provider.connection, userTokenY);
    
    const tx = await program.methods
      .deposit(new anchor.BN(depositAmount), new anchor.BN(maxX), new anchor.BN(maxY))
      .accountsStrict({
        signer: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        config: config,
        mintLp: mintLp,
        vaultX: vaultX,
        vaultY: vaultY,
        userX: userTokenX,
        userY: userTokenY,
        userLp: userTokenLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    
    console.log("Liquidity deposit successful", tx);
    
    // Verify balances
    const finalUserX = await getAccount(provider.connection, userTokenX);
    const finalUserY = await getAccount(provider.connection, userTokenY);
    const finalVaultX = await getAccount(provider.connection, vaultX);
    const finalVaultY = await getAccount(provider.connection, vaultY);
    const finalUserLp = await getAccount(provider.connection, userTokenLp);
    const lpMintInfo = await getMint(provider.connection, mintLp);
    
    // Check that tokens were transferred from user to vault
    expect(Number(finalUserX.amount)).to.equal(Number(initialUserX.amount) - depositAmount);
    expect(Number(finalUserY.amount)).to.equal(Number(initialUserY.amount) - depositAmount);
    expect(Number(finalVaultX.amount)).to.equal(depositAmount);
    expect(Number(finalVaultY.amount)).to.equal(depositAmount);
    
    // Check LP tokens were minted
    expect(Number(finalUserLp.amount)).to.equal(depositAmount);
    expect(Number(lpMintInfo.supply)).to.equal(depositAmount);
  });

  it("Deposit liquidity to existing pool", async () => {
    const depositAmount = 300 * 10**7; // 300M tokens
    const maxX = 1_000_000 * 10**7; // 1000M tokens
    const maxY = 1_000_000 * 10**7; // 1000M tokens
    
    // Get initial state
    const initialVaultX = await getAccount(provider.connection, vaultX);
    const initialVaultY = await getAccount(provider.connection, vaultY);
    const initialLpSupply = await getMint(provider.connection, mintLp);
    const initialUserLp = await getAccount(provider.connection, userTokenLp);
    
    // Calculate expected amounts using constant product formula
    const expectedAmounts = calculateDepositAmounts(
      Number(initialVaultX.amount),
      Number(initialVaultY.amount),
      Number(initialLpSupply.supply),
      depositAmount
    );
    
    const tx = await program.methods
      .deposit(new anchor.BN(depositAmount), new anchor.BN(maxX), new anchor.BN(maxY))
      .accountsStrict({
        signer: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        config: config,
        mintLp: mintLp,
        vaultX: vaultX,
        vaultY: vaultY,
        userX: userTokenX,
        userY: userTokenY,
        userLp: userTokenLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    
    console.log("Liquidity deposit successful", tx);
    
    // Verify balances
    const finalVaultX = await getAccount(provider.connection, vaultX);
    const finalVaultY = await getAccount(provider.connection, vaultY);
    const finalLpSupply = await getMint(provider.connection, mintLp);
    const finalUserLp = await getAccount(provider.connection, userTokenLp);
    
    // Check that the amounts match the constant product formula
    const actualXAdded = Number(finalVaultX.amount) - Number(initialVaultX.amount);
    const actualYAdded = Number(finalVaultY.amount) - Number(initialVaultY.amount);
    const actualLpAdded = Number(finalUserLp.amount) - Number(initialUserLp.amount);
    
    // Allow for small rounding differences
    expect(Math.abs(actualXAdded - expectedAmounts.x)).to.be.lessThan(10);
    expect(Math.abs(actualYAdded - expectedAmounts.y)).to.be.lessThan(10);
    expect(Number(finalLpSupply.supply)).to.equal(Number(initialLpSupply.supply) + depositAmount);
    expect(actualLpAdded).to.equal(depositAmount);
  });

  it("Withdraw liquidity", async () => {
    const withdrawAmount = 300 * 10**7; // 300M LP tokens
    
    // Get initial state
    const initialVaultX = await getAccount(provider.connection, vaultX);
    const initialVaultY = await getAccount(provider.connection, vaultY);
    const initialLpSupply = await getMint(provider.connection, mintLp);
    const initialUserLp = await getAccount(provider.connection, userTokenLp);
    const initialUserX = await getAccount(provider.connection, userTokenX);
    const initialUserY = await getAccount(provider.connection, userTokenY);
    
    // Calculate expected amounts using constant product formula
    const expectedAmounts = calculateWithdrawAmounts(
      Number(initialVaultX.amount),
      Number(initialVaultY.amount),
      Number(initialLpSupply.supply),
      withdrawAmount
    );
    
    const tx = await program.methods
      .withdraw(new anchor.BN(withdrawAmount), new anchor.BN(0), new anchor.BN(0))
      .accountsStrict({
        signer: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        config: config,
        mintLp: mintLp,
        vaultX: vaultX,
        vaultY: vaultY,
        userX: userTokenX,
        userY: userTokenY,
        userLp: userTokenLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    
    console.log("Withdrawal successful", tx);
    
    // Verify balances
    const finalVaultX = await getAccount(provider.connection, vaultX);
    const finalVaultY = await getAccount(provider.connection, vaultY);
    const finalLpSupply = await getMint(provider.connection, mintLp);
    const finalUserLp = await getAccount(provider.connection, userTokenLp);
    const finalUserX = await getAccount(provider.connection, userTokenX);
    const finalUserY = await getAccount(provider.connection, userTokenY);
    
    // Check that LP tokens were burned
    expect(Number(finalUserLp.amount)).to.equal(Number(initialUserLp.amount) - withdrawAmount);
    expect(Number(finalLpSupply.supply)).to.equal(Number(initialLpSupply.supply) - withdrawAmount);
    
    // Check that tokens were withdrawn from vault to user
    const actualXWithdrawn = Number(finalUserX.amount) - Number(initialUserX.amount);
    const actualYWithdrawn = Number(finalUserY.amount) - Number(initialUserY.amount);
    const actualXRemoved = Number(initialVaultX.amount) - Number(finalVaultX.amount);
    const actualYRemoved = Number(initialVaultY.amount) - Number(finalVaultY.amount);
    
    // Allow for small rounding differences
    expect(Math.abs(actualXWithdrawn - expectedAmounts.x)).to.be.lessThan(10);
    expect(Math.abs(actualYWithdrawn - expectedAmounts.y)).to.be.lessThan(10);
    expect(actualXWithdrawn).to.equal(actualXRemoved);
    expect(actualYWithdrawn).to.equal(actualYRemoved);
  });

  it("Swap X for Y", async () => {
    const swapAmount = 100 * 10**7; // 100M tokens
    const minOutput = 0; // No slippage protection for simplicity
    
    // Get initial state
    const initialVaultX = await getAccount(provider.connection, vaultX);
    const initialVaultY = await getAccount(provider.connection, vaultY);
    const initialUserX = await getAccount(provider.connection, userTokenX);
    const initialUserY = await getAccount(provider.connection, userTokenY);
    
    // Calculate expected output using constant product formula
    const expectedOutput = calculateSwapOutput(
      swapAmount,
      Number(initialVaultX.amount),
      Number(initialVaultY.amount),
      FEE
    );
    
    const tx = await program.methods
      .swap(true, new anchor.BN(swapAmount), new anchor.BN(minOutput))
      .accountsStrict({
        signer: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        config: config,
        mintLp: mintLp,
        vaultX: vaultX,
        vaultY: vaultY,
        userX: userTokenX,
        userY: userTokenY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    
    console.log("Successfully swapped between X and Y", tx);
    
    // Verify balances
    const finalVaultX = await getAccount(provider.connection, vaultX);
    const finalVaultY = await getAccount(provider.connection, vaultY);
    const finalUserX = await getAccount(provider.connection, userTokenX);
    const finalUserY = await getAccount(provider.connection, userTokenY);
    
    // Check that X was deposited to vault
    const actualXDeposited = Number(finalVaultX.amount) - Number(initialVaultX.amount);
    expect(actualXDeposited).to.equal(swapAmount);
    
    // Check that Y was withdrawn from vault
    const actualYWithdrawn = Number(initialVaultY.amount) - Number(finalVaultY.amount);
    const actualYReceived = Number(finalUserY.amount) - Number(initialUserY.amount);
    
    // Allow for small rounding differences + fees
    expect(Math.abs(actualYWithdrawn - expectedOutput)).to.be.lessThan((swapAmount * 0.999) + (FEE * swapAmount / 10000));
    expect(actualYWithdrawn).to.equal(actualYReceived);
    
    // Verify constant product is maintained (approximately)
    const initialProduct = Number(initialVaultX.amount) * Number(initialVaultY.amount);
    const finalProduct = Number(finalVaultX.amount) * Number(finalVaultY.amount);
    const productRatio = finalProduct / initialProduct;
    
    // The product should increase due to fees
    expect(productRatio).to.be.greaterThan(1);
  });

  it("Swap Y for X", async () => {
    const swapAmount = 50 * 10**7; // 50M tokens
    const minOutput = 0; // No slippage protection for simplicity
    
    // Get initial state
    const initialVaultX = await getAccount(provider.connection, vaultX);
    const initialVaultY = await getAccount(provider.connection, vaultY);
    const initialUserX = await getAccount(provider.connection, userTokenX);
    const initialUserY = await getAccount(provider.connection, userTokenY);
    
    // Calculate expected output using constant product formula
    const expectedOutput = calculateSwapOutput(
      swapAmount,
      Number(initialVaultY.amount),
      Number(initialVaultX.amount),
      FEE
    );
    
    const tx = await program.methods
      .swap(false, new anchor.BN(swapAmount), new anchor.BN(minOutput))
      .accountsStrict({
        signer: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        config: config,
        mintLp: mintLp,
        vaultX: vaultX,
        vaultY: vaultY,
        userX: userTokenX,
        userY: userTokenY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    
    console.log("Successfully swapped between Y and X", tx);
    
    // Verify balances
    const finalVaultX = await getAccount(provider.connection, vaultX);
    const finalVaultY = await getAccount(provider.connection, vaultY);
    const finalUserX = await getAccount(provider.connection, userTokenX);
    const finalUserY = await getAccount(provider.connection, userTokenY);
    
    // Check that Y was deposited to vault
    const actualYDeposited = Number(finalVaultY.amount) - Number(initialVaultY.amount);
    expect(actualYDeposited).to.equal(swapAmount);
    
    // Check that X was withdrawn from vault
    const actualXWithdrawn = Number(initialVaultX.amount) - Number(finalVaultX.amount);
    const actualXReceived = Number(finalUserX.amount) - Number(initialUserX.amount);
    
    // Allow for small rounding differences + fees
    expect(Math.abs(actualXWithdrawn - expectedOutput)).to.be.lessThan((swapAmount * 0.999) + (FEE * swapAmount / 10000));
    expect(actualXWithdrawn).to.equal(actualXReceived);
  });

  it("Withdraw from empty pool should fail", async () => {
    // This should fail because there's no liquidity
    const seed = new anchor.BN(Math.floor(Math.random() * 100));

    const [config] = PublicKey.findProgramAddressSync(
      [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [mintLp] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), config.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize(seed, FEE, AUTHORITY)
      .accounts({
        initializer: user.publicKey,
        mintX: mintX,
        mintY: mintY,
        config: config,
      })
      .signers([user])
      .rpc();

    await createAssociatedTokenAccount(provider.connection, user, mintLp, user.publicKey);
    
    try {
      await program.methods
        .withdraw(new anchor.BN(1), new anchor.BN(0), new anchor.BN(0))
        .accounts({
          signer: user.publicKey,
          mintX: mintX,
          mintY: mintY,
          config: config,
        })
        .signers([user])
        .rpc();
      
      expect.fail("Failed to withdraw from empty pool");
    } catch (error) {
      expect(error.message).to.include("No liquidity in pool. You need to deposit some liquidity first.");
    }
  });

  it("Swap with slippage protection should fail", async () => {
    const swapAmount = 1000 * 10**7; // 1000M tokens
    const minOutput = 1000 * 10**7; // 1000M tokens
    
    try {
      await program.methods
        .swap(true, new anchor.BN(swapAmount), new anchor.BN(minOutput))
        .accountsStrict({
          signer: user.publicKey,
          mintX: mintX,
          mintY: mintY,
          config: config,
          mintLp: mintLp,
          vaultX: vaultX,
          vaultY: vaultY,
          userX: userTokenX,
          userY: userTokenY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
      
      expect.fail("Failed to swap due to slippage protection");
    } catch (error) {
      expect(error.message).to.include("Slippage exceeded");
    }
  });
});