/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use std::borrow::{Borrow, BorrowMut};
use std::convert::{TryFrom, TryInto};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{Base64VecU8, ValidAccountId};
use near_sdk::{env, ext_contract,near_bindgen, AccountId,Balance,Gas, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, log};
use near_sdk::env::{log, sha256, state_read};
// use near_sdk::PromiseOrValue::Promise;
use near_sdk::serde_json::{json, json_internal_vec};

near_sdk::setup_alloc!();

#[ext_contract(ext_pool)]
pub trait DeployPool {
    fn new_pool(&mut self, poolname:AccountId, owner_id:AccountId,roomsize :U128) -> PromiseOrValue<AccountId>;
}


const NO_DEPOSIT: Balance = 0;
const BASE_GAS: Gas = 5_000_000_000_000;
const PROMISE_CALL: Gas = 5_000_000_000_000;
const GAS_FOR_NFT_ON_APPROVE: Gas = BASE_GAS + PROMISE_CALL;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    tokenIds : LazyOption<String>,
    ContractGlobal : LazyOption<AccountId>
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    TokenIds,
    ContractOwner,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).


    #[init]
    pub fn new(owner_id: ValidAccountId,name :String,symbol:String,base_uri:String) -> Self {
        // assert!(!env::state_exists(), "Already initialized");
        let metadata:NFTContractMetadata=NFTContractMetadata{
            spec: NFT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
            base_uri: Some(base_uri),
            reference:None,
            reference_hash:None
        };
        let initcounter : String = "0".to_string();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id.clone(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            tokenIds: LazyOption::new(StorageKey::TokenIds, Some(&initcounter)),
            ContractGlobal:LazyOption::new(StorageKey::ContractOwner, Some(&owner_id.into()))
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: ValidAccountId,
        ipfs_hash: String
    ) -> Token {
        let globalowner= self.ContractGlobal.get().unwrap();
        if !(globalowner.to_string()==env::predecessor_account_id()){
            let token = self.tokens.borrow();
            let alltokenmap = token.tokens_per_owner.as_ref();
            let some= alltokenmap.unwrap();
            let tokenids= some.get(&env::predecessor_account_id()).unwrap();
            if tokenids.len()<1{
                assert!(false);
            }
        }

        let media_hash= env::sha256(ipfs_hash.clone().as_bytes());
        let latest_counter:String = self.tokenIds.get().unwrap();
        let int_counter : i32 = latest_counter.parse().unwrap();

        let owner_metadata= TokenMetadata{
            title:Some("ownernft".to_string()),
            description:None,
            copies:Some(1),
            issued_at:None,
            expires_at:None,
            starts_at:Some(env::block_timestamp().to_string()),
            updated_at:Some(env::block_timestamp().to_string()),
            extra:None,
            reference:None,
            reference_hash:None,
            media: Some(ipfs_hash.clone()),
            media_hash: Some(Base64VecU8::from(media_hash.clone()))
        };
        for i in 1..3 {
            let token_metadata = TokenMetadata {
                title: Some("inviteNft".to_string()),
                description: None,
                copies: Some(1),
                issued_at: None,
                expires_at: None,
                starts_at: Some(env::block_timestamp().to_string()),
                updated_at: Some(env::block_timestamp().to_string()),
                extra: None,
                reference: None,
                reference_hash: None,
                media: Some(ipfs_hash.clone()),
                media_hash: Some(Base64VecU8::from(media_hash.clone()))

            };

            env::log("token minted".to_string().as_bytes());
           let _= self.tokens.mint((int_counter+1+i).to_string(), receiver_id.clone(), Some(token_metadata));
        }

        env::log("token minted".to_string().as_bytes());
        self.tokenIds.replace(&(int_counter+3).to_string());
        self.tokens.mint((int_counter+1).to_string(), receiver_id, Some(owner_metadata))

    }

    pub fn create_room(&mut self, pool_id : AccountId, roomsize : U128)->PromiseOrValue<String>{
        // assert_eq!(self.tokens.owner_id, env::predecessor_account_id());
        let prepaid_gas = env::prepaid_gas();
        let account_id = env::predecessor_account_id();
        let mut poolname:Vec<&str> = account_id.split(".").collect();
        let counter=self.tokenIds.get().unwrap().to_owned();
        let mut finalname = poolname[0].to_string();
        finalname.push_str("123");
        ext_pool::new_pool(finalname.to_string(),env::predecessor_account_id(),roomsize, &pool_id,NO_DEPOSIT, prepaid_gas - GAS_FOR_NFT_ON_APPROVE).into()
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

// #[cfg(all(test, not(target_arch = "wasm32")))]
// mod tests {
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::testing_env;
//
//     use super::*;
//
//     const MINT_STORAGE_COST: u128 = 5870000000000000000000;
//
//     fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .predecessor_account_id(predecessor_account_id);
//         builder
//     }
//
//     fn sample_token_metadata() -> TokenMetadata {
//         TokenMetadata {
//             title: Some("Olympus Mons".into()),
//             description: Some("The tallest mountain in the charted solar system".into()),
//             media: None,
//             media_hash: None,
//             copies: Some(1u64),
//             issued_at: None,
//             expires_at: None,
//             starts_at: None,
//             updated_at: None,
//             extra: None,
//             reference: None,
//             reference_hash: None,
//         }
//     }
//
//     #[test]
//     fn test_new() {
//         let mut context = get_context(accounts(1));
//         testing_env!(context.build());
//         let baseURI= "somename".to_string();
//         let contract = Contract::new_default_meta(accounts(1).into(),baseURI);
//         testing_env!(context.is_view(true).build());
//         assert_eq!(contract.nft_token("1".to_string()), None);
//     }
//
//     #[near_bindgen]
//     #[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
//     pub struct SomeOtherContract {
//         tokens: NonFungibleToken,
//         metadata: LazyOption<NFTContractMetadata>,
//     }
//
//     #[test]
//     #[should_panic(expected = "The contract is not initialized")]
//     fn test_default() {
//         let context = get_context(accounts(1));
//         testing_env!(context.build());
//         let _contract = Contract::default();
//     }
//
//     #[test]
//     fn test_mint() {
//         let mut context = get_context(accounts(0));
//         testing_env!(context.build());
//         let baseuri:String="somename".to_string();
//         let mut contract = Contract::new_default_meta(accounts(0).into(),baseuri);
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(MINT_STORAGE_COST)
//             .predecessor_account_id(accounts(0))
//             .build());
//
//         let token_id = "0".to_string();
//         let ipfsHash:String="mtwirsqawjuoloq2gvtyug2tc3jbf5htm2zeo4rsknfiv3fdp46a".to_string();
//         let token = contract.nft_mint(token_id.clone(), accounts(0), ipfsHash);
//         assert_eq!(token.token_id, token_id);
//         assert_eq!(token.owner_id, accounts(0).to_string());
//         assert_eq!(token.metadata.unwrap(), sample_token_metadata());
//         assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
//     }
//
//     #[test]
//     fn test_transfer() {
//         let mut context = get_context(accounts(0));
//         testing_env!(context.build());
//         let mut contract = Contract::new_default_meta(accounts(0).into());
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(MINT_STORAGE_COST)
//             .predecessor_account_id(accounts(0))
//             .build());
//         let token_id = "0".to_string();
//         contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(1)
//             .predecessor_account_id(accounts(0))
//             .build());
//         contract.nft_transfer(accounts(1), token_id.clone(), None, None);
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .account_balance(env::account_balance())
//             .is_view(true)
//             .attached_deposit(0)
//             .build());
//         if let Some(token) = contract.nft_token(token_id.clone()) {
//             assert_eq!(token.token_id, token_id);
//             assert_eq!(token.owner_id, accounts(1).to_string());
//             assert_eq!(token.metadata.unwrap(), sample_token_metadata());
//             assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
//         } else {
//             panic!("token not correctly created, or not found by nft_token");
//         }
//     }
//
//     #[test]
//     fn test_approve() {
//         let mut context = get_context(accounts(0));
//         testing_env!(context.build());
//         let mut contract = Contract::new_default_meta(accounts(0).into());
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(MINT_STORAGE_COST)
//             .predecessor_account_id(accounts(0))
//             .build());
//         let token_id = "0".to_string();
//         let hash= "somename".to_string();
//         contract.nft_mint(token_id.clone(), accounts(0), hash);
//
//         // alice approves bob
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(150000000000000000000)
//             .predecessor_account_id(accounts(0))
//             .build());
//         contract.nft_approve(token_id.clone(), accounts(1), None);
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .account_balance(env::account_balance())
//             .is_view(true)
//             .attached_deposit(0)
//             .build());
//         assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
//     }
//
//     #[test]
//     fn test_revoke() {
//         let mut context = get_context(accounts(0));
//         testing_env!(context.build());
//         let mut contract = Contract::new_default_meta(accounts(0).into());
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(MINT_STORAGE_COST)
//             .predecessor_account_id(accounts(0))
//             .build());
//         let token_id = "0".to_string();
//         contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
//
//         // alice approves bob
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(150000000000000000000)
//             .predecessor_account_id(accounts(0))
//             .build());
//         contract.nft_approve(token_id.clone(), accounts(1), None);
//
//         // alice revokes bob
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(1)
//             .predecessor_account_id(accounts(0))
//             .build());
//         contract.nft_revoke(token_id.clone(), accounts(1));
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .account_balance(env::account_balance())
//             .is_view(true)
//             .attached_deposit(0)
//             .build());
//         assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
//     }
//
//     #[test]
//     fn test_revoke_all() {
//         let mut context = get_context(accounts(0));
//         testing_env!(context.build());
//         let mut contract = Contract::new_default_meta(accounts(0).into());
//
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(MINT_STORAGE_COST)
//             .predecessor_account_id(accounts(0))
//             .build());
//         let token_id = "0".to_string();
//         contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
//
//         // alice approves bob
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(150000000000000000000)
//             .predecessor_account_id(accounts(0))
//             .build());
//         contract.nft_approve(token_id.clone(), accounts(1), None);
//
//         // alice revokes bob
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(1)
//             .predecessor_account_id(accounts(0))
//             .build());
//         contract.nft_revoke_all(token_id.clone());
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .account_balance(env::account_balance())
//             .is_view(true)
//             .attached_deposit(0)
//             .build());
//         assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
//     }
// }
