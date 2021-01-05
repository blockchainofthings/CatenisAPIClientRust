use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
    api::*,
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

    let target_device = DeviceId {
        id: String::from("dv3htgvK7hjnKx3617Re"),
        is_prod_unique_id: None,
    };

    let result = ctn_client.send_message(
        "This is only a test",
        target_device,
        Some(SendMessageOptions {
            encoding: Some(Encoding::UTF8),
            encrypt: Some(true),
            off_chain: Some(true),
            storage: Some(Storage::Auto),
            read_confirmation: Some(true),
            async_: None,
        }),
    )?;

    println!("ID of sent message: {}", result.message_id.unwrap());

    Ok(())
}