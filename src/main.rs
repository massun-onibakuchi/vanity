use std::env;
use std::time::Instant;

use ethers::utils;
use ethers::{
    core::rand::thread_rng,
    signers::{LocalWallet, Signer},
    utils::hex,
};
use rayon::prelude::*;
use regex::RegexSet;

fn main() {
    // read argments
    // explicitly annotate the type of args to specify that we want a vector of strings.
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    // the first value in the vector is the name of our binary
    // https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html#accepting-command-line-arguments
    // so, get the second and the third value here
    let starts_with = args.get(1);
    let ends_with = args.get(2);

    let mut regexs = vec![];
    if let Some(prefix) = starts_with {
        println!("{}", prefix);
        let pad_width = prefix.len() + prefix.len() % 2;
        hex::decode(format!("{:0>width$}", prefix, width = pad_width))
            .expect("invalid prefix hex provided");
        regexs.push(format!(r"^{}", prefix));
    }
    if let Some(suffix) = ends_with {
        let pad_width = suffix.len() + suffix.len() % 2;
        hex::decode(format!("{:0>width$}", suffix, width = pad_width))
            .expect("invalid suffix hex provided");
        regexs.push(format!(r"{}$", suffix));
    }

    assert!(
        regexs.iter().map(|p| p.len() - 1).sum::<usize>() <= 40,
        "vanity patterns length exceeded. cannot be more than 40 characters",
    );

    let regex = RegexSet::new(regexs).unwrap();

    println!("Starting to generate vanity address...");
    // measure time
    let timer = Instant::now();
    // generate random value and use it as seed of key
    // search address with matching patterns in parallel
    let wallet = std::iter::repeat_with(move || LocalWallet::new(&mut thread_rng()))
        .into_iter()
        .par_bridge()
        .find_any(|wallet| {
            // looking for wallet address
            let addr = hex::encode(wallet.address().to_fixed_bytes());
            regex.matches(&addr).into_iter().count() == regex.patterns().len()
        })
        .expect("failed to generate vanity wallet");

    // print result
    println!(
        "Successfully found vanity address in {} seconds.\nAddress: {}\nPrivate Key: 0x{}",
        timer.elapsed().as_secs(),
        utils::to_checksum(&wallet.address(), Some(1 as u8)),
        hex::encode(wallet.signer().to_bytes()),
    );
}
