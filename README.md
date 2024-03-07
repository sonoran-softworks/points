# On-chain Points

## TLDR
An on-chain mechanism for tracking and rewarding a community via points which can be exchanged for prizes.

## The Idea
Back in the day points style systems were largely panned, however as of late they've really taken off. Many projects implement their own flavor of a points system with some doing it off-chain and others building bespoke on-chain solutions (at least I think they do...I've not seen it with my own eyes). I wanted to develop a system to reward community members using a points system, but also wanted it to be on-chain...mostly because I didn't want to run a server+db and manage all that stuff for my intended use-case.

### An (intially) NFT-native Points System
At the core this contract provides the following base functionality for a points system:

- Track lifetime points accrued for a given address
- Track lifetime points spent for a given address
- Maintain a "prize pool" of cw-721 compatable NFTs
- Claim a prize from the pool
- An optional whitelist of addressess when may be awarded points

From this one could extend the functionality to support a number of use-cases. 

#### Claiming a prize
This contract uses [Nois](https://nois.network) as a source of randomness when fetching a prize from the pool when a user seeks to claim. This helps prevent "prize snipers" who would otherwise watch the pool for something they like and spam claim transaction to get it (likely botted).

### Roadmap
TBH there are some things I'd love to add:

- "Tiers" to the prize pool like you'd find at an old-school arcade/fun-center like Chuck E Cheese, such as `Tier 1 === 100 points`, `Tier 2 === 200 points`, `Tier 3 === 500 points` and so on.
- Cw20 token "baskets", such as a prize which is a bundle of 20 `$STARS` or 55 `$OSMO`
- Time-based gating to create windows when prizes can be claimed, sort of a "season" mechanic
- Prize pool rebalancing
