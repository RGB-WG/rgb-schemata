use amplify::hex::FromHex;
use bp::dbc::Method;
use bp::{Outpoint, Txid};
use ifaces::{Rgb25, Rgb25Wrapper};
use rgbstd::containers::{ConsignmentExt, FileContent, Kit};
use rgbstd::interface::{FilterIncludeAll, FungibleAllocation};
use rgbstd::invoice::Precision;
use rgbstd::persistence::{MemContract, Stock};
use rgbstd::XWitnessId;
use schemata::dumb::NoResolver;
use schemata::CollectibleFungibleAsset;

#[rustfmt::skip]
fn main() {
    let beneficiary_txid =
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = Outpoint::new(beneficiary_txid, 1);

    let kit = Kit::load_file("schemata/CollectibleFungibleAsset.rgb").unwrap().validate().unwrap();

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::in_memory();
    stock.import_kit(kit).expect("invalid issuer kit");

    #[allow(clippy::inconsistent_digit_grouping)]
    let contract = Rgb25Wrapper::<MemContract>::testnet::<CollectibleFungibleAsset>(Method::TapretFirst, "ssi:anonymous", "Test asset", Precision::CentiMicro)
        .expect("invalid contract data")
        .allocate(beneficiary, 1_000_000_000_00u64.into())
        .expect("invalid allocations")
        .issue_contract()
        .expect("invalid contract data");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract.save_file("test/rgb25-example.rgb").expect("unable to save contract");
    contract.save_armored("test/rgb25-example.rgba").expect("unable to save armored contract");

    stock.import_contract(contract, NoResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_class::<Rgb25>(contract_id).unwrap();
    let allocations = contract.allocations(&FilterIncludeAll);
    eprintln!("\nThe issued contract data:");
    eprintln!("{}", contract.name());

    for FungibleAllocation  { seal, state, witness, .. } in allocations {
        let witness = witness.as_ref().map(XWitnessId::to_string).unwrap_or("~".to_owned());
        eprintln!("amount={state}, owner={seal}, witness={witness}");
    }
    eprintln!("totalSupply={}", contract.total_issued_supply());
}
