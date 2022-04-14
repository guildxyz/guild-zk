use std::str::FromStr;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};

const DEV_TOKEN_ADDRESS: &str = "0x5cAf454Ba92e6F2c929DF14667Ee360eD9fD5b26";
const HOLDER_ACCOUNT: &str = "0x11270D33f9253574EBA0cf3eE7F3d6CeC94D093f";

// NOTE in order to run this example, add a mainnet infura link
// to your `.env` file in the cargo manifest dir, e.g.:
// ENDPOINT="https://mainnet.infura.io/v3/xyz..."

#[tokio::main]
async fn main() -> web3::Result<()> {
    let _ = env_logger::try_init();
    let infura = dotenv::var("ENDPOINT").expect("No such env var");
    let transport = web3::transports::Http::new(&infura)?;
    let web3 = web3::Web3::new(transport);

    let abi = include_bytes!("./dev_token_abi.json");
    let contract_address = Address::from_str(DEV_TOKEN_ADDRESS).expect("Failed to convert address");
    let holder_address = Address::from_str(HOLDER_ACCOUNT).expect("Failed to convert address");
    let contract = Contract::from_json(web3.eth(), contract_address, abi)
        .expect("Failed to generate contract interface");

    let result = contract.query(
        "balanceOf",
        (holder_address,),
        None,
        Options::default(),
        None,
    );
    let balance: U256 = result.await.expect("Failed to query balance");
    let result = contract.query("decimals", (), None, Options::default(), None);
    let decimals: u8 = result.await.expect("Failed to query decimals");
    let balance = balance.as_u128() as f64;
    println!(
        "balance: {:?} [DEV]",
        balance / 10.0f64.powi(decimals as i32)
    );

    Ok(())
}
