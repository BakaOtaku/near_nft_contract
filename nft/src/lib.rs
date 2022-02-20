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
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{Base64VecU8, ValidAccountId};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, log, PromiseResult};
use near_sdk::env::{log, promise_result, sha256, state_read};
// use near_sdk::PromiseOrValue::Promise;
// use near_sdk::PromiseOrValue::Promise;
use near_sdk::serde_json::{json, json_internal_vec};

near_sdk::setup_alloc!();

#[ext_contract(ext_pool)]
pub trait DeployPool {
    fn new_pool(&mut self, poolname:AccountId, owner_id:AccountId,roomsize :U128) -> PromiseOrValue<AccountId>;
}

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_balance_of(&mut self, account_id: AccountId) -> U128;
    fn nft_internal_transfer(&mut self, invitee: AccountId, amount : U128);
}
//
// #[ext_contract(ext_self)]
// pub trait MyContract {
//     fn my_callback(& mut self,reciever_id:ValidAccountId,ipfs_hash:String) -> Token;
// }
#[ext_contract(ext_self)]
pub trait MyContract {
    fn nft_mint_callback(&mut self,reciever_id :ValidAccountId,ipfs_hash: String) -> Token;
    fn invite_other_callback(&mut self, caller : AccountId,invitee : ValidAccountId) -> PromiseOrValue<String>;
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
    ContractGlobal : LazyOption<AccountId>,
    OwnerNftStore : LookupMap<AccountId,String>,
    InviteNftCounts : LookupMap<AccountId,u128>
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
    OwnerNft,
    InviteCount
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new(owner_id: ValidAccountId, name: String, symbol: String, base_uri: String) -> Self {
        // assert!(!env::state_exists(), "Already initialized");
        let metadata: NFTContractMetadata = NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
            base_uri: Some(base_uri),
            reference: None,
            reference_hash: None
        };
        let initcounter: String = "0".to_string();
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
            ContractGlobal: LazyOption::new(StorageKey::ContractOwner, Some(&owner_id.into())),
            OwnerNftStore: LookupMap::new(StorageKey::OwnerNft),
            InviteNftCounts : LookupMap::new( StorageKey::InviteCount)
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
        ipfs_hash: String
    ) -> Promise {
        let reciever_id: String = env::predecessor_account_id();
        let validAccountID = ValidAccountId::try_from(reciever_id.clone()).unwrap();
        let somename = validAccountID.to_string();
        log!(somename);
        ext_ft::ft_balance_of(
            reciever_id.clone().into(),
            &"nfterc20contract.somenewname.testnet", // contract account id
            0, // yocto NEAR to attach
            5_000_000_000_000 // gas to attach
        ).then(ext_self::nft_mint_callback(
            validAccountID.into(),
            ipfs_hash,
            &env::current_account_id(), // this contract's account id
            7620000000000000000000, // yocto NEAR to attach to the callback
            9_000_000_000_000 // gas to attach to the callback
        ))
    }

    #[payable]
    pub fn create_pool(&mut self, pool_id: AccountId, roomsize: U128) -> PromiseOrValue<String> {
        // assert_eq!(self.tokens.owner_id, env::predecessor_account_id());
        // self.tokens.internal_transfer()
        // // log!("{}",env::attached_deposit().to_string());
        // let prepaid_gas = env::prepaid_gas();
        let account_id = env::predecessor_account_id();
        let tokenid = self.OwnerNftStore.get(&account_id.clone()).unwrap_or_else(|| "".to_string());
        log!(tokenid);
        if tokenid == "" {
            assert!(false, "token id not found")
        }
        log!("token id for owner found");
        let validCurrentId = ValidAccountId::try_from(env::current_account_id()).unwrap();
        // self.tokens.nft_approve(tokenid.clone(), validCurrentId.clone(), Some("".to_string()));
        log!("before nft transfer");
        // self.tokens.nft_transfer(validCurrentId, tokenid.clone(), None, None);
        self.tokens.internal_transfer(&env::predecessor_account_id(), &env::current_account_id(), &tokenid.clone(), None, None);
        log!("after nft transfer");
        //
        let mut poolname: Vec<&str> = account_id.split(".").collect();
        let counter = self.tokenIds.get().unwrap().to_owned();
        let mut finalname = poolname[0].to_string();
        finalname.push_str("102202");
        log!("creating pool");

        ext_pool::new_pool(finalname.to_string(), env::predecessor_account_id(), roomsize, &pool_id, NO_DEPOSIT, env::prepaid_gas() / 2).into()
    }

    #[payable]
    pub fn nft_mint_callback(&mut self, reciever_id: ValidAccountId, ipfs_hash: String) -> Token {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        log!(ipfs_hash);
        let stuff = reciever_id.to_string();
        log!(stuff);
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => unreachable!(),
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                log!("this is balance very much");
                if (balance.0 < 0) {
                    assert!(false, "balance less then required")
                }
                // } else {
                //     "Hmmmm".to_string()
                // }
            },
        }
        log!("{}",ipfs_hash.clone());

        let media_hash = env::sha256(ipfs_hash.clone().as_bytes());
        let latest_counter: String = self.tokenIds.get().unwrap();
        let int_counter: i32 = latest_counter.parse().unwrap();
        // log!(int_counter.to_string());
        let owner_metadata = TokenMetadata {
            title: Some("ownerNft".to_string()),
            description: Some(format!("owner nft for {}", reciever_id.to_string())),
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
        // for i in 1..3 {
        //     let token_metadata = TokenMetadata {
        //         title: Some("inviteNft".to_string()),
        //         description: None,
        //         copies: Some(1),
        //         issued_at: None,
        //         expires_at: None,
        //         starts_at: Some(env::block_timestamp().to_string()),
        //         updated_at: Some(env::block_timestamp().to_string()),
        //         extra: None,
        //         reference: None,
        //         reference_hash: None,
        //         media: Some(ipfs_hash.clone()),
        //         media_hash: Some(Base64VecU8::from(media_hash.clone()))
        //
        //     };
        //
        //     env::log("token minted".to_string().as_bytes());
        //     let _= self.tokens.mint((int_cosunter+1+i).to_string(), reciever_id.clone(), Some(token_metadata));
        // }
        let inviteecount : u128 = 2;
        self.InviteNftCounts.insert(&reciever_id.clone().into(), &inviteecount);
        env::log("ownwer token minted".to_string().as_bytes());
        self.tokenIds.replace(&(int_counter + 1).to_string());
        self.OwnerNftStore.insert(&reciever_id.clone().into(), &(int_counter + 1).to_string());
        self.tokens.mint((int_counter + 1).to_string(), reciever_id.clone(), Some(owner_metadata))
    }

    #[payable]
    pub fn invite_other( &mut self , invitee : ValidAccountId)->Promise{
        let inviteeleft=self.InviteNftCounts.get(&env::predecessor_account_id()).unwrap_or_else(||10);
        let caller = env::predecessor_account_id();
        if inviteeleft==10{
            assert!(false,"no invitee nft are left")
        }
        log!("invite init started");
        log!("amount transfer init");
        let amounttransfer = U128::try_from(0).unwrap();

        let first = Promise::new("nfterc20contract.somenewname.testnet".to_string()).function_call(
            b"nft_internal_transfer".to_vec(),
            json!({"invitee":invitee.to_string(),"amount":U128::from(1)}).to_string().into_bytes(),
            0,
            5_000_000_000_000
        );

        // let first=ext_ft::nft_internal_transfer(invitee.clone().into(),amounttransfer , &"nfterc20contract.somenewname.testnet", 0, 9_000_000_000_000);
        log!("after transfer");
        let second= ext_self::invite_other_callback(
            caller,
            invitee,
            &env::current_account_id(), // contract account id
            7630000000000000000000, // yocto NEAR to attach
            9_000_000_000_000 // gas to attach
            );

        first.then(second)
    }

    #[payable]
    pub fn invite_other_callback(&mut self, caller : AccountId,invitee : ValidAccountId) ->PromiseOrValue<String>{
        log!("in invite nft callback");
        let inviteeleft=self.InviteNftCounts.get(&caller.clone()).unwrap_or_else(||10);
        if inviteeleft==10{
            assert!(false,"no invitee nft are left")
        }

        self.InviteNftCounts.insert(&caller,&(inviteeleft-1));

        let latest_counter: String = self.tokenIds.get().unwrap();
        let int_counter: i32 = latest_counter.parse().unwrap();

        log!("latest counter achieved");
        let token_metadata = TokenMetadata {
            title: Some("inviteNft".to_string()),
            description: Some(format!("invited by {}",env::predecessor_account_id())),
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: Some(env::block_timestamp().to_string()),
            updated_at: Some(env::block_timestamp().to_string()),
            extra: None,
            reference: None,
            reference_hash: None,
            media: None,
            media_hash: None

        };




        log!("invitee nft minted1");
        self.tokenIds.replace(&(int_counter + 1).to_string());
        log!("invitee nft minted2");
        self.tokens.mint((int_counter+1).to_string(), invitee.clone(), Some(token_metadata));
        PromiseOrValue::Value((int_counter+1).to_string())
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

