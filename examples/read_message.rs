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

    let message_id = "oDWPuD5kjCsEiNEEWwrW";

    let result = ctn_client.read_message(
        message_id,
        Some(ReadMessageOptions {
            encoding: Some(Encoding::UTF8),
            continuation_token: None,
            data_chunk_size: None,
            async_: None,
        }),
    )?;

    println!("Read message: {}", result.msg_data.unwrap());

    let msg_info = result.msg_info.unwrap();

    if msg_info.action == RecordMessageAction::Send {
        println!("Message sent from: {:?}", msg_info.from.unwrap());
    }

    Ok(())
}