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

    let token_id = "tDGQpGy627J6uAw4grYq";
    let transfer_id = "xuYnPMKQSBXi28wRaZpN";

    let result = ctn_client.retrieve_non_fungible_token_transfer_progress(
        token_id,
        transfer_id,
    )?;
    
    println!("Current data manipulation: {:?}", result.progress.data_manipulation);
    
    if result.progress.done {
        if let Some(true) = result.progress.success {
            // Display result
            println!("Non-fungible token successfully transferred");
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