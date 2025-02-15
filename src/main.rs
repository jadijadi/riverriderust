use alloy::{
    network::EthereumWallet,
    primitives::{address, keccak256, Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolValue,
};
use dotenv::dotenv;
use std::{
    env::{self, var},
    fs::File,
    io::{stdout, Read},
    str::FromStr,
};
use stout_ext::StdoutExt;

use crossterm::{
    cursor::{Hide, Show},
    terminal::{disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand,
};

mod canvas;
mod drawable;
mod entities;
mod events;
mod stout_ext;
mod world;

use events::*;
use world::*;

const CONTRACT_ADDRESS: Address = address!("FEF49B2E79Ee1d04EbF792Eb3060049Ff05d59BD");
const RPC_URL: &str = "https://mainnet.base.org";
sol!(
    #[sol(rpc)]
    "./contract/River.sol",
);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // init the screen
    let mut sc = stdout();
    let (maxc, maxl) = size().unwrap();
    sc.execute(Hide)?;
    enable_raw_mode()?;

    // init the world
    let slowness = 60;
    let mut world = World::new(maxc, maxl);

    // show welcoming banner
    world.welcome_screen(&mut sc)?;

    // Main game loop
    // - Events
    // - Physics
    // - Drawing
    world.game_loop(&mut sc, slowness)?;

    // game is finished
    world.clear_screen(&mut sc)?;
    world.goodbye_screen(&mut sc)?;

    // Instance
    let wallet =
        EthereumWallet::from(PrivateKeySigner::from_str(&var("PRIVATE_KEY").unwrap()).unwrap());
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(RPC_URL.parse().unwrap());
    let river_contract = River::new(CONTRACT_ADDRESS, provider);

    let current_binary = File::open(env::current_exe()?)?;
    let binary_hash = keccak256(
        current_binary
            .bytes()
            .map(|x| x.unwrap())
            .collect::<Vec<_>>(),
    );

    let packed = SolValue::abi_encode_packed(&(binary_hash, U256::from(world.player.score)));
    river_contract
        .giveTokens(U256::from(world.player.score), keccak256(packed))
        .send()
        .await
        .ok();

    sc.clear_all()?.execute(Show)?;
    disable_raw_mode()?;
    Ok(())
}
