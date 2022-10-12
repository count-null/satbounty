# SatBounty 

An open source labor market to incentivize and monetize work.

Create and claim bounties paid in sats over the lightning network. 
Vote with sats to raise the reward of other bounties.

## How it works

A registered user creates a brief for the project deposits the initial reward upfront.
Other users may read the bounty and increase the reward by making deposits of their own.
Another user can submit a claim that they completed the bounty.
The bounty creator may award any claim they choose. The current bounty reward is given to the claimant (minus market fees). 

## Why it works

1. Reputation and Trust

Users build reputation by hunting bounties and creating them. 
Users also have "bonding sats" held with the market.
The bond can only be withdrawn by user deactivation. 
The amount held in bond can affect the user's percieved trust. 

2. Full Transparency

There is no single reputation score. 
Rather statistics are displayed for every user that summarize their activity on the platform.
Each user makes a judgement whether they feel certain the bounty poster will fairly reward them for work.

3. Admin Moderation & Market Fees

The admin must approve every bounty before it's published.
The admin is responsible for hosting SatBounty and complying with local regulations.
The admin is responsible for the custody of funds.

The admin may terminate a user without disbursement of their bond. 
Or subtract from their bond as punishment.  

Market fees are earned each time a bounty is awarded. 
This fee rewards the admin for providing a safe and secure marketplace. 


## Ways to break it 

1. Sock Puppets 

Users can create duplicate accounts and award themselves a bounty they created.
The market fee discorages cheaters. However, it's possible that 
a bounty reward increases significantly beyond the inital deposit and even after paying market fees, the cheater gains sats.

2. Unfair Awarding

A user could feel cheated if they did the work but didn't win the award.
Maybe someone else got the award, or maybe the bounty poster is unresponsive. 
Their only recourse is report to the admin who will investigate case-by-case.

3. Targeted Attacks

Anytime you expose something to the public, you are inviting attacks.
When hosting SatBounty, do not use a lightning node connected with your identity or location.
For extra protection, serve the website only as a tor or I2P hidden service. 

## Installation

### Requirements
* an LND node
* Rust and Cargo
* openssl `apt install libssl-dev`
* gexiv2 `apt install libgexiv2-dev`
* compiler dependencies `apt install libprotobuf-dev protobuf-compiler cmake`

### Step 1. Create the configuration
> Copy **config.example** file to **config.toml** and edit relevant sections to connect to your LND node:

```
db_url="db.sqlite"
admin_username="admin"
admin_password="pass"
lnd_host="localhost"
lnd_port=10009
lnd_tls_cert_path="~/.lnd/tls.cert"
lnd_macaroon_path="~/.lnd/data/chain/bitcoin/mainnet/admin.macaroon"
```

### Step 2. Start satbounty:

```
cargo run
```

Go to http://localhost:8000/ and use the username/password in **config.toml** to log in.

## Test

```
cargo test
```

## Front-end

Satbounty uses SASS/SCSS only. No JavaScript. 

Sass is compiled on `cargo run` using [grass](https://docs.rs/grass/latest/grass/)

For development, run `sass --watch static/scss/style.scss static/css/style.css`

## Database Migrations

Use [sqlx-cli](https://crates.io/crates/sqlx-cli/).

`cargo install sqlx-cli`

`cargo sqlx migrate --source db/migrations add <YOUR_MIGRATION_NAME>`

Then put your SQL changes in the new file.

`cargo sqlx migrate --source db/migrations run`

After running migrations, generate the schema for compile-time type-checking:

`cargo sqlx prepare --database-url sqlite3://db.sqlite`

Optional: create a `.env` with `DATABASE_URL=sqlite3://db.sqlite` to avoid passing `--database-url`

## License

Distributed under the MIT License. See [LICENSE file](LICENSE).
