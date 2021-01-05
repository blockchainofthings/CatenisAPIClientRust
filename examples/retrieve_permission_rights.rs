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

    let result = ctn_client.retrieve_permission_rights(
        PermissionEvent::ReceiveMsg,
    )?;

    println!("Default (system) permission right: {:?}", result.system);

    if let Some(rights_setting) = result.catenis_node {
        if let Some(catenis_node_idxs) = rights_setting.allow {
            println!(
                "Index of Catenis nodes with 'allow' permission right: {:?}",
                catenis_node_idxs
            );
        }

        if let Some(catenis_node_idxs) = rights_setting.deny {
            println!(
                "Index of Catenis nodes with 'deny' permission right: {:?}",
                catenis_node_idxs
            );
        }
    }

    if let Some(rights_setting) = result.client {
        if let Some(client_ids) = rights_setting.allow {
            println!("ID of clients with 'allow' permission right: {:?}", client_ids);
        }

        if let Some(client_ids) = rights_setting.deny {
            println!("ID of clients with 'deny' permission right: {:?}", client_ids);
        }
    }

    if let Some(rights_setting) = result.device {
        if let Some(devices) = rights_setting.allow {
            println!("Devices with 'allow' permission right: {:?}", devices);
        }

        if let Some(devices) = rights_setting.deny {
            println!("Devices with 'deny' permission right: {:?}", devices);
        }
    }

    Ok(())
}