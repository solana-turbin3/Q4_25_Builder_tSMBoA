import { address, createKeyPairSignerFromBytes, createSolanaRpc, createSolanaRpcSubscriptions, devnet } from "@solana/kit";

import DevWallet from "./dev-wallet.json";
import Turbin3Wallet from "./turbin3-wallet.json";

export const LAMPORTS_PER_SOL = BigInt(1_000_000_000);
export const TURBINE_PROGRAM = address("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM");
export const MPL_CORE_PROGRAM = address("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
export const SYSTEM_PROGRAM = address("11111111111111111111111111111111");
export const COLLECTION = address("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2");

export const DevKeypair = await createKeyPairSignerFromBytes(new Uint8Array(DevWallet));

export const Turbin3Keypair = await createKeyPairSignerFromBytes(new Uint8Array(Turbin3Wallet));

// Create an rpc connection
export const rpc = createSolanaRpc(devnet("https://api.devnet.solana.com"));

export const rpcSubscriptions = createSolanaRpcSubscriptions(devnet('ws://api.devnet.solana.com'));