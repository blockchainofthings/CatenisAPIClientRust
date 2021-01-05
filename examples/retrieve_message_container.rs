use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
};

fn main() -> Result<()> {
    let device_credentials = (
        "dnN3Ea43bhMTHtTvpytS",
        concat!(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
        ),
    ).into();

    let mut ctn_client = CatenisClient::new_with_options(
        Some(device_credentials),
        &[
            ClientOptions::Environment(Environment::Sandbox),
        ],
    )?;

    let message_id = "oDWPuD5kjCsEiNEEWwrW";

    let result = ctn_client.retrieve_message_container(
        message_id,
    )?;

    if let Some(off_chain) = result.off_chain {
        println!("IPFS CID of Catenis off-chain message envelope: {}", off_chain.cid);
    }

    if let Some(blockchain) = result.blockchain {
        println!("ID of blockchain transaction containing the message: {}", blockchain.txid);
    }

    if let Some(external_storage) = result.external_storage {
        println!("IPFS reference to message: {}", external_storage.ipfs);
    }

    Ok(())
}