const express = require('express');
const app = express();
app.use(express.json());

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');
const { Keyring } = require('@polkadot/keyring');
const fs = require('fs');

let metadata;
fs.readFile('./metadata.json', (err, data) => {
    if (err) throw err;
    metadata = JSON.parse(data);
});

const port = 8080;
const provider = new WsProvider('wss://ws-smartnet.test.azero.dev');
const keyring = new Keyring({ type: 'sr25519' });
const contractAddr = '5C5LbckqB4YTBpt1EKLQqqBNbiSc2AzZ2Fje3ccbeRLswsmX';

let api;
let contract;
app.listen(port, async() => {
    api = await ApiPromise.create({ provider });
    contract = new ContractPromise(api, metadata, contractAddr);
    console.log(`Server listening on port ${port}`);
});

async function sendRes(res, result) {
    result = result.toHuman();
    console.log(result);
    let dispatchError = result.dispatchError;
    let internalError = result.internalError;
    if (dispatchError) {
        return res.status(400).send(dispatchError);
    }
    if (internalError) {
        return res.status(400).send(internalError);
    }

    return res.status(200).send(result);
}

async function createAuction(res, privateKey, startingPrice, description, duration, gasLimit) {
    const owner = keyring.createFromUri(privateKey);

    const { output } = await contract.query.getCreateAuctionFee(0, {});
    const createAuctionFee = output;

    return contract.tx.createAuction({ value: createAuctionFee, gasLimit: gasLimit }, startingPrice, description, duration)
        .signAndSend(owner, result => {
            if (result.status.isFinalized) {
                sendRes(res, result);
            }
        });
};

async function bid(res, privateKey, auctionId, bidPrice, gasLimit) {
    const owner = keyring.createFromUri(privateKey);

    return contract.tx.bid({ value: bidPrice, gasLimit: gasLimit }, auctionId)
        .signAndSend(owner, result => {
            if (result.status.isFinalized) {
                sendRes(res, result);
            }
        });
};

async function finishAuction(res, privateKey, auctionId, gasLimit) {
    const owner = keyring.createFromUri(privateKey);

    return contract.tx.finishAuction({ gasLimit: gasLimit }, auctionId)
        .signAndSend(owner, result => {
            if (result.status.isFinalized) {
                sendRes(res, result);
            }
        });
};

async function cancelAuction(res, privateKey, auctionId, gasLimit) {
    const owner = keyring.createFromUri(privateKey);

    const { output } = await contract.query.getCreateAuctionFee(0, {});
    const finalizeFee = output;

    // TODO: change const value to finalize fee
    return contract.tx.cancelAuction({ value: 10000000, gasLimit: gasLimit }, auctionId)
        .signAndSend(owner, result => {
            if (result.status.isFinalized) {
                sendRes(res, result);
            }
        });
};

async function getAuctions(res) {
    const { output } = await contract.query.getAuctions(0, {});
    res.status(200).send(output.toHuman());
};


app.get('/createauction', async(req, res) => {
    await createAuction(
        res,
        req.query.privateKey,
        req.query.startingPrice,
        req.query.description,
        req.query.duration,
        req.query.gasLimit
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

app.get('/bid', async(req, res) => {
    await bid(
        res,
        req.query.privateKey,
        req.query.auctionId,
        req.query.bidPrice,
        req.query.gasLimit
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

app.get('/finishauction', async(req, res) => {
    await finishAuction(
        res,
        req.query.privateKey,
        req.query.auctionId,
        req.query.gasLimit
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

app.get('/cancelauction', async(req, res) => {
    await cancelAuction(
        res,
        req.query.privateKey,
        req.query.auctionId,
        req.query.gasLimit
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

app.get('/getauctions', async(req, res) => {
    await getAuctions(
        res
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

// Example queries for private key 0x12d797ce064de04a047241cfcbde08033482a74be3a076fb1c32ffb33f01373c
// http://127.0.0.1:8080/createauction?privateKey=0x12d797ce064de04a047241cfcbde08033482a74be3a076fb1c32ffb33f01373c&startingPrice=123&description=haha&duration=123&gasLimit=10000000000
// http://127.0.0.1:8080/bid?privateKey=0x12d797ce064de04a047241cfcbde08033482a74be3a076fb1c32ffb33f01373c&auctionId=<INSERT ID>&bidPrice=123&gasLimit=10000000000
// http://127.0.0.1:8080/finishauction?privateKey=0x12d797ce064de04a047241cfcbde08033482a74be3a076fb1c32ffb33f01373c&auctionId=<INSERT ID>&gasLimit=10000000000
// http://127.0.0.1:8080/cancelauction?privateKey=0x12d797ce064de04a047241cfcbde08033482a74be3a076fb1c32ffb33f01373c&auctionId=<INSERT ID>&gasLimit=10000000000
// http://127.0.0.1:8080/getauctionson?privateKey=0x12d797ce064de04a047241cfcbde08033482a74be3a076fb1c32ffb33f01373c&auctionId=<INSERT ID>&gasLimit=10000000000
// http://127.0.0.1:8080/getauctions