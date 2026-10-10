// @ts-ignore
globalThis.WebSocket = require('websocket').w3cwebsocket; // W3C WebSocket module shim

const kaspa = require('../../../../nodejs/kaspa');
const { parseArgs, rpcUrl } = require("../utils");
const { RpcClient } = kaspa;

kaspa.initConsolePanicHook();

const {
    networkId,
    encoding,
} = parseArgs();

(async () => {

    const rpc = new RpcClient({ url: rpcUrl(networkId), encoding, networkId });
    console.log(`Resolving RPC endpoint...`);
    await rpc.connect();
    console.log(`Connecting to ${rpc.url}`)

    const info = await rpc.getBalancesByAddresses({ addresses : ["slug:qpamkvhgh0kzx50gwvvp5xs8ktmqutcy3dfs9dc3w7lm9rq0zs76vwpkn7d08"]});
    // const info = await rpc.getBalancesByAddresses(["slug:qpamkvhgh0kzx50gwvvp5xs8ktmqutcy3dfs9dc3w7lm9rq0zs76vwpkn7d08"]);
    console.log("GetBalancesByAddresses response:", info);

    await rpc.disconnect();
    console.log("bye!");
})();
