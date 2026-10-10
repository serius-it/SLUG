// @ts-ignore
globalThis.WebSocket = require("websocket").w3cwebsocket; // W3C WebSocket module shim

const kaspa = require("../../../../nodejs/kaspa");
const { RpcClient, Encoding } = kaspa;

kaspa.initConsolePanicHook();

(async () => {
  const rpc = new RpcClient({
    url: "ws://127.0.0.1:19112",
    encoding: Encoding.Borsh,
    networkId: "slug-mainnet",
  });
  console.log(`Resolving RPC endpoint...`);
  await rpc.connect();
  console.log(`Connecting to ${rpc.url}`);

  const response = await rpc.getMempoolEntries({
    includeOrphanPool: true,
    filterTransactionPool: false,
  });
  console.log(`${response.mempoolEntries.length} transactions in the mempool`);
})();
