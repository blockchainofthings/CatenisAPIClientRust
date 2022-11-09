use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
    api::AssetIssuanceEventEntry,
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

    let asset_id = "aQjlzShmrnEZeeYBZihc";

    let result = ctn_client.retrieve_asset_issuance_history(
        asset_id,
        Some("2017-01-01T00:00:00Z".into()),
        None,
        Some(200),
        Some(0),
    )?;

    for idx in 0..result.issuance_events.len() {
        let issuance_event = &result.issuance_events[idx];

        println!("Issuance event #{}:", idx + 1);

        match issuance_event {
            AssetIssuanceEventEntry::Regular(regular_asset_event) => {
                println!(" - asset amount: {}", regular_asset_event.amount);
                println!(" - device to which issued amount had been assigned: {:?}", regular_asset_event.holding_device);
                println!(" - date of issuance: {}", regular_asset_event.date);
            }
            AssetIssuanceEventEntry::NonFungible(non_fungible_asset_event) => {
                println!(" - IDs of issued non-fungible tokens: {:?}", non_fungible_asset_event.nf_token_ids);
                println!(
                    " - devices to which issued non-fungible tokens have been assigned: {:?}",
                    non_fungible_asset_event.holding_devices
                );
                println!(" - date of issuance: {}", non_fungible_asset_event.date);
            }
        }
    }

    if result.has_more {
        println!("Not all asset issuance events have been returned");
    }

    Ok(())
}