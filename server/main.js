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
const contractAddr = '5DbGKPyM3QXaRGhdb3JycuUZ2U3jL5ucwX8NnX21GwCWorxs';

app.use(function(req, res, next) {
    res.header("Access-Control-Allow-Origin", "*");
    res.header("Access-Control-Allow-Methods", "GET, PUT, POST");
    res.header("Access-Control-Allow-Headers", "Origin, X-Requested-With, Content-Type, Accept");
    next();
});

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

    return res.status(201).send(result);
}

async function createAuction(res, privateKey, startingBid, description, duration, gasLimit) {
    const owner = keyring.createFromUri(privateKey);

    const { output } = await contract.query.getCreateAuctionFee(0, {});
    const createAuctionFee = output;

    return contract.tx.createAuction({ value: createAuctionFee, gasLimit: gasLimit }, startingBid, description, duration)
        .signAndSend(owner, result => {
            if (result.status.isFinalized) {
                sendRes(res, result);
            }
        });
};

async function createNftAuction(res, privateKey, startingBid, description, duration, gasLimit, nftContract, nftId) {
    const owner = keyring.createFromUri(privateKey);

    const { output } = await contract.query.getCreateAuctionFee(0, {});
    const createAuctionFee = output;

    return contract.tx.createAuction({ value: createAuctionFee, gasLimit: gasLimit }, startingBid, description, duration, nftContract, nftId)
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

    const { output } = await contract.query.getFinalizeFeeOf(0, {}, auctionId);
    const finalizeFee = output.toHuman().Ok;

    return contract.tx.cancelAuction({ value: finalizeFee, gasLimit: gasLimit }, auctionId)
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

async function lastTimestamp(res) {
    const now = await api.query.timestamp.now();
    res.status(200).send(now);
};


app.post('/createauction', async(req, res) => {
    await createAuction(
        res,
        req.query.privateKey,
        req.query.startingBid,
        req.query.description,
        req.query.duration,
        req.query.gasLimit
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

app.post('/createnftauction', async(req, res) => {
    await createNftAuction(
        res,
        req.query.privateKey,
        req.query.startingBid,
        req.query.description,
        req.query.duration,
        req.query.gasLimit,
        res.query.nftContract,
        res.query.nftId
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

app.post('/bid', async(req, res) => {
    console.log(req.query);
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

app.post('/finishauction', async(req, res) => {
    await finishAuction(
        res,
        req.query.privateKey,
        req.query.auctionId,
        req.query.gasLimit
    ).catch((e) => {
        res.status(400).send(e.toString());
    });
});

app.post('/cancelauction', async(req, res) => {
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

app.get('/lasttimestamp', async(req, res) => {
    await lastTimestamp(
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