// @ts-ignore
globalThis.WebSocket = require('websocket').w3cwebsocket; // W3C WebSocket module shim

const kaspa = require('../../../../nodejs/kaspa');
const { parseArgs } = require("../utils");
const { RpcClient } = kaspa;
const { rpcUrl } = require("../utils");

kaspa.initConsolePanicHook();

const {
    networkId,
    encoding,
} = parseArgs();

(async () => {

    const rpc = new RpcClient({
        url: rpcUrl(networkId),
        encoding,
        networkId
    });
    console.log("Using the explicitly configured local SLUG RPC endpoint.");
    await rpc.connect();
    console.log("Connected to", rpc.url);
    console.log("RPC", rpc);

    // console.log(`Connecting to ${rpc.url}`)

    const info = await rpc.getBlockDagInfo();
    console.log("GetBlockDagInfo response:", info);

    await rpc.disconnect();
})();
