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

    let _result = ctn_client.set_permission_rights(
        PermissionEvent::ReceiveMsg,
        AllPermissionRightsUpdate {
            system: None,
            catenis_node: None,
            client: Some(PermissionRightsUpdate {
                allow: Some(vec![
                    String::from("self"),
                ]),
                deny: Some(vec![
                    String::from("cjNhuvGMUYoepFcRZadP"),
                ]),
                none: None,
            }),
            device: Some(DevicePermissionRightsUpdate {
                allow: Some(vec![
                    DeviceId {
                        id: String::from("dv3htgvK7hjnKx3617Re"),
                        is_prod_unique_id: None,
                    },
                    DeviceId {
                        id: String::from("XYZ0001"),
                        is_prod_unique_id: Some(true),
                    },
                ]),
                deny: None,
                none: None,
            }),
        },
    )?;

    println!("Permission rights successfully set");

    Ok(())
}