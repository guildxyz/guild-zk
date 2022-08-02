use agora_zkp_backend::balancy::*;
use agora_zkp_backend::config::get_test_config;
use std::io::{self, Write};

fn get_test_balancy() -> BalancyClient {
    let config = get_test_config();
    let balancy_client = BalancyClient::new(
        config.url_balancy,
        config.apikey_balancy,
        config.url_pubkey,
        config.apikey_pubkey,
        180,
    );
    balancy_client
}

#[tokio::test]
async fn test_get_pubkeys() {
    let config = get_test_config();
    if config.url_pubkey == "" {
        write!(
            &mut io::stdout(),
            "WARNING-test_get_pubkeys- no test url set, skipping test\n"
        )
        .ok();
        return;
    }
    let balancy_client = get_test_balancy();

    let addresses = vec![
        String::from("0x646dB8ffC21e7ddc2B6327448dd9Fa560Df41087"),
        String::from("0x64BDCCd18f59997404C90d729f5009FFd4C85e17"),
    ];
    let mut want = vec![
            String::from("04ee2ab4bf0e5e48a9e360a646609d52f59918b4f0861cbaf1968335aa2c0fdd8643213eb807174bd7a13d20d3f46710dd64d9860084d7cd5533e88cf45cfe8e26"),
            String::from("043298e36b5d63ae4341f9487c14c4dd89a9240d43e8b3f59cab476551bca27e7f417fed52dee4bb54f7e7ec76d167bd148ff8c47e809269d9c1e488cdba7051a0"),
        ];

    let mut got = balancy_client
        .get_pubkeys(addresses)
        .await
        .expect("balancy /pubkey request failed");
    want.sort();
    got.sort();
    assert_eq!(got, want);
}

#[tokio::test]
async fn test_get_xyz_holders() {
    let config = get_test_config();
    if config.url_balancy == "" {
        write!(
            &mut io::stdout(),
            "WARNING-test_get_xyz_holders- no test url set, skipping test\n"
        )
        .ok();
        return;
    }
    let balancy_client = get_test_balancy();

    let req_body = ReqXyzHolders {
        logic: String::from("AND"),
        limit: 0,
        offset: 0,
        requirements: vec![XyzHoldersRequirement {
            token_address: String::from("0xCda2f16C6Aa895D533506B426AFF827b709c87F5"),
            amount: String::from("100000000000000000000000"),
        }],
    };
    let mut got = balancy_client.get_xyz_holders_addresses(req_body)
        .await
        .expect("balancy /xyzHolders request failed");
    let mut want = vec![
        String::from("0x996Ed16B7a5FAfB99c6722101e083E455882D8B6"),
        String::from("0x299A299A22F8C7397d9DB3702439069d951AeA74"),
        String::from("0x621A78F100aab6a8e9f388E135E7BE42efA1e29d"),
        String::from("0xC77aab3c6D7dAb46248F3CC3033C856171878BD5"),
        String::from("0x4D7324471e0e4fa908E5573c5f0A4E1CcBB8aD8B"),
    ];
    got.sort();
    want.sort();
    assert_eq!(got, want);
}
