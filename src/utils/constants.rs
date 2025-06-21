use std::collections::HashMap;
use std::path::PathBuf;

pub fn wallet_file_path() -> PathBuf {
    dirs::data_local_dir()
        .expect("Failed to get data directory")
        .join("rootstock-wallet")
        .join("rootstock-wallet.json")
}

pub const METHOD_TYPES: &str = "read";

pub const ALLOWED_BRIDGE_METHODS: &[(&str, &[&str])] = &[
    (
        "read",
        &[
            "getBtcBlockchainBestChainHeight",
            "getStateForBtcReleaseClient",
            "getStateForDebugging",
            "getBtcBlockchainInitialBlockHeight",
            "getBtcBlockchainBlockHashAtDepth",
            "getBtcTxHashProcessedHeight",
            "isBtcTxHashAlreadyProcessed",
            "getFederationAddress",
            "getFederationSize",
            "getFederationThreshold",
            "getFederatorPublicKey",
            "getFederatorPublicKeyOfType",
            "getFederationCreationTime",
            "getFederationCreationBlockNumber",
            "getRetiringFederationAddress",
            "getRetiringFederationSize",
            "getRetiringFederationThreshold",
            "getRetiringFederatorPublicKeyOfType",
            "getRetiringFederationCreationTime",
            "getRetiringFederationCreationBlockNumber",
            "getPendingFederationHash",
            "getPendingFederationSize",
            "getPendingFederatorPublicKeyOfType",
            "getFeePerKb",
            "getMinimumLockTxValue",
            "getBtcTransactionConfirmations",
            "getLockingCap",
            "hasBtcBlockCoinbaseTransactionInformation",
            "getActiveFederationCreationBlockHeight",
            "getBtcBlockchainBestBlockHeader",
            "getBtcBlockchainBlockHeaderByHash",
            "getBtcBlockchainBlockHeaderByHeight",
            "getBtcBlockchainParentBlockHeaderByHash",
            "getEstimatedFeesForNextPegOutEvent",
            "getNextPegoutCreationBlockNumber",
            "getQueuedPegoutsCount",
            "getActivePowpegRedeemScript",
        ],
    ),
    (
        "write",
        &[
            "registerBtcTransaction",
            "registerBtcCoinbaseTransaction",
            "receiveHeader",
        ],
    ),
];
