#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod about;
mod account;
mod account_activation;
mod activate_account;
mod active_users;
mod admin;
mod auth;
mod base;
mod config;
mod db;
mod deactivate_account;
mod deactivated_bounties;
mod delete_bounty;
mod disabled_users;
mod image_util;
mod lightning;
mod bounty;
mod bounties;
mod market_liabilities;
mod models;
mod my_account_balance;
mod my_active_bounties;
mod my_deactivated_bounties;
mod my_paid_cases;
mod my_pending_bounties;
mod my_processing_cases;
mod my_rejected_bounties;
mod my_unpaid_cases;
mod my_unsubmitted_bounties;
mod new_bounty;
mod case;
mod case_expiry;
mod payment_processor;
mod prepare_case;
mod view_pending_bounties;
mod routes;
mod search;
mod seller_history;
mod top_sellers;
mod update_fee_rate;
mod update_bounty_images;
mod update_market_name;
mod update_max_allowed_users;
mod update_pgp_info;
mod update_user_bond_price;
mod update_user_pgp_info;
mod user;
mod user_account;
mod user_account_expiry;
mod user_profile;
mod util;
mod withdraw;
mod withdrawal;

#[launch]
fn rocket() -> _ {
    let config_figment = config::Config::get_config();
    let config: config::Config = config_figment.extract().unwrap();
    println!("Starting with config: {:?}", config);

    let figment = rocket::Config::figment().merge((
        "databases.satbounty",
        rocket_db_pools::Config {
            url: config.clone().db_url,
            min_connections: None,
            max_connections: 1024,
            connect_timeout: 3,
            idle_timeout: None,
        },
    ));

    rocket::custom(figment).attach(routes::stage(config))
}
