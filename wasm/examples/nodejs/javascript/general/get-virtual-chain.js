// @ts-ignore
globalThis.WebSocket = require("websocket").w3cwebsocket; // W3C WebSocket module shim

const kaspa = require("../../../../nodejs/kaspa");
const { parseArgs, rpcUrl } = require("../utils");
const { RpcClient } = kaspa;

kaspa.initConsolePanicHook();

const { networkId, encoding } = parseArgs();

(async () => {
  const rpc = new RpcClient({
    url: rpcUrl(networkId),
    encoding,
    networkId,
  });
  console.log(`Resolving RPC endpoint...`);
  await rpc.connect();
  console.log(`Connecting to ${rpc.url}`);

  console.log("Getting known block hash from node...");

  const info = await rpc.getBlockDagInfo();
  const sinkHash = info.sink;

  console.log(`Found sink hash: ${sinkHash}`);

  console.log("Getting virtual chain from sink hash...");

  // 1. wait for the sink hash to be confirmed by 30 blocks
  const virtualChainFromBlockResponseOne = await rpc.getVirtualChainFromBlock({
    startHash: sinkHash,
    includeAcceptedTransactionIds: true,
    minConfirmationCount: 30,
  });
  console.log({ virtualChainFromBlockResponseOne });

  // 2. do not specify a minConfirmationCount, so the virtual chain will be returned from the sink hash
  //    without waiting for any confirmations
  const virtualChainFromBlockResponseTwo = await rpc.getVirtualChainFromBlock({
    startHash: sinkHash,
    includeAcceptedTransactionIds: true,
  });
  console.log({ virtualChainFromBlockResponseTwo });

  await rpc.disconnect();
  console.log("bye!");
})();
