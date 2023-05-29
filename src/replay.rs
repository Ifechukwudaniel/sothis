use crate::RpcConnection;
use crate::rpc::rpc::BlockResult;
//use crate::rpc::rpc::Transaction;

// To replay blocks we:
// 1) Make sure that the replay rpc block is equal to `block`
// 2) Set the `evm_autoMine` mode to create blocks
// 3) Set the `evm_set_interval_mining` to something ridiculously high.
// 4) Get transaction hashes from block
// 5) Get transactions from hashes, `eth_sendTransaction` that to the mempool
// 6) Loop for all transactions in a block
// 7) Set next block timestamp
// 8) `evm_mine` the block
pub async fn replay_blocks(
    historic_rpc: RpcConnection,
    replay_rpc: RpcConnection,
    until: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // make sure that both rpcs have the same chainid to satisfy the replay thingy
    let historical_chainid = historic_rpc.chain_id().await?;
    let replay_chainid = replay_rpc.chain_id().await?;

    if historical_chainid != replay_chainid {
        return Err("Chain IDs don't match".into());
    }

    // set automine to false
    replay_rpc.evm_set_automine(false).await?;
    // set insanely high interval for the blocks
    replay_rpc.evm_set_interval_mining(std::u32::MAX.into()).await?;

    // get block mumber of replay node
    let mut replay_block = replay_rpc.block_number().await?;
    while until != replay_block {
        // get block from historical node
        let historical_block = historic_rpc.get_block_by_number(replay_block.clone()).await?;

        // get transaction hashes from block
        let historical_block: BlockResult = serde_json::from_str(&historical_block)?;
        let historical_txs = historical_block.transactions;

        // send transactions to mempool
        for tx in historical_txs {
            replay_rpc.send_transaction(tx).await?;
        }

        // set next block timestamp
        replay_rpc.evm_set_next_block_timestamp(
            historical_block.timestamp.parse::<u64>()?,
        ).await?;

        println!("Successfully replayed block {}", &replay_block);

        // get next block
        replay_block = replay_rpc.block_number().await?;

        // mine the block
        replay_rpc.evm_mine().await?;    
    }
    println!("Done replaying blocks");
    Ok(())
}
