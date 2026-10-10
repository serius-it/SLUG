const fs = require('fs');
const path = require('path');
const nodeUtil = require('node:util');
const { parseArgs: nodeParseArgs, } = nodeUtil;

const {
    Address,
    Encoding,
    NetworkId,
    Mnemonic,
} = require('../../../nodejs/kaspa');

/**
 * Helper function to parse command line arguments for running the scripts
 * @param options Additional options to configure the parsing, such as additional arguments for the script and additional help output to go with it
 * @returns {{address: Address, tokens: any, networkId: (NetworkId), encoding: (Encoding)}}
 */
function parseArgs(options = {
    additionalParseArgs: {},
    additionalHelpOutput: '',
}) {
    const script = path.basename(process.argv[1]);
    let args = process.argv.slice(2);
    const {
        values,
        positionals,
        tokens,
    } = nodeParseArgs({
        args, options: {
            ...options.additionalParseArgs,
            help: {
                type: 'boolean',
            },
            json: {
                type: 'boolean',
            },
            address: {
                type: 'string',
            },
            network: {
                type: 'string',
            },
            encoding: {
                type: 'string',
            },
        }, tokens: true, allowPositionals: true
    });
    if (values.help) {
        console.log(`Usage: node ${script} [address] [slug-mainnet|slug-testnet-1|slug-devnet|slug-simnet] [--address <address>] [--network <slug-network>] [--encoding <borsh|json>] ${options.additionalHelpOutput}`);
        process.exit(0);
    }

    let config = null;
    // TODO load address from config file if no argument is specified
    let configFile = path.join(__dirname, '../../data/config.json');
    if (fs.existsSync(configFile)) {
        config = JSON.parse(fs.readFileSync(configFile, "utf8"));
    } else {
        console.error("Please create a config file by running 'node init' in the 'examples/' folder");
        process.exit(0);
    }

    const addressRegex = new RegExp(/(slug|slugtest|slugdev|slugsim):\S+/i);
    const addressArg = values.address ?? positionals.find((positional) => addressRegex.test(positional)) ?? null;
    const address = addressArg === null ? null : new Address(addressArg);

    const networkArg = values.network ?? positionals.find((positional) => positional.match(/^slug-(mainnet|testnet-1|devnet|simnet)$/)) ?? config.networkId ?? null;
    if (!networkArg) {
        console.error('Network id must be specified: --network=slug-(mainnet|testnet-1|devnet|simnet)');
        process.exit(1);
    }
    const networkId = new NetworkId(networkArg);

    const encodingArg = values.encoding ?? positionals.find((positional) => positional.match(/^(borsh|json)$/)) ?? null;
    let encoding = Encoding.Borsh;
    if (encodingArg == "json") {
        encoding = Encoding.SerdeJson;
    }

    return {
        address,
        networkId,
        encoding,
        tokens,
    };
}

function rpcUrl(networkId) {
    const network = networkId.toString().replace(/^slug-/, '');
    const ports = {
        mainnet: 19112,
        'testnet-1': 19212,
        devnet: 19312,
        simnet: 19412,
    };
    const port = ports[network];
    if (!port) {
        throw new Error(`Unsupported SLUG network: ${networkId}`);
    }
    return `ws://127.0.0.1:${port}`;
}

module.exports = {
    parseArgs,
    rpcUrl,
};
