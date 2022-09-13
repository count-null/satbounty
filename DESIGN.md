MVP Design Criteria:

- Users may register with username and password
- New users must deposit sats as a bond to activate their account
- MIN_BOND_PRICE is an admin setting
- Users may add more sats to their bond 
- Users may withdraw their bond when they deactivate their account 
- Users have a reputation consisting of reputation metrics
- The reputation metrics include: BOND_LEVEL, USER_ACTIVATION_TIMESTAMP, LAST_LOGIN, BOUNTIES_CLAIMED, BOUNTIES_AWARDED, BOUNTIES_ACTIVE, SATS_WON, SATS_PAID, SATS_ESCROW
- Some actions may be constrained by a user's reputation metrics
- Users may add a PGP public key to their profile
- Users have an account balance
- Users can widthdraw or deposit sats to credit their account balance
- Users may post a bounty
- A bounty has a description, satoshi reward, PGP required, and reputation filters
- Bounties requiring PGP warns that the market will be unable to arbitrate 
- Reward amount minimum and maximum is based on user reputation
- NEW_BOUNTY_REWARD_FILTER is an admin setting 
- A reputation filter is a constraint on a user's reputation metrics
- To submit the bounty, user must deposit the reward to hold in escrow
- An admin must approve every bounty before it is public
- Users may browse public bounties
- Users may increase the reward for any public bounty using sats from their account balance 
- Users may submit a case for any eligible bounty
- Bounties are eligible when the user can pass its reputation filters
- A case is a markdown text submission that exibits evidence that the bounty is complete  
- A user may submit multiple cases for a bounty 
- Users pay a market fee to submit a case 
- CASE_MARKET_FEE is an admin setting
- Users can view case submissions for bounties they have created
- Users can mark a case as awarded
- Once a case is awarded, the reward is credited to the user who submitted the case
- The bounty is marked as “completed” 
- Non-winning users are credited any case submission fee
- Fees unable to return are credited to the admin 
- The admin takes a fee from every reward distribution

Fun Future Plans:

- Users may register with username and LNURL-Auth
- Use Lightning Escrow instead of custodial escrow

Bounty poster generates preimage, sends hash of preimage to market, market creates hold invoice for initial bounty reward, bounty poster pays hold invoice, market sends hash to every case submission, case sumbission creates hold invoice for current reward amount, market pays hold invoice, some case is awarded by bounty creator and given the preimage, awarded case settles invoice with preimage, market settles bounty creation invoice with preimage. In this example there is a risk associated with reuse of the payment hash. Intermediate nodes on both paths may shortcut the preimage and disallow the webshop from receiving any money.

