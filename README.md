# PixeLana Further
PixeLana is a multiplayer web app game where Stable Diffusion meets the Solana Blockchain. Players will take turns entering a prompt for the rest of the party to utilize the power of Stable Diffusion and generate images based on the prompt. The author of each prompt will then judge and select which art piece is the funniest, prettiest, or best image in the context of the prompt.

Winners will have their art minted in memory and as an NFT on the Solana Blockchain with fast confirmation times and low transaction fees.

## What's Next?

This is Frank's(main dev of PixeLana) personal fork of PixeLana that would be focusing on multi-room and more gartic-phone like game instead of going fully on chain, it is designed to be highly performant(backend rewritten in Rust), and would offer better UX once everything works!

## TODO

- replace hashmap with mongodb for scalability
- better handling user disconnect
- UX improvement
- draw ai integration
- transition
- host 
- ...


## Try it Out!
We are working fast to deploy our application! For eager players that would like to try the game **on the same network**:

1. Open a terminal
```
cd PixeLana_further
npm i
npm run dev
```

2. Open another terminal
```
cd PixeLana_further/backend_rs
cargo run
```

3. Have each player create a Wallet from any of the following providers: 
  - [Phantom](https://phantom.app/)
  - [OKX Wallet](https://chromewebstore.google.com/detail/okx-wallet/mcohilncbfahbmgdjkbpemcciiolgcge)
  - [Solflare](https://chromewebstore.google.com/detail/solflare-wallet/bhhhlbepdkbapadjdnnojkbgioiodbic)

  
4. Have each player **on the same network** access this link: http://localhost:3000/
