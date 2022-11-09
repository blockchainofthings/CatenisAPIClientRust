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

    let result = ctn_client.issue_non_fungible_asset(
        NFAssetIssuanceInfoOrContToken::IssuanceInfo(NonFungibleAssetIssuanceInfo {
            asset_info: Some(NewNonFungibleAssetInfo {
                name: String::from("NFA 1"),
                description: Some(String::from("Non-fungible asset #1 for testing")),
                can_reissue: true
            }),
            encrypt_nft_contents: None,
            holding_devices: None,
            async_: None,
        }),
        Some(vec![
            NewNonFungibleTokenInfo {
                metadata: Some(NewNonFungibleTokenMetadata {
                    name: String::from("NFA1 NFT 1"),
                    description: Some(String::from("First token of non-fungible asset #1")),
                    custom: None,
                }),
                contents: Some(NewNonFungibleTokenContents {
                    data: String::from("Contents of first token of non-fungible asset #1"),
                    encoding: Encoding::UTF8
                }),
            },
            NewNonFungibleTokenInfo {
                metadata: Some(NewNonFungibleTokenMetadata {
                    name: String::from("NFA1 NFT 2"),
                    description: Some(String::from("Second token of non-fungible asset #1")),
                    custom: None,
                }),
                contents: Some(NewNonFungibleTokenContents {
                    data: String::from("Contents of second token of non-fungible asset #1"),
                    encoding: Encoding::UTF8
                }),
            },
        ]),
        Some(true)
    )?;

    println!("ID of newly created non-fungible asset: {}", result.asset_id.unwrap());
    println!("IDs of newly issued non-fungible tokens: {:?}", result.nf_token_ids.unwrap());

    Ok(())
}