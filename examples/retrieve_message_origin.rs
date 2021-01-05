use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
};

fn main() -> Result<()> {
    let ctn_client = CatenisClient::new_with_options(
        None,
        &[
            ClientOptions::Environment(Environment::Sandbox),
        ],
    )?;

    let message_id = "oDWPuD5kjCsEiNEEWwrW";

    let result = ctn_client.retrieve_message_origin(
        message_id,
        Some("Any text to be signed"),
    )?;

    if let Some(tx) = result.tx {
        println!("Catenis message transaction info: {:?}", tx);
    }

    if let Some(off_chain_msg_env) = result.off_chain_msg_envelope {
        println!("Off-chain message envelope info: {:?}", off_chain_msg_env);
    }

    if let Some(proof) = result.proof {
        println!("Origin proof info: {:?}", proof);
    }

    Ok(())
}