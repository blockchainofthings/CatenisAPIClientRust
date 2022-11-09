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

    let issuance_id = "iWWKqTx6svmErabyCZKM";

    let result = ctn_client.retrieve_non_fungible_asset_issuance_progress(
        issuance_id,
    )?;

    if let Some(asset_id) = result.asset_id {
        println!("Reissuance for non-fungible asset: {}", asset_id);
    }

    println!("Percent processed: {}", result.progress.percent_processed.to_string());

    if result.progress.done {
        if let Some(true) = result.progress.success {
            // Get result
            let issuance_result = result.result.unwrap();

            if let Some(asset_id) = issuance_result.asset_id {
                println!("ID of newly created non-fungible asset: {}", asset_id);
            }

            println!("IDs of newly issued non-fungible tokens:: {:?}", issuance_result.nf_token_ids);
        } else {
            // Process error
            let error = result.progress.error.unwrap();

            println!("Asynchronous processing error: [{}] - {}", error.code, error.message);
        }
    } else {
        // Asynchronous processing not done yet. Continue pooling
    }

    Ok(())
}