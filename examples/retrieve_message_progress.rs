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

    let cached_message_id = "hfHtyPCS68toB9FjA8rM";

    let result = ctn_client.retrieve_message_progress(
        cached_message_id,
    )?;

    println!("Number of bytes processed so far: {}", result.progress.bytes_processed);

    if result.progress.done {
        if let Some(true) = result.progress.success {
            // Get result
            println!("Asynchronous processing result: {:?}", result.result.unwrap());
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