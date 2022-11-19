use bindings::i_uniswap_v3_pool::IUniswapV3Pool;
use bindings::uniswap_v3_factory::UniswapV3Factory;
use ethers::abi::Address;
use ethers::prelude::*;
use ethers::providers::Provider;
use num_bigfloat::BigFloat;
use std::sync::Arc;

use crate::tokens::{self, get_tokens, Token};

// pub const two: BigFloat = 2.0.into();
// pub const ten: BigFloat = 10.0.into();
pub fn convert(q64_96: U256) -> BigFloat {
    let least_sig = q64_96.0[0];
    let second_sig = q64_96.0[1];
    let third_sig = q64_96.0[2];
    let most_sig = q64_96.0[3];

    let bf2 = BigFloat::from(2);
    let bf64 = BigFloat::from(64);
    let bf128 = BigFloat::from(128);
    let bf192 = BigFloat::from(192);
    let bf96 = BigFloat::from(96);

    ((BigFloat::from(most_sig) * bf2.pow(&bf192))
        + (BigFloat::from(third_sig) * bf2.pow(&bf128))
        + (BigFloat::from(second_sig) * bf2.pow(&bf64))
        + BigFloat::from(least_sig))
        / bf2.pow(&bf96)
}
pub async fn get_provider() -> Arc<Provider<Http>> {
    let provider =
        Provider::try_from("https://eth-mainnet.g.alchemy.com/v2/I93POQk49QE9O-NuOz7nj7sbiluW76it")
            .unwrap();
    Arc::new(provider)
    //https://eth-mainnet.g.alchemy.com/v2/I93POQk49QE9O-NuOz7nj7sbiluW76it
}
pub async fn get_uniswapv3_factory(
    provider: Arc<Provider<Http>>,
) -> UniswapV3Factory<Provider<Http>> {
    let uniswap_v3_factory_address = "0x1F98431c8aD98523631AE4a59f267346ea31F984"
        .parse::<Address>()
        .unwrap();
    UniswapV3Factory::new(uniswap_v3_factory_address, provider.clone())
}
pub async fn get_pool_from_uniswap(
    token_0: Address,
    token_1: Address,
    factory: UniswapV3Factory<Provider<Http>>,
) -> Vec<Address> {
    // BP 10000, 3000, 500, 100
    let pool_500 = factory
        .get_pool(token_0, token_1, 500)
        .call()
        .await
        .unwrap();
    let pool_100 = factory
        .get_pool(token_0, token_1, 100)
        .call()
        .await
        .unwrap();
    let pool_3000 = factory
        .get_pool(token_0, token_1, 3000)
        .call()
        .await
        .unwrap();
    let pool_10000 = factory
        .get_pool(token_0, token_1, 10000)
        .call()
        .await
        .unwrap();
    vec![pool_100, pool_500, pool_3000, pool_10000]
}
pub async fn get_pool_objects(
    addresses: Vec<Address>,
    provider: Arc<Provider<Http>>,
) -> Vec<IUniswapV3Pool<Provider<Http>>> {
    let mut vec: Vec<IUniswapV3Pool<Provider<Http>>> = vec![];
    for address in addresses {
        let uniswap_pool = &mut vec![IUniswapV3Pool::new(address, provider.clone())];
        vec.append(uniswap_pool);
    }
    vec
}
// pub async fn multi_thread_listener(pools: Vec<IUniswapV3Pool<Provider<Http>>>) {
//     for pool in pools {
//         // tokio::spawn(future)
//         let thread = thread::spawn(move || {
//             monitor_pool(&pool);
//         });
//     }
// }

pub async fn monitor_pool(pool: &IUniswapV3Pool<Provider<Http>>) {
    let two: BigFloat = 2.0.into();
    let ten: BigFloat = 10.0.into();
    let swap_events = pool.swap_filter();
    let mut swap_stream = swap_events.stream().await.unwrap();
    while let Some(Ok(event)) = swap_stream.next().await {
        println!("------------New Swap------------");
        println!("From pool {:#?}", pool.address());
        println!(
            "Sender: {:#?}, Recipient: {:#?}",
            event.sender, event.recipient
        ); // H160s
        println!("amount_0 {:#?}", event.amount_0); // I256
        println!("amount_1 {:#?}", event.amount_1); // I256
        println!("liquidity {:#?}", event.liquidity); // u128
        println!("tick {:#?}", event.tick); // i32

        // https://docs.uniswap.org/sdk/guides/fetching-prices
        let tokens = get_tokens();
        let diff_decimals: BigFloat =
            (tokens.get("ETH").unwrap().base_units - tokens.get("USDC").unwrap().base_units).into();
        let one: BigFloat = 1.into();
        println!(
            "price {:#?}",
            one.div(
                &convert(event.sqrt_price_x96)
                    .pow(&two)
                    .div(&ten.pow(&diff_decimals))
            )
            .to_string()
        )
    }
}