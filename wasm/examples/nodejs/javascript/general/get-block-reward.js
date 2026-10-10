// @ts-ignore
globalThis.WebSocket = require("websocket").w3cwebsocket; // W3C WebSocket module shim

const kaspa = require("../../../../nodejs/kaspa");
const { RpcClient } = kaspa;
const { parseArgs, rpcUrl } = require("../utils");

kaspa.initConsolePanicHook();

const { networkId, encoding } = parseArgs();

(async () => {
  const rpc = new RpcClient({
    url: rpcUrl(networkId),
    encoding,
    networkId,
  });

  console.log("Resolving RPC endpoint...");
  await rpc.connect();
  console.log(`Connected to ${rpc.url}`);

  try {
    const { sink } = await rpc.getBlockDagInfo();
    console.log("Querying block reward for node sink:", sink);

    const rewardInfo = await rpc.getBlockRewardInfo({
      hash: sink,
    });

    console.log({ rewardInfo });
  } finally {
    await rpc.disconnect();
    console.log("bye!");
  }
})().catch((err) => {
  console.error("GetBlockRewardInfo example failed:", err);
  process.exitCode = 1;
});
