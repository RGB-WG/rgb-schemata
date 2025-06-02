use std::fs;

use amplify::confinement::SmallBlob;
use amplify::hex::FromHex;
use amplify::{Bytes, Wrapper};
use bp::Txid;
use rgbstd::containers::{ConsignmentExt, FileContent, Kit};
use rgbstd::contract::{DataAllocation, FilterIncludeAll, IssuerWrapper};
use rgbstd::invoice::Precision;
use rgbstd::persistence::Stock;
use rgbstd::stl::{
    AssetSpec, Attachment, ContractTerms, EmbeddedMedia, MediaType, RicardianContract, TokenData,
};
use rgbstd::{Allocation, ChainNet, GenesisSeal, TokenIndex};
use schemata::dumb::NoResolver;
use schemata::UniqueDigitalAsset;
use sha2::{Digest, Sha256};

fn main() {
    let beneficiary_txid =
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = GenesisSeal::new_random(beneficiary_txid, 1);

    let spec = AssetSpec::new("TEST", "Test uda", Precision::Indivisible);

    let file_bytes = fs::read("README.md").unwrap();
    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    let file_hash = hasher.finalize();
    let terms = ContractTerms {
        text: RicardianContract::default(),
        media: Some(Attachment {
            ty: MediaType::with("text/*"),
            digest: Bytes::from_byte_array(file_hash),
        }),
    };

    let index = TokenIndex::from_inner(2);
    let preview = EmbeddedMedia {
        ty: MediaType::with("image/*"),
        data: SmallBlob::try_from_iter(vec![0, 0]).expect("invalid data"),
    };
    let token_data = TokenData {
        index,
        preview: Some(preview),
        ..Default::default()
    };

    let allocation = Allocation::with(index, 1);

    let mut stock = Stock::in_memory();
    let kit = Kit::load_file("schemata/UniqueDigitalAsset.rgb")
        .unwrap()
        .validate()
        .unwrap();
    stock.import_kit(kit).expect("invalid issuer kit");

    let contract = stock
        .contract_builder(
            "ssi:anonymous",
            UniqueDigitalAsset::schema().schema_id(),
            ChainNet::BitcoinTestnet4,
        )
        .unwrap()
        .add_global_state("spec", spec)
        .expect("invalid spec")
        .add_global_state("terms", terms)
        .expect("invalid contract terms")
        .add_global_state("tokens", token_data)
        .expect("invalid token data")
        .add_data("assetOwner", beneficiary, allocation)
        .expect("invalid asset blob")
        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract
        .save_file("test/uda-example.rgb")
        .expect("unable to save contract");
    contract
        .save_armored("test/uda-example.rgba")
        .expect("unable to save armored contract");

    stock.import_contract(contract, NoResolver).unwrap();

    // Reading contract state from the stock:
    let contract = stock
        .contract_wrapper::<UniqueDigitalAsset>(contract_id)
        .unwrap();
    let allocations = contract.allocations(&FilterIncludeAll);
    eprintln!("\nThe issued contract:");
    eprintln!("{}", serde_json::to_string(&contract.spec()).unwrap());

    for DataAllocation {
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
        eprintln!("state={state}, owner={seal}, witness={witness}");
    }
}
