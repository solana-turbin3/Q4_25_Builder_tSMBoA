import {
    address,
    appendTransactionMessageInstructions,
    assertIsTransactionWithinSizeLimit,
    compileTransaction,
    createTransactionMessage,
    getSignatureFromTransaction,
    lamports,
    pipe,
    sendAndConfirmTransactionFactory,
    setTransactionMessageFeePayerSigner,
    setTransactionMessageLifetimeUsingBlockhash,
    signTransactionMessageWithSigners,
    type TransactionMessageBytesBase64
} from "@solana/kit";

import { rpc, DevKeypair, rpcSubscriptions } from "./index";
import { getTransferSolInstruction } from "@solana-program/system";


const turbin3Wallet = address('Dbh6kfHKPm7E94ci9HeVt3nj2NLa5Syk2pzEHsG6GGon');


// const transferInstruction = getTransferSolInstruction({
//     source: DevKeypair,
//     destination: turbin3Wallet,
//     amount: lamports(1n * LAMPORTS_PER_SOL)
// });

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

// const transactionMessage = pipe(
//     createTransactionMessage({ version: 0 }),
//     tx => setTransactionMessageFeePayerSigner(DevKeypair, tx),
//     tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
//     tx => appendTransactionMessageInstructions([transferInstruction],tx),
//     );

// const signedTransaction = await signTransactionMessageWithSigners(transactionMessage);

// assertIsTransactionWithinSizeLimit(signedTransaction);

// const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({rpc, rpcSubscriptions });

// console.log(`Transferring 1 SOL to ${turbin3Wallet}`);

// try {
//     await sendAndConfirmTransaction(
//         signedTransaction,
//         { commitment: 'confirmed' }
//     );
//     const signature = getSignatureFromTransaction(signedTransaction);
//     console.log(`Success! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);
// } catch (e) {
//     console.error('Transfer failed:', e);
// }


// This part of this code is used to empty the dev wallet into the turbine wallet. I commented out the above to run this.

// First get the balance from our wallet
const { value: balance } = await rpc.getBalance(DevKeypair.address).send();

console.log(`The wallet's balance is: ${balance}`)
// Build a dummy transfer instruction with 0 amount to calculate the fee
const dummyTransferInstruction = getTransferSolInstruction({
    source: DevKeypair,
    destination: turbin3Wallet,
    amount: lamports(0n)
});

const dummyTransactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    tx => setTransactionMessageFeePayerSigner(DevKeypair, tx),
    tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    tx =>
    appendTransactionMessageInstructions([dummyTransferInstruction], tx)
);

// Compile the dummy transaction message to get the message bytes
const compiledDummy = compileTransaction(dummyTransactionMessage);
const dummyMessageBase64 = Buffer.from(compiledDummy.messageBytes).toString('base64') as TransactionMessageBytesBase64;
// Calculate the transaction fee
const { value: fee } = await rpc.getFeeForMessage(dummyMessageBase64).send() || 0n;

if (fee === null) {
    throw new Error('Unable to calculate transaction fee');
}

if (balance < fee ) {
    throw new Error(`Insufficient balance to cover the transaction fee. Balance: ${balance}, Fee: ${fee}`);
}
// Calculate the exact amount to send (balance minus fee)
const sendAmount = balance - fee;

const transferInstruction = getTransferSolInstruction({
    source: DevKeypair,
    destination: turbin3Wallet,
    amount: lamports(sendAmount)
});

console.log("Transferring all the available balance to the turbine wallet.")

const transactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    tx => setTransactionMessageFeePayerSigner(DevKeypair, tx),
    tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    tx => appendTransactionMessageInstructions([transferInstruction], tx)
);

const signedTransaction = await signTransactionMessageWithSigners(transactionMessage);

assertIsTransactionWithinSizeLimit(signedTransaction);

const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

try {
    await sendAndConfirmTransaction(
        signedTransaction,
        { commitment: 'confirmed' }
    );
    const signature = getSignatureFromTransaction(signedTransaction);
    console.log(`Success! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);
} catch (e) {
console.error('Transfer failed:', e);}