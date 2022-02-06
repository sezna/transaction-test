/** 
 * Generate test files for the transaction processor.
 *
 * Essentially implements a lightweight version of the processor without feature completeness to test
 * bulk processing.
 */

const fs = require('fs');
// script to generate test data
const NUM_CLIENTS = 100;
const STARTING_BALANCE = 0;
const NUM_TRANSACTIONS = 1000000;

let clientBalances = Array(NUM_CLIENTS).fill(0);
let clientLocked = Array(NUM_CLIENTS).fill(false);
let clientHeld = Array(NUM_CLIENTS).fill(0);

let disputedTxns = [];
let txns = [];

function randomTransactionType() {
  // have a bias towards withdraw/deposit since they're more common
  // 50% chance of deposit, 40% chance of withdrawal, 4% chance dispute, 3% chance chargeback, 3% chance resolve

  let randomNumber = Math.floor(Math.random() * 100);

  if (randomNumber < 50) { return 'deposit'; }
  else if (randomNumber < 90) { return 'withdrawal'; }
  else if (randomNumber < 94) { return 'dispute'; }
  else if (randomNumber < 97) { return 'chargeback'; }
  else { return 'resolve'; }
}

function randomNumber(max) {
  return Math.floor(Math.random() * max)
}
let input = "";

input += 'type,client,tx,amount\n';

for (let i = 0; i < NUM_TRANSACTIONS; i++) {
  let transaction = randomTransactionType();
  let client = randomNumber(NUM_CLIENTS);
  let amount = randomNumber(20);
  if (transaction == 'deposit') {
    if (clientLocked[client]) { continue; }
    clientBalances[client] += amount;
    txns.push({type: 'deposit', clientId: client, amount, id: i});
    input += `deposit,${client},${i},${amount}\n`;
  } else if (transaction == 'withdrawal') {
    if (clientLocked[client]) { continue; }
    clientBalances[client] -= amount;
    txns.push({type: 'withdrawal', clientId: client, amount, id: i});
    input += `withdrawal,${client},${i},${amount}\n`;
  } else if (transaction == 'dispute') {
    if (txns.length == 0) { continue; }
    let txnId = randomNumber(txns.length - 1);
    let txnToDispute = txns[txnId]; 
    if (clientLocked[txnToDispute.clientId]) { continue; }
    input += `dispute,${txnToDispute.clientId},${txnToDispute.id}\n`;
    if (txnToDispute.type == 'deposit') {
      clientHeld[txnToDispute.clientId] += txnToDispute.amount;
    }
    disputedTxns.push(txnToDispute);
  } else if (transaction == 'chargeback') {
    if (disputedTxns.length == 0) { continue; }
    let txnIx = randomNumber(disputedTxns.length - 1);
    let txnToChargeback = disputedTxns[txnIx];
    if (clientLocked[txnToChargeback.clientId]) { continue; }
    if (txnToChargeback.type == 'deposit') {
      clientBalances[txnToChargeback.clientId] -= txnToChargeback.amount;
      clientHeld[txnToChargeback.clientId] -= txnToChargeback.amount;
    } else if (txnToChargeback.type == 'withdrawal') {
      clientBalances[txnToChargeback.clientId] += txnToChargeback.amount;
    }
    input += `chargeback,${txnToChargeback.clientId},${txnToChargeback.id}\n`;
    disputedTxns.splice(txnIx, 1);
    clientLocked[txnToChargeback.clientId] = true;
  } else if (transaction == 'resolve') {
    if (disputedTxns.length == 0) { continue; }
    let txnIx = randomNumber(disputedTxns.length - 1);
    let txnToResolve = disputedTxns[txnIx];
    if (clientLocked[txnToResolve.clientId]) { continue; }
    if (txnToResolve.type == 'deposit') {
      clientHeld[txnToResolve.clientId] -= txnToResolve.amount;
    }
    input += `resolve,${txnToResolve.clientId},${txnToResolve.id}\n`;
    disputedTxns.splice(txnIx, 1);
  }

}

let expectedOutput = "client,available,held,total,locked\n";
for (let i = 0; i < NUM_CLIENTS; i++) {
  expectedOutput += `${i},${clientBalances[i] - clientHeld[i]},${clientHeld[i]},${clientBalances[i]},${clientLocked[i]}\n`;
}


fs.writeFile('input.csv', input, err => {
  if (err) {
    console.error(err)
    return
  }
})


fs.writeFile('output.csv', expectedOutput, err => {
  if (err) {
    console.error(err)
    return
  }
})
