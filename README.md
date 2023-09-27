# Curta 18 solver
This is solver for @horsefacts puzzle implemented in Rust.  
Consists of 2 functional pieces:
1. Bruteforcer for runtime code to match expected codeHash
2. Bundle submitter via flashbot mev share

## Usage
Imagine your seed for puzzle is `0x*00112233*44556677*8899aabb*ccddeeff`. In this case your lead contract should be returning 0x8899aabb as essence and your gold contract should be returing 0x00112233.  

In src/ you can find minimal huff contract to return your essence. You can include your essence and get the runtime code using huffc `huffc -r essence_template.huff`.
Otherwise you can just include your essence in the next runtime code `63` + 4 bytes of essence + `60e01b5f5260205ff3`.

To bruteforce necessary runtime code and deploy bytecode you should use next commands with your essences
`cargo run --bin curta-18-solver bruteforce-code-hash lead 63*8899aabb*60e01b5f5260205ff3`
`cargo run --bin curta-18-solver bruteforce-code-hash gold 63*00112233*60e01b5f5260205ff3`

Now you have to use deployment bytecodes to submit your bundle
Fill .env similar to .env.example, `PRIV_KEY` is the private key for your curta account and `REP_PRIV_KEY` is the key for flashbots bundle signer used for reputation. You can use Alchemy WS and HTTP rpc endpoints.

`cargo run --bin curta-18-solver submit-bundle 60118060093d393df3636cbf043d60e01b5f5260205ff37692e4 60128060093d393df363081fa55a60e01b5f5260205ff3012fffa5`

First parameter is lead deployment bytecode and second is gold deployment bytecode.
Program will try to submit bundle to flashbots on every block.

After successful bundle submission you can submit solution to curta using official UI
