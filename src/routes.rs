use crate::config::Config;
use crate::db::Db;
use crate::case_expiry;
use crate::payment_processor;
use crate::user_account_expiry;
use rocket::fairing::{self, AdHoc};
use rocket::fs::{relative, FileServer};
use rocket::{Build, Rocket};
use rocket_auth::Error::SqlxError;
use rocket_auth::Users;
use rocket_db_pools::{sqlx, Database};
use rocket_dyn_templates::Template;

const PAYMENT_PROCESSOR_TASK_INTERVAL_S: u64 = 10;
const ORDER_EXPIRY_TASK_INTERVAL_S: u64 = 600;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

async fn create_users_table(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => {
            let users: Users = db.0.clone().into();
            match users.create_table().await {
                Ok(_) => Ok(rocket.manage(users)),
                Err(e) => {
                    error!("Failed to create users table: {}", e);
                    Err(rocket)
                }
            }
        }
        None => Err(rocket),
    }
}

async fn create_admin_user(rocket: Rocket<Build>, config: Config) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => {
            let users: Users = db.0.clone().into();
            // TODO: Delete all existing admins users here.
            let username = config.admin_username;
            let password = config.admin_password;
            match users.create_user(&username, &password, true).await {
                Ok(_) => Ok(rocket),
                Err(e) => match e {
                    SqlxError(_) => Ok(rocket),
                    _ => {
                        error!("Failed to create admin user: {}", e);
                        Err(rocket)
                    }
                },
            }
        }
        None => Err(rocket),
    }
}

pub fn stage(config: Config) -> AdHoc {
    let config_clone = config.clone();
    let config_clone_2 = config.clone();
    let config_clone_3 = config.clone();
    let config_clone_4 = config.clone();

    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(AdHoc::try_on_ignite("Manage config", |rocket| {
                Box::pin(async move { Ok(rocket.manage(config)) })
            }))
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .attach(AdHoc::try_on_ignite(
                "SQLx Create Users table",
                create_users_table,
            ))
            .attach(AdHoc::try_on_ignite("SQLx Create Admin User", |r| {
                create_admin_user(r, config_clone_2)
            }))
            .attach(AdHoc::on_liftoff("Process Payments", |rocket| {
                // Copied from: https://stackoverflow.com/a/72457117/1639564
                Box::pin(async move {
                    let pool = match Db::fetch(rocket) {
                        Some(pool) => pool.0.clone(), // clone the wrapped pool
                        None => panic!("failed to get db for background task."),
                    };
                    rocket::tokio::spawn(async move {
                        loop {
                            if let Ok(conn) = pool.acquire().await {
                                match payment_processor::handle_received_payments(
                                    config_clone.clone(),
                                    conn,
                                )
                                .await
                                {
                                    Ok(_) => println!(
                                        "payment processor task `completed` (shouldn't happen)."
                                    ),
                                    Err(e) => println!("payment processor task failed: {:?}", e),
                                }
                                println!(
                                    "Subscription failed. Trying again in {:?} seconds.",
                                    PAYMENT_PROCESSOR_TASK_INTERVAL_S
                                );
                                rocket::tokio::time::sleep(
                                    rocket::tokio::time::Duration::from_secs(
                                        PAYMENT_PROCESSOR_TASK_INTERVAL_S,
                                    ),
                                )
                                .await;
                            }
                        }
                    });
                })
            }))
            .attach(AdHoc::on_liftoff("Remove expired cases", |rocket| {
                Box::pin(async move {
                    let pool = match Db::fetch(rocket) {
                        Some(pool) => pool.0.clone(), // clone the wrapped pool
                        None => panic!("failed to get db for background task."),
                    };
                    rocket::tokio::spawn(async move {
                        let mut interval = rocket::tokio::time::interval(
                            rocket::tokio::time::Duration::from_secs(ORDER_EXPIRY_TASK_INTERVAL_S),
                        );
                        loop {
                            if let Ok(conn) = pool.acquire().await {
                                // Remove expired cases
                                match case_expiry::remove_expired_cases(
                                    config_clone_3.clone(),
                                    conn,
                                )
                                .await
                                {
                                    Ok(_) => (),
                                    Err(e) => println!("case expiry task failed: {:?}", e),
                                }
                            }
                            interval.tick().await;
                        }
                    });
                })
            }))
            .attach(AdHoc::on_liftoff(
                "Remove expired user accounts",
                |rocket| {
                    Box::pin(async move {
                        let pool = match Db::fetch(rocket) {
                            Some(pool) => pool.0.clone(), // clone the wrapped pool
                            None => panic!("failed to get db for background task."),
                        };
                        rocket::tokio::spawn(async move {
                            let mut interval = rocket::tokio::time::interval(
                                rocket::tokio::time::Duration::from_secs(
                                    ORDER_EXPIRY_TASK_INTERVAL_S,
                                ),
                            );
                            loop {
                                if let Ok(conn) = pool.acquire().await {
                                    // Remove expired cases
                                    match user_account_expiry::remove_expired_user_accounts(
                                        config_clone_4.clone(),
                                        conn,
                                    )
                                    .await
                                    {
                                        Ok(_) => (),
                                        Err(e) => {
                                            println!("user account expiry task failed: {:?}", e)
                                        }
                                    }
                                }
                                interval.tick().await;
                            }
                        });
                    })
                },
            ))
            .attach(Template::fairing())
            .mount("/", FileServer::from(relative!("static")))
            .attach(crate::about::about_stage())
            .attach(crate::auth::auth_stage())
            .attach(crate::admin::admin_stage())
            .attach(crate::active_users::active_users_stage())
            .attach(crate::disabled_users::disabled_users_stage())
            .attach(crate::activate_account::activate_account_stage())
            .attach(crate::account_activation::account_activation_stage())
            .attach(crate::deactivate_account::deactivate_account_stage())
            .attach(crate::market_liabilities::market_liabilities_stage())
            .attach(crate::bounties::bounties_stage())
            .attach(crate::deactivated_bounties::deactivated_bounties_stage())
            .attach(crate::bounty::bounty_stage())
            .attach(crate::new_bounty::new_bounty_stage())
            .attach(crate::update_bounty_images::update_bounty_images_stage())
            .attach(crate::user::user_stage())
            .attach(crate::user_profile::user_profile_stage())
            .attach(crate::update_market_name::update_market_name_stage())
            .attach(crate::update_market_info::update_market_info_stage())
            .attach(crate::update_fee_rate::update_fee_rate_stage())
            .attach(crate::update_user_bond_price::update_user_bond_price_stage())
            .attach(crate::update_pgp_info::update_pgp_info_stage())
            .attach(crate::update_max_allowed_users::update_max_allowed_users_stage())
            .attach(crate::update_user_pgp_info::update_user_pgp_info_stage())
            .attach(crate::view_pending_bounties::view_pending_bounties_stage())
            .attach(crate::delete_bounty::delete_bounty_stage())
            .attach(crate::account::account_stage())
            .attach(crate::my_unsubmitted_bounties::my_unsubmitted_bounties_stage())
            .attach(crate::my_pending_bounties::my_pending_bounties_stage())
            .attach(crate::my_active_bounties::my_active_bounties_stage())
            .attach(crate::my_rejected_bounties::my_rejected_bounties_stage())
            .attach(crate::my_deactivated_bounties::my_deactivated_bounties_stage())
            .attach(crate::my_unpaid_cases::my_unpaid_cases_stage())
            .attach(crate::my_paid_cases::my_paid_cases_stage())
            .attach(crate::my_account_balance::my_account_balance_stage())
            .attach(crate::my_processing_cases::my_processing_cases_stage())
            .attach(crate::prepare_case::prepare_case_stage())
            .attach(crate::case::case_stage())
            .attach(crate::withdraw::withdraw_stage())
            .attach(crate::withdrawal::withdrawal_stage())
            .attach(crate::seller_history::seller_history_stage())
            .attach(crate::top_sellers::top_sellers_stage())
            .attach(crate::search::search_stage())
    })
}
