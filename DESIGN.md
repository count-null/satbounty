MVP Design Criteria:

1. Users may register with username and password
1. New users must deposit sats as a bond to activate their account
1. MIN_BOND_PRICE is an admin setting
1. Users may add more sats to their bond 
1. Users may withdraw their bond when they deactivate their account 
1. Users have a reputation consisting of reputation metrics
1. The reputation metrics include: BOND_LEVEL, USER_ACTIVATION_TIMESTAMP, LAST_LOGIN, BOUNTIES_CLAIMED, BOUNTIES_AWARDED, BOUNTIES_ACTIVE, SATS_WON, SATS_PAID, SATS_ESCROW
1. Some actions may be constrained by a user's reputation metrics
1. Users may add a PGP public key to their profile
1. Users have an account balance
1. Users can widthdraw or deposit sats to credit their account balance
1. Users may post a bounty
1. A bounty has a description, satoshi reward, PGP required, and reputation filters
1. Bounties requiring PGP warns that the market will be unable to arbitrate 
1. Reward amount minimum and maximum is based on user reputation
1. NEW_BOUNTY_REWARD_FILTER is an admin setting 
1. A reputation filter is a constraint on a user's reputation metrics
1. To submit the bounty, user must deposit the reward to hold in escrow
1. An admin must approve every bounty before it is public
1. Users may browse public bounties
1. Users may increase the reward for any public bounty using sats from their account balance 
1. Users may submit a case for any eligible bounty
1. Bounties are eligible when the user can pass its reputation filters
1. A case is a markdown text submission that exibits evidence that the bounty is complete  
1. A user may submit multiple cases for a bounty 
1. Users pay a market fee to submit a case 
1. CASE_MARKET_FEE is an admin setting
1. Users can view case submissions for bounties they have created
1. Users can mark a case as awarded
1. Once a case is awarded, the reward is credited to the user who submitted the case
1. Their case is now public and shown alonside the bounty
1. If the case was PGP encrypted, do not show it publicly 
1. The bounty is marked as “completed” 
1. Non1.winning users are credited any case submission fee
1. Fees unable to return are credited to the admin 
1. The admin takes a fee from every reward distribution

Fun Future Plans:

1. Users may register with username and LNURL1.Auth
1. Use Lightning Escrow instead of custodial escrow

- Bounty poster generates preimage, sends hash of preimage to market, market creates hold invoice for initial bounty reward, bounty poster pays hold invoice, market sends hash to every case submission, case sumbission creates hold invoice for current reward amount, market pays hold invoice, some case is awarded by bounty creator and given the preimage, awarded case settles invoice with preimage, market settles bounty creation invoice with preimage. In this example there is a risk associated with reuse of the payment hash. Intermediate nodes on both paths may shortcut the preimage and disallow the webshop from receiving any money.

