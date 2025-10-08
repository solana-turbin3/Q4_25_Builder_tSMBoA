import {
    appendTransactionMessageInstructions,
    assertIsTransactionWithinSizeLimit,
    createTransactionMessage,
    getSignatureFromTransaction,
    pipe,
    sendAndConfirmTransactionFactory,
    setTransactionMessageFeePayerSigner,
    setTransactionMessageLifetimeUsingBlockhash,
    signTransactionMessageWithSigners,
    getProgramDerivedAddress,
    generateKeyPairSigner,
    getAddressEncoder,
    addSignersToTransactionMessage,
    address
    } from "@solana/kit";

import { getInitializeInstruction, getSubmitTsInstruction } from "./clients/js/src/generated/index";

import { rpc, COLLECTION, Turbin3Keypair, TURBINE_PROGRAM, MPL_CORE_PROGRAM, SYSTEM_PROGRAM, rpcSubscriptions } from "./index";

const addressEncoder = getAddressEncoder();

// console.log("callin the initialize function")

// // Create the PDA for enrollment account
// const accountSeeds = [Buffer.from("prereqs"),addressEncoder.encode(Turbin3Keypair.address)];

// const [account, _bump] = await getProgramDerivedAddress({
//     programAddress: TURBINE_PROGRAM,
//     seeds: accountSeeds
// });

// // Generate mint keypair for the NFT
const mintKeyPair = await generateKeyPairSigner();

// // Execute the initialize transaction
// const initializeIx = getInitializeInstruction({
//     github: "tsmboa0",
//     user: Turbin3Keypair,
//     account,
//     systemProgram: SYSTEM_PROGRAM
// });

// // Fetch latest blockhash
// const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

// const transactionMessageInit = pipe(
//     createTransactionMessage({ version: 0 }),
//     tx => setTransactionMessageFeePayerSigner(Turbin3Keypair, tx),
//     tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash,tx),
//     tx => appendTransactionMessageInstructions([initializeIx], tx),
// );

// const signedTxInit = await signTransactionMessageWithSigners(transactionMessageInit);

// assertIsTransactionWithinSizeLimit(signedTxInit);

// const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({rpc, rpcSubscriptions });

// try {
//     const result = await sendAndConfirmTransaction(
//         signedTxInit,
//         { commitment: 'confirmed', skipPreflight: false }
//     );

//     console.log( `Initialize transaction result: ${result}`);

//     const signatureInit = getSignatureFromTransaction(signedTxInit);

//     console.log(`Success! Check out your TX here:https://explorer.solana.com/tx/${signatureInit}?cluster=devnet`);

// } catch (e) {
//         console.error(`Oops, something went wrong: ${e}`);
// }


// Execute the submitTs transaction

console.log("calling the submitTs function");

// Account created by the initialize step. I copied it from the explorer.
const account = address("B6WZrdE2q9dmd9usFoE7vRRtPgoZzPjxswiWyx9gyLUK");

// Create a PDA for the authority
const authoritySeeds = [Buffer.from("collection"),addressEncoder.encode(COLLECTION)];

const [authority, _authorityBump] = await getProgramDerivedAddress({
    programAddress: TURBINE_PROGRAM,
    seeds: authoritySeeds
});

const submitIx = getSubmitTsInstruction({
    user: Turbin3Keypair,
    account,
    mint: mintKeyPair,
    collection: COLLECTION,
    authority,
    mplCoreProgram: MPL_CORE_PROGRAM,
    systemProgram: SYSTEM_PROGRAM
});

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const transactionMessageSubmit = pipe(
    createTransactionMessage({ version: 0 }),
    tx => setTransactionMessageFeePayerSigner(Turbin3Keypair, tx),
    tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash,tx),
    tx => appendTransactionMessageInstructions([submitIx], tx),
    tx => addSignersToTransactionMessage([mintKeyPair], tx) // Add mint as additional signer after appending instructions
);

const signedTxSubmit = await signTransactionMessageWithSigners(transactionMessageSubmit);

assertIsTransactionWithinSizeLimit(signedTxSubmit);

const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({rpc, rpcSubscriptions });

try {
    await sendAndConfirmTransaction(
    signedTxSubmit,
    { commitment: 'confirmed', skipPreflight: false }
    );
    const signatureSubmit = getSignatureFromTransaction(signedTxSubmit);
    console.log(`Success! Check out your TX here: https://explorer.solana.com/tx/${signatureSubmit}?cluster=devnet`);
} catch (e) {
    console.error(`Oops, something went wrong: ${e}`);
}