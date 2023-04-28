import {
    storeCode, instantiateContract, executeContract, queryStakingDelegations,
    queryWasmContract, queryAddressBalance, queryStaking, queryStakingParameters, sendCoin
} from "./common";
import {DirectSecp256k1HdWallet} from '@cosmjs/proto-signing';
import {parseCoins, coins, coin} from "@cosmjs/stargate";

require("dotenv").config();

async function main(): Promise<void> {

    console.log(`--- deploy enter ---`)

    const LCD_ENDPOINT = process.env.LCD_ENDPOINT;
    const RPC_ENDPOINT = process.env.RPC_ENDPOINT;
    const mnemonic = process.env.MNEMONIC;
    const mnemonic2 = process.env.MNEMONIC2;
    const validator = process.env.validator;
    const stable_coin_denom = process.env.stable_coin_denom;

    if (!LCD_ENDPOINT || !RPC_ENDPOINT || !mnemonic || !mnemonic2 || !validator || !stable_coin_denom) {
        console.log(`--- deploy error, missing some attributes ---`)
        process.exit(0);
        return;
    }

    const prefix = process.env.PREFIX ?? "sei";
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {prefix});
    const [account] = await wallet.getAccounts();
    const wallet2 = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic2, {prefix});
    const [account2] = await wallet2.getAccounts();

    console.log(`address1: `, account.address)
    await queryAddressBalance(LCD_ENDPOINT, account.address, "usei");
    await queryAddressBalance(LCD_ENDPOINT, account.address, stable_coin_denom);
    console.log()
    console.log(`address2: `, account2.address)
    await queryAddressBalance(LCD_ENDPOINT, account2.address, "usei");
    await queryAddressBalance(LCD_ENDPOINT, account2.address, stable_coin_denom);

    let hubCodeId = 253;
    let rewardCodeId = 254;
    let bSeiTokenCodeId = 255;
    let rewardsDispatcherCodeId = 256;
    let validatorsRegistryCodeId = 257;
    let stSeiTokenCodeId = 258;

    let hubContractAddress = process.env.hubContractAddress;
    let rewardAddress = process.env.rewardAddress;
    let bSeiTokenAddress = process.env.bSeiTokenAddress;
    let rewardsDispatcherAddress = process.env.rewardsDispatcherAddress;
    let validatorsRegistryAddress = process.env.validatorsRegistryAddress;
    let stSeiTokenAddress = process.env.stSeiTokenAddress;

    let deployAddressFlag: boolean = false;
    if (!hubContractAddress) {
        deployAddressFlag = true;
        hubCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_hub.wasm");
        hubContractAddress = await instantiateContract(RPC_ENDPOINT, wallet, hubCodeId,
            {
                epoch_period: 30,
                er_threshold: "1.0",
                peg_recovery_fee: "0",
                reward_denom: stable_coin_denom,
                unbonding_period: 120,
                underlying_coin_denom: "usei",
                validator: validator   //local node validator address
            }, parseCoins(""), "lido sei hub")
    }

    if (!rewardAddress) {
        deployAddressFlag = true;
        rewardCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_reward.wasm");
        rewardAddress = await instantiateContract(RPC_ENDPOINT, wallet, rewardCodeId,
            {
                owner: account.address,
                hub_contract: hubContractAddress,
                reward_denom: stable_coin_denom,
            }, parseCoins(""), "sei reward")
    }

    if (!bSeiTokenAddress) {
        deployAddressFlag = true;
        bSeiTokenCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_token_bsei.wasm");
        bSeiTokenAddress = await instantiateContract(RPC_ENDPOINT, wallet, bSeiTokenCodeId,
            {
                decimals: 6, hub_contract: hubContractAddress, initial_balances: [],
                name: "bsei", symbol: "BSEI",
                mint: {minter: hubContractAddress, cap: null}
            }, parseCoins(""), "bond sei")
    }
    if (!rewardsDispatcherAddress) {
        deployAddressFlag = true;
        rewardsDispatcherCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_rewards_dispatcher.wasm");
        rewardsDispatcherAddress = await instantiateContract(RPC_ENDPOINT, wallet, rewardsDispatcherCodeId,
            {
                lido_fee_address: account.address,
                lido_fee_rate: "0.05",
                hub_contract: hubContractAddress,
                bsei_reward_contract: rewardAddress,
                stsei_reward_denom: "usei",
                bsei_reward_denom: stable_coin_denom,
            }, parseCoins(""), "reward dispatcher")
    }

    if (!validatorsRegistryAddress) {
        deployAddressFlag = true;
        validatorsRegistryCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_validators_registry.wasm")
        validatorsRegistryAddress = await instantiateContract(RPC_ENDPOINT, wallet, validatorsRegistryCodeId,
            {
                hub_contract: hubContractAddress,
                registry: [{
                    active: true,
                    address: validator,    //env 192.168.2.66 validator
                    total_delegated: "0"
                }]
            }, parseCoins(""), "validator registery")
    }

    if (!stSeiTokenAddress) {
        deployAddressFlag = true;
        stSeiTokenCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_token_stsei.wasm");
        stSeiTokenAddress = await instantiateContract(RPC_ENDPOINT, wallet, stSeiTokenCodeId,
            {
                decimals: 6,
                hub_contract: hubContractAddress,
                initial_balances: [],
                name: "stsei", symbol: "STSEI",
                mint: {minter: hubContractAddress, cap: null}
            }, parseCoins(""), "staking sei")
    }

    // console.log()

    // console.log(`HUB_CONTRACT = ${hubContractAddress}`)
    // console.log(`REWARD_CONTRACT = ${rewardAddress}`)
    // console.log(`BSEI_TOKEN_CONTRACT = ${bSeiTokenAddress}`)
    // console.log(`REWARDS_DISPATCHER_CONTRACT = ${rewardsDispatcherAddress}`)
    // console.log(`VALIDATORS_REGISTRY_CONTRACT = ${validatorsRegistryAddress}`)
    // console.log(`STSEI_TOKEN_CONTRACT = ${stSeiTokenAddress}`)
    //
    // let addressObj = {
    //     "hubContractAddress": hubContractAddress,
    //     "bSeiRewardAddress": rewardAddress,
    //     "rewardsDispatcherAddress": rewardsDispatcherAddress,
    //     "validatorsRegistryAddress": validatorsRegistryAddress,
    //     "bSeiTokenAddress": bSeiTokenAddress,
    //     "stSeiTokenAddress": stSeiTokenAddress,
    // }
    // console.log(JSON.stringify(addressObj))
    console.log()
    console.log(`hubContractAddress: "${hubContractAddress}",`)
    console.log(`rewardAddress: "${rewardAddress}",`)
    console.log(`rewardsDispatcherAddress: "${rewardsDispatcherAddress}",`)
    console.log(`validatorsRegistryAddress: "${validatorsRegistryAddress}",`)
    console.log(`bSeiTokenAddress: "${bSeiTokenAddress}",`)
    console.log(`stSeiTokenAddress: "${stSeiTokenAddress}",`)
    console.log()

    // //////////////////////////////////////configure contracts///////////////////////////////////////////
    // //////////////////////////////////////must behind deploy moneymarket contract///////////////////////

    if (deployAddressFlag) {
        console.log("Updating hub's config...")
        await executeContract(RPC_ENDPOINT, wallet, hubContractAddress, {
            update_config: {
                bsei_token_contract: bSeiTokenAddress,
                stsei_token_contract: stSeiTokenAddress,
                rewards_dispatcher_contract: rewardsDispatcherAddress,
                validators_registry_contract: validatorsRegistryAddress,
                rewards_contract: rewardAddress,
            }
        }, "", parseCoins(""))
        console.log("Updating hub's config end")
    }

    //======================deployed contracts，change creater to update_global_index=======================================//
    //change creater，
    // await executeContract(RPC_ENDPOINT, wallet, hubAddress,
    // {
    //   update_config: {
    //     owner : "sei1xm3mccak0yjfts96jszdldxka6xkw00ywv6au0"
    //   }
    // }, "", parseCoins("") )
    // console.log("transfer owener ok.")


    // //just a few simple tests to make sure the contracts are not failing
    // //for more accurate tests we must use integration-tests repo
    await executeContract(RPC_ENDPOINT, wallet, hubContractAddress, {bond_for_st_sei: {}}, "", parseCoins("1000000usei"))
    console.log("test hubaddress bond for stSei ok...")

    await executeContract(RPC_ENDPOINT, wallet, hubContractAddress, {bond_for_st_sei: {}}, "", parseCoins("1000000usei"))
    console.log("test hubaddress bond for stSei ok...")

    await executeContract(RPC_ENDPOINT, wallet, hubContractAddress, {bond: {}}, "", parseCoins("10000000usei"))
    console.log("test hubaddress bond for bSei ok...")

    console.log("query bond sei balance:")
    await queryWasmContract(RPC_ENDPOINT, wallet, bSeiTokenAddress, {balance: {address: account.address}})


    await executeContract(RPC_ENDPOINT, wallet, bSeiTokenAddress, {
        send: {
            contract: hubContractAddress, amount: "500000",
            msg: Buffer.from(JSON.stringify({"unbond": {}})).toString('base64')
        }
    }, "", parseCoins(""))
    console.log("send bseiTokenAddress unbond for bSei ok...")

    console.log("query hub contract balance:")
    await queryAddressBalance(LCD_ENDPOINT, hubContractAddress, stable_coin_denom);

    console.log("query withdraw able unbonded:")
    await queryWasmContract(RPC_ENDPOINT, wallet, hubContractAddress, {withdrawable_unbonded: {address: account.address}})

    console.log("query staking pool:")
    await queryStaking(LCD_ENDPOINT);


    console.log("query current batch:")
    await queryWasmContract(RPC_ENDPOINT, wallet, hubContractAddress, {current_batch: {}})


    console.log("query unbond request:")
    await queryWasmContract(RPC_ENDPOINT, wallet, hubContractAddress, {unbond_requests: {address: account.address}})


    console.log("query staking parameter:")
    await queryStakingParameters(LCD_ENDPOINT);

    // console.log("query delegations list:")
    // await queryStakingDelegations(LCD_ENDPOINT, account.address, "seivaloper1wukzl3ppckudkkny744k4kmrw0p0l6h98sm43s");

    console.log("withdraw able unbonded:")
    let withdrawRet = await executeContract(RPC_ENDPOINT, wallet, hubContractAddress, {withdraw_unbonded: {}}, "", parseCoins(""))
    console.log("withdraw able unbonded ok")

    console.log("query hub config:")
    await queryWasmContract(RPC_ENDPOINT, wallet, hubContractAddress, {config: {}})

    await queryAddressBalance(LCD_ENDPOINT, account.address, "")

    await executeContract(RPC_ENDPOINT, wallet, hubContractAddress, {update_global_index: {}}, "",
        coins(100000000, stable_coin_denom))
    console.log("update_reward_basset ok")

    console.log("query accured reward:")
    await queryWasmContract(RPC_ENDPOINT, wallet, rewardAddress,
        {
            accrued_rewards:
                {
                    address: account.address
                }
        })

    console.log("query rewards balance:")
    await queryAddressBalance(LCD_ENDPOINT, rewardAddress, stable_coin_denom)

    // console.log("query test2 balance:")
    // await queryAddressBalance(LCD_ENDPOINT, account2.address, "")

    console.log("claim reward:")
    let claimRewardRet = await executeContract(RPC_ENDPOINT, wallet, rewardAddress,
        {
            claim_rewards: {
                recipient: account.address,
            }
        }, "", parseCoins(""))
    console.log("claim reward ok!")

    console.log(`query ${account.address}`)
    await queryAddressBalance(LCD_ENDPOINT, account.address, "")

    console.log("query address test2 balance after claim reward:")
    await queryAddressBalance(LCD_ENDPOINT, account2.address, "")


    // let recAddress = "sei1tqm527sqmuw2tmmnlydge024ufwnvlv9e7draq";
    // // 2.2 send stable coin to test2 address
    // let sendCoinRet = await sendCoin(RPC_ENDPOINT, wallet, recAddress, "", coin(10000000, stable_coin_denom))
    // console.log(`send stable token to ${recAddress} succeed`);
    // console.log(`query ${recAddress} usdc balance:`);
    // let queryUsdcRet = await queryAddressBalance(LCD_ENDPOINT, recAddress, "")


    //***Test sending denom usdt token and sei coin**/
    // let receipientAddress = "sei1mlwyp04y5g95klqzq92tun0xsz7t5sef4h88a3";
    // // console.log("send denom usdt token to address:")
    // // await sendCoin(RPC_ENDPOINT, wallet, receipientAddress, "", coin(100000000, stable_coin_denom))
    // // console.log("send sei to address:")
    // // await sendCoin(RPC_ENDPOINT, wallet, receipientAddress, "", coin(100000000, "usei"))
    // console.log(`query ${receipientAddress} balance:`)
    // await queryAddressBalance(LCD_ENDPOINT, receipientAddress, "")

}

main().catch(console.log);

