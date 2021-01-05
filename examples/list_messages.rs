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

    let result = ctn_client.list_messages(
        Some(ListMessagesOptions {
            action: Some(MessageActionOption::Send),
            direction: Some(MessageDirectionOption::Inbound),
            from_devices: Some(vec![
                DeviceId {
                    id: String::from("dv3htgvK7hjnKx3617Re"),
                    is_prod_unique_id: None,
                },
            ]),
            to_devices: None,
            read_state: Some(MessageReadStateOption::Unread),
            start_date: Some("2018-01-01T00:00:00Z".into()),
            end_date: Some("2018-02-28T23:59:59Z".into()),
            limit: Some(200),
            skip: Some(0),
        }),
    )?;

    if result.msg_count > 0 {
        println!("Returned messages: {:?}", result.messages);

        if result.has_more {
            println!("Not all messages have been returned");
        }
    }

    Ok(())
}