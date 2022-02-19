#./build.sh
#
#### resetting everything
#near delete nftcontract.somenewname.testnet somenewname.testnet
#near delete nftpoolcontract.somenewname.testnet somenewname.testnet
###
#### deplow section
#near state somenewname.testnet
#near create_account nftcontract.somenewname.testnet --masterAccount somenewname.testnet --initialBalance 15
#near create_account nftpoolcontract.somenewname.testnet --masterAccount somenewname.testnet --initialBalance 15
####
#near deploy --accountId nftcontract.somenewname.testnet --wasmFile ./res/non_fungible_token.wasm --initFunction new --initArgs '{"owner_id": "somenewname.testnet", "name": "somename" , "symbol" : "someothername" ,"base_uri": "somenewname"}'
#near deploy --accountId nftpoolcontract.somenewname.testnet --wasmFile ./res/nft_pool.wasm --initFunction new --initArgs '{"subowner" : "somenewname.testnet"}'
####
###### for calling
####
#near call nftcontract.somenewname.testnet nft_mint '{"receiver_id":"somenewname.testnet","ipfs_hash":"somenewshitlikethis"}' --accountId somenewname.testnet --depositYocto 7400000000000000000000
#near call nftcontract.somenewname.testnet create_room '{"pool_id":"nftpoolcontract.somenewname.testnet","roomsize":"200000000"}' --accountId somenewname.testnet
##near call somenewname.testnet nft_transfer '{"receiver_id":"somename.testnet","token_id":"somenamelikethis"}' --accountId somenewname.testnet --depositYocto 1

near call somenewname123.nftpoolcontract.somenewname.testnet ft_balance_of '{"account_id":"somenewname.testnet"}' --accountId somenewname.testnet
#'somenewname20.nftpoolcontract.somenewname.testnet'



