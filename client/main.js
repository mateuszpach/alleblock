getCookie = function(sName) {
    var oCrumbles = document.cookie.split(';');
    for (var i = 0; i < oCrumbles.length; i++) {
        var oPair = oCrumbles[i].split('=');
        var sKey = decodeURIComponent(oPair[0].trim());
        var sValue = oPair.length > 1 ? oPair[1] : '';
        if (sKey == sName) {
            return decodeURIComponent(sValue);
        }
    }
    return '';
};

setCookie = function(sName, sValue, options) {
    //oDate.setYear(oDate.getFullYear()+1);
    var sCookie = encodeURIComponent(sName) + '=' + encodeURIComponent(sValue);

    // Shorthand: options === expires date
    if (options && options instanceof Date) {
        options = {
            expires: options
        };
    }
    // Longhand: options object
    if (options && typeof options == 'object') {
        if (options.expires) {
            sCookie += '; expires=' + options.expires.toGMTString();
        }
        if (options.path) {
            sCookie += '; path=' + options.path.toString();
        }
        if (options.domain) {
            sCookie += '; domain=' + options.domain.toString();
        }
        if (options.secure) {
            sCookie += '; secure';
        }
    }
    document.cookie = sCookie;
};

function setAccount() {
    const address = document.querySelector('input[name=address]').value;
    const key = document.querySelector('input[name=key]').value;
    setCookie('address', address);
    setCookie('key', key);
}

function showSavedAccount() {
    document.querySelector('input[name=address]').value = getCookie('address');
    document.querySelector('input[name=key]').value = getCookie('key');
}

const api = 'http://localhost:8080/';

async function fetchLastTimestamp() {
    return fetch(api + 'lasttimestamp').then(response => response.json());
}

function showLiveAuctions() {
    fetch(api + 'getauctions')
        .then(response => response.json())
        .then(async data => {
            const lastTimestamp = await fetchLastTimestamp();

            data.forEach(item => {
                if (item.auctionState != 'InProgress') {
                    return;
                }
                const row = document.createElement('tr');

                const id = document.createElement('td');
                const owner = document.createElement('td');
                const description = document.createElement('td');
                const started = document.createElement('td');
                const ends = document.createElement('td');
                const startingBid = document.createElement('td');
                const highestBid = document.createElement('td');
                const highestBidder = document.createElement('td');
                const actions = document.createElement('td');

                id.innerText = item.id;
                owner.innerText = item.owner.substring(0, 4) + ' ... ' + item.owner.slice(-4);
                description.innerText = item.description;
                started.innerText = item.creationDate;
                ends.innerText = item.finishDate;
                startingBid.innerText = item.startingBid;
                highestBid.innerText = item.highestBid;
                highestBidder.innerText = item.highestBidder.substring(0, 4) + ' ... ' + item.highestBidder.slice(-4);

                const address = getCookie('address');

                if (parseInt(item.finishDate.replace(/,/g, '')) > lastTimestamp) {
                    actions.innerHTML = '<button type="button" onClick="bid(' + item.id + ')">Bid</button>';
                    if (item.owner == address) {
                        actions.innerHTML += '<button type="button" onClick="cancel(' + item.id + ')">Cancel</button>';
                    }
                } else if (item.highestBidder == address || item.owner == address) {
                    actions.innerHTML = '<button type="button" onClick="finish(' + item.id + ')">Finish</button>';
                }

                row.appendChild(id);
                row.appendChild(owner);
                row.appendChild(description);
                row.appendChild(started);
                row.appendChild(ends);
                row.appendChild(startingBid);
                row.appendChild(highestBid);
                row.appendChild(highestBidder);
                row.appendChild(actions);

                document.querySelector('tbody').appendChild(row);
            });
        });
}


function showFinishedAuctions() {
    fetch(api + 'getauctions')
        .then(response => response.json())
        .then(async data => {
            const lastTimestamp = await fetchLastTimestamp();

            data.forEach(item => {
                if (item.auctionState == 'InProgress') {
                    return;
                }
                const row = document.createElement('tr');

                const id = document.createElement('td');
                const owner = document.createElement('td');
                const description = document.createElement('td');
                const started = document.createElement('td');
                const ends = document.createElement('td');
                const startingBid = document.createElement('td');
                const highestBid = document.createElement('td');
                const highestBidder = document.createElement('td');
                const state = document.createElement('td');

                id.innerText = item.id;
                owner.innerText = item.owner.substring(0, 4) + ' ... ' + item.owner.slice(-4);
                description.innerText = item.description;
                started.innerText = item.creationDate;
                ends.innerText = item.finishDate;
                startingBid.innerText = item.startingBid;
                highestBid.innerText = item.highestBid;
                highestBidder.innerText = item.highestBidder.substring(0, 4) + ' ... ' + item.highestBidder.slice(-4);
                state.innerText = item.auctionState;

                row.appendChild(id);
                row.appendChild(owner);
                row.appendChild(description);
                row.appendChild(started);
                row.appendChild(ends);
                row.appendChild(startingBid);
                row.appendChild(highestBid);
                row.appendChild(highestBidder);
                row.appendChild(state);

                document.querySelector('tbody').appendChild(row);
            });
        });
}


function showOwnedAuctions() {
    fetch(api + 'getauctions')
        .then(response => response.json())
        .then(async data => {
            const lastTimestamp = await fetchLastTimestamp();

            data.forEach(item => {
                const address = getCookie('address');

                if (item.owner != address) {
                    return;
                }
                const row = document.createElement('tr');

                const id = document.createElement('td');
                const description = document.createElement('td');
                const started = document.createElement('td');
                const ends = document.createElement('td');
                const startingBid = document.createElement('td');
                const highestBid = document.createElement('td');
                const highestBidder = document.createElement('td');
                const actions = document.createElement('td');
                const state = document.createElement('td');

                id.innerText = item.id;
                description.innerText = item.description;
                started.innerText = item.creationDate;
                ends.innerText = item.finishDate;
                startingBid.innerText = item.startingBid;
                highestBid.innerText = item.highestBid;
                highestBidder.innerText = item.highestBidder.substring(0, 4) + ' ... ' + item.highestBidder.slice(-4);
                state.innerText = item.auctionState;

                if (item.auctionState == 'InProgress') {
                    if (parseInt(item.finishDate.replace(/,/g, '')) > lastTimestamp) {
                        actions.innerHTML = '<button type="button" onClick="bid(' + item.id + ')">Bid</button>';
                        if (item.owner == address) {
                            actions.innerHTML += '<button type="button" onClick="cancel(' + item.id + ')">Cancel</button>';
                        }
                    } else if (item.highestBidder == address || item.owner == address) {
                        actions.innerHTML = '<button type="button" onClick="finish(' + item.id + ')">Finish</button>';
                    }
                }

                row.appendChild(id);
                row.appendChild(description);
                row.appendChild(started);
                row.appendChild(ends);
                row.appendChild(startingBid);
                row.appendChild(highestBid);
                row.appendChild(highestBidder);
                row.appendChild(state);
                row.appendChild(actions);

                document.querySelector('tbody').appendChild(row);
            });
        });
}

function bid(auctionId) {
    let bidPrice = prompt("Please enter bid price:");
    if (bidPrice == null || bidPrice == undefined) {
        return;
    }
    let gasLimit = prompt("Please enter gas limit:");
    if (gasLimit == null || gasLimit == undefined) {
        return;
    }

    fetch(api + 'bid?' + new URLSearchParams({
            privateKey: getCookie('key'),
            auctionId: auctionId,
            bidPrice: bidPrice,
            gasLimit: gasLimit
        }).toString(), {
            method: 'POST'
        })
        .then(res => {
            if (!res.ok) {
                return res.text().then(text => { throw new Error(text) });
            } else {
                return res.json();
            }
        })
        .then(data => {
            alert('Action successful.\n\n' + JSON.stringify(data));
            window.location.reload();
        })
        .catch(err => {
            alert('Action failed.\n\n' + err);
            window.location.reload();
        });
}

function cancel(auctionId) {
    let gasLimit = prompt("Please enter gas limit:");
    if (gasLimit == null || gasLimit == undefined) {
        return;
    }

    fetch(api + 'cancelauction?' + new URLSearchParams({
            privateKey: getCookie('key'),
            auctionId: auctionId,
            gasLimit: gasLimit
        }).toString(), {
            method: 'POST'
        })
        .then(res => {
            if (!res.ok) {
                return res.text().then(text => { throw new Error(text) });
            } else {
                return res.json();
            }
        })
        .then(data => {
            alert('Action successful.\n\n' + JSON.stringify(data));
            window.location.reload();
        })
        .catch(err => {
            alert('Action failed.\n\n' + err);
            window.location.reload();
        });
}


function finish(auctionId) {
    let gasLimit = prompt("Please enter gas limit:");
    if (gasLimit == null || gasLimit == undefined) {
        return;
    }

    fetch(api + 'finishauction?' + new URLSearchParams({
            privateKey: getCookie('key'),
            auctionId: auctionId,
            gasLimit: gasLimit
        }).toString(), {
            method: 'POST'
        })
        .then(res => {
            if (!res.ok) {
                return res.text().then(text => { throw new Error(text) });
            } else {
                return res.json();
            }
        })
        .then(data => {
            alert('Action successful.\n\n' + JSON.stringify(data));
            window.location.reload();
        })
        .catch(err => {
            alert('Action failed.\n\n' + err);
            window.location.reload();
        });
}

function create() {
    const startingBid = document.querySelector('input[name=startingBid]').value;
    const description = document.querySelector('input[name=description]').value;
    const duration = document.querySelector('input[name=duration]').value;
    const gasLimit = document.querySelector('input[name=gasLimit]').value;

    fetch(api + 'createauction?' + new URLSearchParams({
            privateKey: getCookie('key'),
            startingBid: startingBid,
            description: description,
            duration: duration,
            gasLimit: gasLimit
        }).toString(), {
            method: 'POST'
        })
        .then(res => {
            if (!res.ok) {
                return res.text().then(text => { throw new Error(text) });
            } else {
                return res.json();
            }
        })
        .then(data => {
            alert('Action successful.\n\n' + JSON.stringify(data));
            window.location = './owned_auctions.html';
        })
        .catch(err => {
            alert('Action failed.\n\n' + err);
        });
}

function createNft() {
    const startingBid = document.querySelector('input[name=startingBid]').value;
    const description = document.querySelector('input[name=description]').value;
    const duration = document.querySelector('input[name=duration]').value;
    const gasLimit = document.querySelector('input[name=gasLimit]').value;
    const nftContract = document.querySelector('input[name=nftContract]').value;
    const nftId = document.querySelector('input[name=nftId]').value;

    fetch(api + 'createnftauction?' + new URLSearchParams({
            privateKey: getCookie('key'),
            startingBid: startingBid,
            description: description,
            duration: duration,
            gasLimit: gasLimit,
            nftContract: nftContract,
            nftId: nftId
        }).toString(), {
            method: 'POST'
        })
        .then(res => {
            if (!res.ok) {
                return res.text().then(text => { throw new Error(text) });
            } else {
                return res.json();
            }
        })
        .then(data => {
            alert('Action successful.\n\n' + JSON.stringify(data));
            window.location = './owned_auctions.html';
        })
        .catch(err => {
            alert('Action failed.\n\n' + err);
        });
}