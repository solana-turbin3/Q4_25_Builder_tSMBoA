import { airdropFactory, lamports } from "@solana/kit";

import { DevKeypair, rpc, rpcSubscriptions, LAMPORTS_PER_SOL } from "./index.ts";

const airdrop = airdropFactory({ rpc, rpcSubscriptions });

console.log(`Airdropping 2 SOL to ${DevKeypair.address}`);

try {
    const sig = await airdrop({
        commitment: 'confirmed',
        recipientAddress: DevKeypair.address,
        lamports: lamports(2n * LAMPORTS_PER_SOL),
    });

    console.log(`Success! Check out your TX here: https://explorer.solana.com/tx/${sig}?cluster=devnet`);
} catch (error) {
    console.error(`Oops, something went wrong: ${error}`)
}