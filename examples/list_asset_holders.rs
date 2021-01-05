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

    let asset_id = "aQjlzShmrnEZeeYBZihc";

    let result = ctn_client.list_asset_holders(
        asset_id,
        Some(200),
        Some(0),
    )?;

    for idx in 0..result.asset_holders.len() {
        let asset_holder = &result.asset_holders[idx];

        println!("Asset holder #{}:", idx + 1);
        println!(" - device holding an amount of the asset: {:?}", asset_holder.holder);
        println!(" - amount of asset currently held by device: {}", asset_holder.balance.total);
        println!(" - amount not yet confirmed: {}", asset_holder.balance.unconfirmed);
    }

    if result.has_more {
        println!("Not all asset holders have been returned");
    }

    Ok(())
}