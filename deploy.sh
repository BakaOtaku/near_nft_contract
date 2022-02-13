## for deploying the smart contract
#near deploy --accountId somenewname.testnet --wasmFile ./res/non_fungible_token.wasm --initFunction new_default_meta --initArgs '{"owner_id": "somenewname.testnet", "base_uri": "myteammateisanasshole"}'

## for calling functions
#near call somenewname.testnet nft_mint '{"token_id":"somenamelikethis","receiver_id":"somenewname.testnet","ipfs_hash":"somenewshitlikethis"}' --accountId somenewname.testnet --depositYocto 7400000000000000000000
near call somenewname.testnet nft_transfer '{"receiver_id":"somename.testnet","token_id":"somenamelikethis"}' --accountId somenewname.testnet --depositYocto 1



