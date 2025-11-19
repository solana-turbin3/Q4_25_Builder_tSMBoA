import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, Transaction, LAMPORTS_PER_SOL } from "@solana/web3.js";
import BN from "bn.js";
import { expect } from "chai";
import { CasinoNchurch } from "../target/types/casino_nchurch";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.theCasinoAndTheChurchCapstone as Program<CasinoNchurch>;
const connection = provider.connection;
const wallet = provider.wallet as anchor.Wallet & { payer: Keypair };

const ONE_USDC = 1_000_000;

let usdcMint: PublicKey;
let casinoStatePda: PublicKey;
let casinoVaultAta: PublicKey;
let liquidityPoolPda: PublicKey;
let lpTokenMintPda: PublicKey;
let lpVaultPda: PublicKey;

const findCasinoStatePda = () =>
  PublicKey.findProgramAddressSync([Buffer.from("casino_state")], program.programId);

const findVrfGameStatePda = (user: PublicKey) =>
  PublicKey.findProgramAddressSync(
    [Buffer.from("vrf_game_state"), user.toBuffer()],
    program.programId
  );

const findUserStatsPda = (user: PublicKey) =>
  PublicKey.findProgramAddressSync([Buffer.from("user_stats"), user.toBuffer()], program.programId);

const findQuestFactoryPda = () =>
  PublicKey.findProgramAddressSync([Buffer.from("quest_factory")], program.programId);

const findQuestCampaignPda = (creator: PublicKey, counter: BN) => {
  const counterBytes = counter.toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("quest_campaign"), creator.toBuffer(), counterBytes],
    program.programId
  );
};

const findQuestVaultPda = (creator: PublicKey, counter: BN) => {
  const counterBytes = counter.toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("quest_vault"), creator.toBuffer(), counterBytes],
    program.programId
  );
};

const findQuestRewardsPda = (creator: PublicKey, counter: BN) => {
  const counterBytes = counter.toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("quest_rewards"), creator.toBuffer(), counterBytes],
    program.programId
  );
};

const findLpUserStatsPda = (user: PublicKey) =>
  PublicKey.findProgramAddressSync([Buffer.from("lp_user_stats"), user.toBuffer()], program.programId);

const findLpStakingPda = (user: PublicKey, counter: BN) => {
  const counterBytes = counter.toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("lp_staking"), user.toBuffer(), counterBytes],
    program.programId
  );
};

const findLiquidityPoolPda = () =>
  PublicKey.findProgramAddressSync([Buffer.from("liquidity_pool")], program.programId);

const findLpTokenMintPda = () =>
  PublicKey.findProgramAddressSync([Buffer.from("lp_token_mint")], program.programId);

const findLpVaultPda = () =>
  PublicKey.findProgramAddressSync([Buffer.from("lp_vault")], program.programId);

const findFeeDistributionPda = (epoch: BN) => {
  const epochBytes = epoch.toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("fee_distribution"), epochBytes],
    program.programId
  );
};

async function airdropSol(pubkey: PublicKey, amount = 2 * LAMPORTS_PER_SOL) {
  const sig = await connection.requestAirdrop(pubkey, amount);
  await connection.confirmTransaction(sig, "confirmed");
}

async function createRandomnessAccount(space = 8) {
  const account = Keypair.generate();
  const lamports = await connection.getMinimumBalanceForRentExemption(space);
  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: account.publicKey,
      lamports,
      space,
      programId: SystemProgram.programId,
    })
  );
  await provider.sendAndConfirm(tx, [account]);
  return account;
}

async function createUserWithUsdc(amount: number) {
  const user = Keypair.generate();
  await airdropSol(user.publicKey);
  const ata = await getOrCreateAssociatedTokenAccount(
    connection,
    wallet.payer,
    usdcMint,
    user.publicKey
  );
  await mintTo(connection, wallet.payer, usdcMint, ata.address, wallet.payer, amount);
  return { user, tokenAccount: ata.address };
}

async function initializeCasino() {
  await program.methods
    .initializeCasino()
    .accounts({
      casinoState: casinoStatePda,
      casinoVault: casinoVaultAta,
      authority: wallet.publicKey,
      usdcMint,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    } as any)
    .rpc();
}

async function initializeLiquidityPool() {
  await program.methods
    .initializeLiquidityPool()
    .accounts({
      liquidityPool: liquidityPoolPda,
      lpTokenMint: lpTokenMintPda,
      lpVault: lpVaultPda,
      authority: wallet.publicKey,
      usdcMint,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    } as any)
    .rpc();
}

describe("the-casino-and-the-church-capstone", () => {
  before(async () => {
    usdcMint = await createMint(connection, wallet.payer, wallet.publicKey, null, 6);
    [casinoStatePda] = findCasinoStatePda();
    casinoVaultAta = getAssociatedTokenAddressSync(usdcMint, casinoStatePda, true);
    [liquidityPoolPda] = findLiquidityPoolPda();
    [lpTokenMintPda] = findLpTokenMintPda();
    [lpVaultPda] = findLpVaultPda();

    await initializeCasino();
    await initializeLiquidityPool();
  });

  describe("casino module", () => {
    it("initializes casino state with default config", async () => {
      const casinoState = await program.account.casinoState.fetch(casinoStatePda);
      expect(casinoState.authority.toBase58()).to.eq(wallet.publicKey.toBase58());
      expect(casinoState.houseEdgeConfig.slotsRtpBps).to.eq(9500);
      expect(casinoState.isActive).to.be.true;
    });

    describe("slots", () => {
      it("requests and settles a slots game using the test RNG", async () => {
        const betAmount = new BN(2 * ONE_USDC);
        const { user, tokenAccount } = await createUserWithUsdc(5 * ONE_USDC);
        const [vrfGameStatePda] = findVrfGameStatePda(user.publicKey);
        const [userStatsPda] = findUserStatsPda(user.publicKey);
        const randomnessAccount = await createRandomnessAccount();

        await program.methods
          .requestSlotsGame(betAmount, randomnessAccount.publicKey)
          .accounts({
            casinoState: casinoStatePda,
            casinoVault: casinoVaultAta,
            vrfGameState: vrfGameStatePda,
            userStats: userStatsPda,
            user: user.publicKey,
            userTokenAccount: tokenAccount,
            usdcMint,
            randomnessAccountData: randomnessAccount.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          } as any)
          .signers([user])
          .rpc();

        await program.methods
          .settleSlotsGame()
          .accounts({
            casinoState: casinoStatePda,
            casinoVault: casinoVaultAta,
            vrfGameState: vrfGameStatePda,
            userStats: userStatsPda,
            user: user.publicKey,
            userTokenAccount: tokenAccount,
            usdcMint,
            randomnessAccountData: randomnessAccount.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          } as any)
          .signers([user])
          .rpc();

        const vrfState = await program.account.vrfGameState.fetch(vrfGameStatePda);
        expect(vrfState.isComplete).to.be.true;
        expect(vrfState.gameData.length).to.eq(3);

        const userStats = await program.account.userStats.fetch(userStatsPda);
        expect(userStats.gamesPlayed.toNumber()).to.eq(1);

        const casinoState = await program.account.casinoState.fetch(casinoStatePda);
        expect(casinoState.totalGamesPlayed.toNumber()).to.be.greaterThan(0);
      });

      it("rejects a slots bet below the minimum stake", async () => {
        const betAmount = new BN(100); // below 1 USDC
        const { user, tokenAccount } = await createUserWithUsdc(ONE_USDC);
        const [vrfGameStatePda] = findVrfGameStatePda(user.publicKey);
        const [userStatsPda] = findUserStatsPda(user.publicKey);
        const randomnessAccount = await createRandomnessAccount();

        try {
          await program.methods
            .requestSlotsGame(betAmount, randomnessAccount.publicKey)
            .accounts({
              casinoState: casinoStatePda,
              casinoVault: casinoVaultAta,
              vrfGameState: vrfGameStatePda,
              userStats: userStatsPda,
              user: user.publicKey,
              userTokenAccount: tokenAccount,
              usdcMint,
              randomnessAccountData: randomnessAccount.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            } as any)
            .signers([user])
            .rpc();
          expect.fail("bet should have been rejected");
        } catch (err: any) {
          expect(err.error.errorCode.code).to.eq("BetAmountTooLow");
        }
      });
    });

    describe("roulette", () => {
      it("plays a roulette round end-to-end", async () => {
        const betAmount = new BN(2 * ONE_USDC);
        const { user, tokenAccount } = await createUserWithUsdc(5 * ONE_USDC);
        const [vrfGameStatePda] = findVrfGameStatePda(user.publicKey);
        const [userStatsPda] = findUserStatsPda(user.publicKey);
        const randomnessAccount = await createRandomnessAccount();

        await program.methods
          .requestRouletteGame(
            betAmount,
            { straight: {} },
            Buffer.from([7]),
            randomnessAccount.publicKey
          )
          .accounts({
            casinoState: casinoStatePda,
            casinoVault: casinoVaultAta,
            vrfGameState: vrfGameStatePda,
            userStats: userStatsPda,
            user: user.publicKey,
            userTokenAccount: tokenAccount,
            usdcMint,
            randomnessAccountData: randomnessAccount.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          } as any)
          .signers([user])
          .rpc();

        await program.methods
          .settleRouletteGame()
          .accounts({
            casinoState: casinoStatePda,
            casinoVault: casinoVaultAta,
            vrfGameState: vrfGameStatePda,
            userStats: userStatsPda,
            user: user.publicKey,
            userTokenAccount: tokenAccount,
            usdcMint,
            randomnessAccountData: randomnessAccount.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          } as any)
          .signers([user])
          .rpc();

        const vrfState = await program.account.vrfGameState.fetch(vrfGameStatePda);
        expect(vrfState.isComplete).to.be.true;
        expect(vrfState.gameData.length).to.eq(1);
      });

      it("rejects roulette bets with invalid number counts", async () => {
        const betAmount = new BN(ONE_USDC);
        const { user, tokenAccount } = await createUserWithUsdc(5 * ONE_USDC);
        const [vrfGameStatePda] = findVrfGameStatePda(user.publicKey);
        const [userStatsPda] = findUserStatsPda(user.publicKey);
        const randomnessAccount = await createRandomnessAccount();

        try {
          await program.methods
            .requestRouletteGame(
              betAmount,
              { split: {} },
              Buffer.from([1]),
              randomnessAccount.publicKey
            )
            .accounts({
              casinoState: casinoStatePda,
              casinoVault: casinoVaultAta,
              vrfGameState: vrfGameStatePda,
              userStats: userStatsPda,
              user: user.publicKey,
              userTokenAccount: tokenAccount,
              usdcMint,
              randomnessAccountData: randomnessAccount.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            } as any)
            .signers([user])
            .rpc();
          expect.fail("invalid split bet should fail");
        } catch (err: any) {
          expect(err.error.errorCode.code).to.eq("InvalidRouletteNumbers");
        }
      });
    });
  });

  describe("church / quest module", () => {
    it("cannot create a quest campaign until the factory is activated", async () => {
      const rewardPool = new BN(10 * ONE_USDC);
      const counter = new BN(1);
      const { user: creator, tokenAccount } = await createUserWithUsdc(20 * ONE_USDC);
      const [questCampaignPda] = findQuestCampaignPda(creator.publicKey, counter);
      const [questVaultPda] = findQuestVaultPda(creator.publicKey, counter);
      const [questRewardsPda] = findQuestRewardsPda(creator.publicKey, counter);
      const [questFactoryPda] = findQuestFactoryPda();

      try {
        await program.methods
          .createQuestCampaign("Demo Quest", "Earn rewards", rewardPool, 10, { social: {} }, counter)
          .accounts({
            questCampaign: questCampaignPda,
            questVault: questVaultPda,
            questRewards: questRewardsPda,
            questFactory: questFactoryPda,
            creator: creator.publicKey,
            creatorTokenAccount: tokenAccount,
            usdcMint,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          } as any)
          .signers([creator])
          .rpc();
        expect.fail("quest factory should gate campaign creation");
      } catch (err: any) {
        expect(err.error.errorCode.code).to.eq("QuestFactoryNotActive");
      }
    });
  });

  describe("liquidity module", () => {
    it("initializes the liquidity pool with default parameters", async () => {
      const liquidityPool = await program.account.liquidityPool.fetch(liquidityPoolPda);
      expect(liquidityPool.lpTokenMint.toBase58()).to.eq(lpTokenMintPda.toBase58());
      expect("active" in liquidityPool.status).to.be.true;
      expect(liquidityPool.totalLiquidity.toNumber()).to.eq(0);
    });

    it("allows users to deposit liquidity and receive LP tokens", async () => {
      const { user, tokenAccount } = await createUserWithUsdc(20 * ONE_USDC);
      const [lpUserStatsPda] = findLpUserStatsPda(user.publicKey);
      const userLpTokenAccount = getAssociatedTokenAddressSync(lpTokenMintPda, user.publicKey);

      await program.methods
        .depositLiquidity(new BN(5 * ONE_USDC))
        .accounts({
          liquidityPool: liquidityPoolPda,
          lpTokenMint: lpTokenMintPda,
          lpVault: lpVaultPda,
          userLpTokenAccount,
          lpUserStats: lpUserStatsPda,
          user: user.publicKey,
          userTokenAccount: tokenAccount,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        } as any)
        .signers([user])
        .rpc();

      const lpUserStats = await program.account.lpUserStats.fetch(lpUserStatsPda);
      expect(lpUserStats.totalLpTokens.toNumber()).to.eq(5 * ONE_USDC);

      const pool = await program.account.liquidityPool.fetch(liquidityPoolPda);
      expect(pool.totalLiquidity.toNumber()).to.be.greaterThan(0);
    });

    it("distributes placeholder platform fees into a fee distribution snapshot", async () => {
      const epoch = new BN(1);
      const [feeDistributionPda] = findFeeDistributionPda(epoch);

      await program.methods
        .distributePlatformFees(epoch)
        .accounts({
          liquidityPool: liquidityPoolPda,
          feeDistribution: feeDistributionPda,
          authority: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        } as any)
        .rpc();

      const feeDistribution = await program.account.feeDistribution.fetch(feeDistributionPda);
      expect(feeDistribution.totalPlatformFees.toNumber()).to.eq(1_000_000);
    });
  });
});
