use amplify::hex::FromHex;
use bp::Txid;
use rgbstd::containers::{ConsignmentExt, FileContent, Kit};
use rgbstd::contract::{FilterIncludeAll, FungibleAllocation, IssuerWrapper};
use rgbstd::invoice::Precision;
use rgbstd::persistence::Stock;
use rgbstd::stl::{ContractTerms, Name, RicardianContract};
use rgbstd::{Amount, ChainNet, GenesisSeal};
use schemata::dumb::NoResolver;
use schemata::CollectibleFungibleAsset;

fn main() {
    let beneficiary_txid =
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = GenesisSeal::new_random(beneficiary_txid, 1);

    let name = Name::from("Test asset");

    let precision = Precision::CentiMicro;

    let terms = ContractTerms {
        text: RicardianContract::default(),
        media: None,
    };

    let issued_supply = Amount::from(100000u64);

    let mut stock = Stock::in_memory();
    let kit = Kit::load_file("schemata/CollectibleFungibleAsset.rgb")
        .unwrap()
        .validate()
        .unwrap();
    stock.import_kit(kit).expect("invalid issuer kit");

    let contract = stock
        .contract_builder(
            "ssi:anonymous",
            CollectibleFungibleAsset::schema().schema_id(),
            ChainNet::BitcoinTestnet4,
        )
        .unwrap()
        .add_global_state("name", name)
        .expect("invalid name")
        .add_global_state("precision", precision)
        .expect("invalid precision")
        .add_global_state("terms", terms)
        .expect("invalid contract terms")
        .add_global_state("issuedSupply", issued_supply)
        .expect("invalid issued supply")
        .add_fungible_state("assetOwner", beneficiary, 100000u64)
        .expect("invalid fungible state")
        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract
        .save_file("test/cfa-example.rgb")
        .expect("unable to save contract");
    contract
        .save_armored("test/cfa-example.rgba")
        .expect("unable to save armored contract");

    stock.import_contract(contract, NoResolver).unwrap();

    // Reading contract state from the stock:
    let contract = stock
        .contract_wrapper::<CollectibleFungibleAsset>(contract_id)
        .unwrap();
    let allocations = contract.allocations(&FilterIncludeAll);
    eprintln!("\nThe issued contract:");
    eprintln!("{}", contract.name());

    for FungibleAllocation {
        seal,
        state,
        witness,
        ..
    } in allocations
    {
        let witness = witness
            .as_ref()
            .map(Txid::to_string)
            .unwrap_or("~".to_owned());
        eprintln!("amount={}, owner={seal}, witness={witness}", state.value());
    }
    eprintln!("totalSupply={}", contract.total_issued_supply().value());
}
