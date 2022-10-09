use crate::db::Db;
use crate::rocket::futures::TryFutureExt;
use crate::rocket::futures::TryStreamExt;
use crate::util;
use rocket::fs::TempFile;
use rocket::serde::Serialize;
use rocket_db_pools::{sqlx, Connection};
use sqlx::pool::PoolConnection;
use sqlx::Acquire;
use sqlx::Row;
use sqlx::Sqlite;
use std::future::Future;
use std::result::Result;

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Bounty {
    pub id: Option<i32>,
    pub public_id: String,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub price_sat: u64,
    pub fee_rate_basis_points: u32,
    pub submitted: bool,
    pub viewed: bool,
    pub approved: bool,
    pub deactivated_by_seller: bool,
    pub deactivated_by_admin: bool,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm)]
pub struct InitialBountyInfo {
    pub title: String,
    pub description: String,
    pub price_sat: Option<u64>,
}

#[derive(FromForm)]
pub struct FileUploadForm<'f> {
    pub file: TempFile<'f>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct BountyImage {
    pub id: Option<i32>,
    pub public_id: String,
    pub bounty_id: i32,
    pub image_data: Vec<u8>,
    pub is_primary: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct BountyDisplay {
    pub bounty: Bounty,
    pub images: Vec<BountyImageDisplay>,
    pub user: Option<RocketAuthUser>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct BountyCardDisplay {
    pub bounty: Bounty,
    pub image: Option<BountyImageDisplay>,
    pub user: RocketAuthUser,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct BountyCard {
    pub bounty: Bounty,
    pub image: Option<BountyImage>,
    pub user: RocketAuthUser,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct BountyImageDisplay {
    pub id: Option<i32>,
    pub public_id: String,
    pub bounty_id: i32,
    pub image_data_base64: String,
    pub is_primary: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct RocketAuthUser {
    pub id: Option<i32>,
    pub username: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AdminSettings {
    pub id: Option<i32>,
    pub market_name: String,
    pub market_info: String,
    pub fee_rate_basis_points: u32,
    pub user_bond_price_sat: u64,
    pub pgp_key: String,
    pub max_allowed_users: u64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UserSettings {
    pub id: Option<i32>,
    pub pgp_key: String,
}

#[derive(Debug, FromForm)]
pub struct MarketNameInput {
    pub market_name: String,
}

#[derive(Debug, FromForm)]
pub struct MarketInfoInput {
    pub market_info: String,
}

#[derive(Debug, FromForm)]
pub struct PGPInfoInput {
    pub pgp_key: String,
}

#[derive(Debug, FromForm)]
pub struct FeeRateInput {
    pub fee_rate_basis_points: Option<i32>,
}

#[derive(Debug, FromForm)]
pub struct UserBondPriceInput {
    pub user_bond_price_sat: Option<u64>,
}

#[derive(Debug, FromForm)]
pub struct MaxAllowedUsersInput {
    pub max_allowed_users: Option<u64>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Case {
    pub id: Option<i32>,
    pub public_id: String,
    pub quantity: u32,
    pub buyer_user_id: i32,
    pub seller_user_id: i32,
    pub bounty_id: i32,
    pub case_details: String,
    pub amount_owed_sat: u64,
    pub seller_credit_sat: u64,
    pub paid: bool,
    pub awarded: bool,
    pub canceled_by_seller: bool,
    pub canceled_by_buyer: bool,
    pub invoice_hash: String,
    pub invoice_payment_request: String,
    pub created_time_ms: u64,
    pub payment_time_ms: u64,
}

#[derive(Debug, FromForm, Clone)]
pub struct CaseInfo {
    pub quantity: Option<u32>,
    pub case_details: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct CaseCard {
    pub case: Case,
    pub bounty: Option<Bounty>,
    pub image: Option<BountyImage>,
    pub user: Option<RocketAuthUser>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AccountInfo {
    pub account_balance_sat: i64,
    pub num_unawarded_cases: u32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AdminInfo {
    pub num_pending_bounties: u32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AccountBalanceChange {
    pub amount_change_sat: i64,
    pub event_type: String,
    pub event_id: String,
    pub event_time_ms: u64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Withdrawal {
    pub id: Option<i32>,
    pub public_id: String,
    pub user_id: i32,
    pub amount_sat: u64,
    pub invoice_hash: String,
    pub invoice_payment_request: String,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm, Clone)]
pub struct WithdrawalInfo {
    pub invoice_payment_request: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SellerInfo {
    pub username: String,
    pub total_amount_sold_sat: u64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UserAccount {
    pub id: Option<i32>,
    pub public_id: String,
    pub user_id: i32,
    pub amount_owed_sat: u64,
    pub paid: bool,
    pub disabled: bool,
    pub invoice_hash: String,
    pub invoice_payment_request: String,
    pub created_time_ms: u64,
    pub payment_time_ms: u64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UserCard {
    pub user: RocketAuthUser,
    pub user_account: UserAccount,
}

impl Default for AdminSettings {
    fn default() -> AdminSettings {
        AdminSettings {
            id: None,
            market_name: "Sat Bounty".to_string(),
            market_info: "About this market...".to_string(),
            fee_rate_basis_points: 500,
            user_bond_price_sat: 1,
            pgp_key: "".to_string(),
            max_allowed_users: 10000,
        }
    }
}

impl Default for UserSettings {
    fn default() -> UserSettings {
        UserSettings {
            id: None,
            pgp_key: "".to_string(),
        }
    }
}

impl Bounty {
    /// Returns the id of the inserted row.
    pub async fn insert(
        bounty: Bounty,
        max_unapproved_bounties: u32,
        db: &mut Connection<Db>,
    ) -> Result<i32, String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        let price_sat: i64 = bounty.price_sat.try_into().unwrap();
        let created_time_ms: i64 = bounty.created_time_ms.try_into().unwrap();

        let insert_result = sqlx::query!(
            "INSERT INTO bounties (public_id, user_id, title, description, price_sat, fee_rate_basis_points, submitted, viewed, approved, deactivated_by_seller, deactivated_by_admin, created_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            bounty.public_id,
            bounty.user_id,
            bounty.title,
            bounty.description,
            price_sat,
            bounty.fee_rate_basis_points,
            bounty.submitted,
            bounty.viewed,
	        bounty.approved,
            bounty.deactivated_by_seller,
            bounty.deactivated_by_admin,
            created_time_ms,
        )
            .execute(&mut *tx)
            .await
            .map_err(|_| "failed to insert new bounty.")?;

        let num_unapproved_bounties = sqlx::query!(
            "
select
 COUNT(bounties.id) as num_unapproved_bounties
from
 bounties
WHERE
 bounties.user_id = ?
AND
 NOT bounties.approved
;",
            bounty.user_id,
        )
        .fetch_one(&mut *tx)
        .map_ok(|r| r.num_unapproved_bounties as u32)
        .await
        .map_err(|_| "failed to get count of unapproved bounties.")?;

        if num_unapproved_bounties > max_unapproved_bounties {
            return Err(format!(
                "more than {:?} unapproved bounties not allowed.",
                max_unapproved_bounties
            ));
        }

        tx.commit()
            .await
            .map_err(|_| "failed to commit transaction.")?;

        Ok(insert_result.last_insert_rowid() as _)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<Bounty, sqlx::Error> {
        let bounty = sqlx::query!("select * from bounties WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| Bounty {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                title: r.title,
                description: markdown::to_html(&r.description),
                price_sat: r.price_sat.try_into().unwrap(),
                fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                submitted: r.submitted,
                viewed: r.viewed,
		        approved: r.approved,
                deactivated_by_seller: r.deactivated_by_seller,
                deactivated_by_admin: r.deactivated_by_admin,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(bounty)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Bounty, sqlx::Error> {
        let bounty = sqlx::query!("select * from bounties WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Bounty {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                title: r.title,
                description: markdown::to_html(&r.description),
                price_sat: r.price_sat.try_into().unwrap(),
                fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                submitted: r.submitted,
                viewed: r.viewed,
		        approved: r.approved,
                deactivated_by_seller: r.deactivated_by_seller,
                deactivated_by_admin: r.deactivated_by_admin,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(bounty)
    }

    pub async fn mark_as_submitted(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE bounties SET submitted = true WHERE public_id = ?",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_approved(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE bounties SET viewed = true, approved = true WHERE public_id = ?",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_rejected(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE bounties SET viewed = true, approved = false WHERE public_id = ?",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_deactivated_by_seller(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE bounties
SET deactivated_by_seller = true
WHERE
 public_id = ?
AND
 approved
AND NOT (deactivated_by_seller OR deactivated_by_admin)
;",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_deactivated_by_admin(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE bounties
SET deactivated_by_admin = true
WHERE
 public_id = ?
AND
 approved
AND NOT (deactivated_by_seller OR deactivated_by_admin)
;",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn num_pending(db: &mut Connection<Db>) -> Result<u32, sqlx::Error> {
        let num_bounties = sqlx::query!(
            "
select
 COUNT(bounties.id) as num_pending_bounties
from
 bounties
WHERE
 bounties.submitted
AND
 NOT bounties.viewed
;",
        )
        .fetch_one(&mut **db)
        .map_ok(|r| r.num_pending_bounties as u32)
        .await?;

        Ok(num_bounties)
    }

    pub async fn delete(bounty_id: i32, db: &mut Connection<Db>) -> Result<usize, String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        let delete_result = sqlx::query!(
            "
DELETE FROM bounties
WHERE
 id = ?
;",
            bounty_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete bounty.")?;

        sqlx::query!(
            "
DELETE from bountyimages
WHERE
 bounty_id = ?
;",
            bounty_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete images for bounty.")?;

        tx.commit()
            .await
            .map_err(|_| "failed to commit transaction.")?;

        Ok(delete_result.rows_affected() as _)
        //Ok(())
    }
}

impl BountyImage {
    /// Returns the number of affected rows: 1.
    pub async fn insert(
        bountyimage: BountyImage,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let insert_result = sqlx::query!(
            "INSERT INTO bountyimages (public_id, bounty_id, image_data, is_primary) VALUES (?, ?, ?, ?)",
            bountyimage.public_id,
            bountyimage.bounty_id,
            bountyimage.image_data,
            bountyimage.is_primary,
        )
        .execute(&mut **db)
        .await?;

        Ok(insert_result.rows_affected() as _)
    }

    pub async fn all_for_bounty(
        db: &mut Connection<Db>,
        bounty_id: i32,
    ) -> Result<Vec<BountyImage>, sqlx::Error> {
        let bounty_images = sqlx::query!(
            "select * from bountyimages WHERE bounty_id = ? ORDER BY bountyimages.is_primary DESC;",
            bounty_id
        )
        .fetch(&mut **db)
        .map_ok(|r| BountyImage {
            id: r.id.map(|n| n.try_into().unwrap()),
            public_id: r.public_id,
            bounty_id: r.bounty_id.try_into().unwrap(),
            image_data: r.image_data,
            is_primary: r.is_primary,
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(bounty_images)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<BountyImage, sqlx::Error> {
        let bounty_image = sqlx::query!(
            "select * from bountyimages WHERE public_id = ?;",
            public_id
        )
        .fetch_one(&mut **db)
        .map_ok(|r| BountyImage {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            bounty_id: r.bounty_id.try_into().unwrap(),
            image_data: r.image_data,
            is_primary: r.is_primary,
        })
        .await?;

        Ok(bounty_image)
    }

    pub async fn mark_image_as_primary_by_public_id(
        db: &mut Connection<Db>,
        bounty_id: i32,
        image_id: &str,
    ) -> Result<usize, sqlx::Error> {
        // Set all images for bounty_id to not primary.
        let update_primary_result = sqlx::query!(
            "
UPDATE
 bountyimages
SET
 is_primary = (public_id = ?)
WHERE
 bounty_id = ?
;",
            image_id,
            bounty_id
        )
        .execute(&mut **db)
        .await?;

        Ok(update_primary_result.rows_affected() as _)
    }

    /// Returns the number of affected rows: 1.
    pub async fn delete_with_public_id(
        public_id: &str,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let delete_result =
            sqlx::query!("DELETE FROM bountyimages WHERE public_id = ?", public_id)
                .execute(&mut **db)
                .await?;

        Ok(delete_result.rows_affected() as _)
    }
}

impl RocketAuthUser {
    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<RocketAuthUser, sqlx::Error> {
        let rocket_auth_user = sqlx::query!("select id, email from users WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| RocketAuthUser {
                id: Some(r.id as i32),
                username: r.email.unwrap(),
            })
            .await?;

        Ok(rocket_auth_user)
    }

    pub async fn single_by_username(
        db: &mut Connection<Db>,
        username: String,
    ) -> Result<RocketAuthUser, sqlx::Error> {
        let rocket_auth_user =
            sqlx::query!("select id, email from users WHERE email = ?;", username)
                .fetch_one(&mut **db)
                .map_ok(|r| RocketAuthUser {
                    id: Some(r.id.unwrap() as i32),
                    username: r.email.unwrap(),
                })
                .await?;

        Ok(rocket_auth_user)
    }
}

impl BountyDisplay {
    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<BountyDisplay, sqlx::Error> {
        let bounty = Bounty::single_by_public_id(&mut *db, public_id).await?;
        let images = BountyImage::all_for_bounty(&mut *db, bounty.id.unwrap()).await?;
        let image_displays = images
            .iter()
            .map(|img| BountyImageDisplay {
                id: img.id,
                public_id: img.clone().public_id,
                bounty_id: img.bounty_id,
                image_data_base64: util::to_base64(&img.image_data),
                is_primary: img.is_primary,
            })
            .collect::<Vec<_>>();
        let rocket_auth_user = RocketAuthUser::single(&mut *db, bounty.user_id).await.ok();

        let bounty_display = BountyDisplay {
            bounty,
            images: image_displays,
            user: rocket_auth_user,
        };

        Ok(bounty_display)
    }
}

impl BountyCard {
    pub async fn all_active(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of awarded cases.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
INNER JOIN
 useraccounts
ON
 bounties.user_id = useraccounts.user_id
AND
 NOT useraccounts.disabled
WHERE
 bounties.approved
AND
 not (bounties.deactivated_by_seller OR bounties.deactivated_by_admin)
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
		            deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_deactivated(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 bounties.approved
AND
 (bounties.deactivated_by_seller OR bounties.deactivated_by_admin)
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
		            approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_pending(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of awarded cases.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 bounties.submitted
AND
 NOT bounties.viewed
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_unsubmitted_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of awarded cases.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 not bounties.submitted
AND
 users.id = ?
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_pending_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of awarded cases. 
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 bounties.submitted
AND
 NOT bounties.viewed
AND
 users.id = ?
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_rejected_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of awarded cases.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 not bounties.approved
AND
 NOT bounties.viewed
AND
 users.id = ?
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_admin.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_deactivated_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 bounties.approved
AND
 (bounties.deactivated_by_seller OR bounties.deactivated_by_admin)
AND
 users.id = ?
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_admin.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_active_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of awarded cases.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 bounties.approved
AND
 bounties.viewed
AND
 NOT (bounties.deactivated_by_seller OR bounties.deactivated_by_admin)
AND
 users.id = ?
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }

    pub async fn all_active_for_search_text(
        db: &mut Connection<Db>,
        search_text: &str,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of awarded cases.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        // let uppercase_search_term = search_text.to_owned().to_ascii_uppercase();
        let wildcard_search_term = format!("%{}%", search_text.to_ascii_uppercase());
        let bounty_cards =
            sqlx::query!("
select
 bounties.id, bounties.public_id, bounties.user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 bounties
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
INNER JOIN
 users
ON
 bounties.user_id = users.id
INNER JOIN
 useraccounts
ON
 bounties.user_id = useraccounts.user_id
AND
 NOT useraccounts.disabled
WHERE
 bounties.approved
AND
 bounties.viewed
AND
 NOT (bounties.deactivated_by_seller OR bounties.deactivated_by_admin)
AND
 (UPPER(bounties.title) like ? OR UPPER(bounties.description) like ?)
GROUP BY
 bounties.id
ORDER BY bounties.created_time_ms DESC
LIMIT ?
OFFSET ?
;", wildcard_search_term, wildcard_search_term, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Bounty {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                BountyCard {
                    bounty: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(bounty_cards)
    }
}

impl BountyCardDisplay {
    fn bounty_card_to_display(card: &BountyCard) -> BountyCardDisplay {
        BountyCardDisplay {
            bounty: card.bounty.clone(),
            image: card.image.clone().map(|image| BountyImageDisplay {
                id: image.id,
                public_id: image.public_id,
                bounty_id: image.bounty_id,
                image_data_base64: util::to_base64(&image.image_data),
                is_primary: image.is_primary,
            }),
            user: card.clone().user,
        }
    }

    pub async fn all_active(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards = BountyCard::all_active(db, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_deactivated(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards = BountyCard::all_deactivated(db, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_pending(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards = BountyCard::all_pending(db, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_unsubmitted_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards =
            BountyCard::all_unsubmitted_for_user(db, user_id, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_pending_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards =
            BountyCard::all_pending_for_user(db, user_id, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_rejected_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards =
            BountyCard::all_rejected_for_user(db, user_id, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_deactivated_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards =
            BountyCard::all_deactivated_for_user(db, user_id, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_active_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards =
            BountyCard::all_active_for_user(db, user_id, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }

    pub async fn all_active_for_search_text(
        db: &mut Connection<Db>,
        search_text: &str,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<BountyCardDisplay>, sqlx::Error> {
        let bounty_cards =
            BountyCard::all_active_for_search_text(db, search_text, page_size, page_num).await?;
        let bounty_card_displays = bounty_cards
            .iter()
            .map(BountyCardDisplay::bounty_card_to_display)
            .collect::<Vec<_>>();

        Ok(bounty_card_displays)
    }
}

impl AdminSettings {
    pub async fn single(db: &mut Connection<Db>) -> Result<AdminSettings, sqlx::Error> {
        let maybe_admin_settings = sqlx::query!("select * from adminsettings;")
            .fetch_optional(&mut **db)
            .map_ok(|maybe_r| {
                maybe_r.map(|r| AdminSettings {
                    id: Some(r.id.try_into().unwrap()),
                    market_name: r.market_name,
                    market_info: markdown::to_html(&r.market_info),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    user_bond_price_sat: r.user_bond_price_sat.try_into().unwrap(),
                    pgp_key: r.pgp_key,
                    max_allowed_users: r.max_allowed_users.try_into().unwrap(),
                })
            })
            .await?;

        let admin_settings = maybe_admin_settings.unwrap_or_default();

        Ok(admin_settings)
    }

    async fn insert_if_doesnt_exist(db: &mut Connection<Db>) -> Result<(), sqlx::Error> {
        let admin_settings = AdminSettings::default();
        let user_bond_price_sat_i64: i64 = admin_settings.user_bond_price_sat.try_into().unwrap();
        let max_allowed_users_i64: i64 = admin_settings.max_allowed_users.try_into().unwrap();
        sqlx::query!(
            "
INSERT INTO
 adminsettings (market_name, market_info, fee_rate_basis_points, user_bond_price_sat, pgp_key, max_allowed_users)
SELECT ?, ?, ?, ?, ?, ?
WHERE NOT EXISTS(SELECT 1 FROM adminsettings)
;",
            admin_settings.market_name,
            admin_settings.market_info,
            admin_settings.fee_rate_basis_points,
            user_bond_price_sat_i64,
            admin_settings.pgp_key,
            max_allowed_users_i64,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_market_name(
        db: &mut Connection<Db>,
        new_market_name: &str,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!("UPDATE adminsettings SET market_name = ?", new_market_name)
            .execute(&mut **db)
            .await?;

        Ok(())
    }

    pub async fn set_market_info(
        db: &mut Connection<Db>,
        new_market_info: &str,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!("UPDATE adminsettings SET market_info = ?", new_market_info)
            .execute(&mut **db)
            .await?;

        Ok(())
    }

    pub async fn set_fee_rate(
        db: &mut Connection<Db>,
        new_fee_rate_basis_points: i32,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!(
            "UPDATE adminsettings SET fee_rate_basis_points = ?",
            new_fee_rate_basis_points,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_user_bond_price(
        db: &mut Connection<Db>,
        new_user_bond_price_sat: u64,
    ) -> Result<(), sqlx::Error> {
        let user_bond_price_sat_i64: i64 = new_user_bond_price_sat.try_into().unwrap();

        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!(
            "UPDATE adminsettings SET user_bond_price_sat = ?",
            user_bond_price_sat_i64,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_pgp_key(
        db: &mut Connection<Db>,
        new_pgp_key: &str,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!("UPDATE adminsettings SET pgp_key = ?", new_pgp_key,)
            .execute(&mut **db)
            .await?;

        Ok(())
    }

    pub async fn set_max_allowed_users(
        db: &mut Connection<Db>,
        new_max_allowed_users: u64,
    ) -> Result<(), sqlx::Error> {
        let max_allowed_users_i64: i64 = new_max_allowed_users.try_into().unwrap();

        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!(
            "UPDATE adminsettings SET max_allowed_users = ?",
            max_allowed_users_i64,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }
}

impl UserSettings {
    pub async fn single(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<UserSettings, sqlx::Error> {
        let maybe_user_settings =
            sqlx::query!("select * from usersettings WHERE user_id = ?;", user_id,)
                .fetch_optional(&mut **db)
                .map_ok(|maybe_r| {
                    maybe_r.map(|r| UserSettings {
                        id: Some(r.id.try_into().unwrap()),
                        pgp_key: r.pgp_key,
                    })
                })
                .await?;
        let user_settings = maybe_user_settings.unwrap_or_default();

        Ok(user_settings)
    }

    /// Returns the number of affected rows: 1.
    async fn insert_if_doesnt_exist(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<(), sqlx::Error> {
        let user_settings = UserSettings::default();
        sqlx::query!(
            "
INSERT INTO
 usersettings (user_id, pgp_key)
SELECT ?, ?
WHERE NOT EXISTS(SELECT 1 FROM usersettings WHERE user_id = ?)
;",
            user_id,
            user_settings.pgp_key,
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_pgp_key(
        db: &mut Connection<Db>,
        user_id: i32,
        new_pgp_key: &str,
    ) -> Result<(), sqlx::Error> {
        UserSettings::insert_if_doesnt_exist(db, user_id).await?;

        sqlx::query!(
            "UPDATE usersettings SET pgp_key = ? WHERE user_id = ?;",
            new_pgp_key,
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }
}

impl Case {
    /// Returns the id of the inserted row.
    pub async fn insert(
        case: Case,
        max_unpaid_cases: u32,
        db: &mut Connection<Db>,
    ) -> Result<i32, String> {
        let amount_owed_sat: i64 = case.amount_owed_sat.try_into().unwrap();
        let seller_credit_sat: i64 = case.seller_credit_sat.try_into().unwrap();
        let created_time_ms: i64 = case.created_time_ms.try_into().unwrap();
        let payment_time_ms: i64 = case.payment_time_ms.try_into().unwrap();

        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        let insert_result = sqlx::query!(
            "INSERT INTO cases (public_id, buyer_user_id, seller_user_id, quantity, bounty_id, case_details, amount_owed_sat, seller_credit_sat, paid, awarded, canceled_by_seller, canceled_by_buyer, invoice_hash, invoice_payment_request, created_time_ms, payment_time_ms ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            case.public_id,
            case.buyer_user_id,
            case.seller_user_id,
            case.quantity,
            case.bounty_id,
            case.case_details,
            amount_owed_sat,
            seller_credit_sat,
            case.paid,
            case.awarded,
            case.canceled_by_seller,
            case.canceled_by_buyer,
            case.invoice_hash,
            case.invoice_payment_request,
            created_time_ms,
            payment_time_ms,
        )
            .execute(&mut *tx)
            .await
            .map_err(|_| "failed to insert case.")?;

        let num_unpaid_cases = sqlx::query!(
            "
select
 COUNT(cases.id) as num_unpaid_cases
from
 cases
WHERE
 cases.buyer_user_id = ?
AND
 NOT cases.paid
;",
            case.buyer_user_id,
        )
        .fetch_one(&mut *tx)
        .map_ok(|r| r.num_unpaid_cases as u32)
        .await
        .map_err(|_| "failed to get count of unpaid cases for buyer.")?;

        if num_unpaid_cases > max_unpaid_cases {
            return Err(format!(
                "more than {:?} unpaid cases not allowed.",
                max_unpaid_cases,
            ));
        }

        tx.commit()
            .await
            .map_err(|_| "failed to commit transaction.")?;

        Ok(insert_result.last_insert_rowid() as _)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<Case, sqlx::Error> {
        let case = sqlx::query!("select * from cases WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| Case {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                buyer_user_id: r.buyer_user_id.try_into().unwrap(),
                seller_user_id: r.seller_user_id.try_into().unwrap(),
                bounty_id: r.bounty_id.try_into().unwrap(),
                case_details: markdown::to_html(&r.case_details),
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                awarded: r.awarded,
                canceled_by_seller: r.canceled_by_seller,
                canceled_by_buyer: r.canceled_by_buyer,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(case)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Case, sqlx::Error> {
        let case = sqlx::query!("select * from cases WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Case {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                buyer_user_id: r.buyer_user_id.try_into().unwrap(),
                seller_user_id: r.seller_user_id.try_into().unwrap(),
                bounty_id: r.bounty_id.try_into().unwrap(),
                case_details: markdown::to_html(&r.case_details),
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                awarded: r.awarded,
                canceled_by_seller: r.canceled_by_seller,
                canceled_by_buyer: r.canceled_by_buyer,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(case)
    }

    pub async fn single_by_invoice_hash(
        db: &mut PoolConnection<Sqlite>,
        invoice_hash: &str,
    ) -> Result<Case, sqlx::Error> {
        let case = sqlx::query!("select * from cases WHERE invoice_hash = ?;", invoice_hash)
            .fetch_one(&mut **db)
            .map_ok(|r| Case {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                buyer_user_id: r.buyer_user_id.try_into().unwrap(),
                seller_user_id: r.seller_user_id.try_into().unwrap(),
                bounty_id: r.bounty_id.try_into().unwrap(),
                case_details: markdown::to_html(&r.case_details),
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                awarded: r.awarded,
                canceled_by_seller: r.canceled_by_seller,
                canceled_by_buyer: r.canceled_by_buyer,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(case)
    }

    pub async fn all_older_than(
        db: &mut PoolConnection<Sqlite>,
        created_time_ms: u64,
    ) -> Result<Vec<Case>, sqlx::Error> {
        let created_time_ms_i64: i64 = created_time_ms.try_into().unwrap();

        let cases = sqlx::query!(
            "
select *
from
 cases
WHERE
 created_time_ms < ?
AND
 NOT paid
;",
            created_time_ms_i64,
        )
        .fetch(&mut **db)
        .map_ok(|r| Case {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            quantity: r.quantity.try_into().unwrap(),
            buyer_user_id: r.buyer_user_id.try_into().unwrap(),
            seller_user_id: r.seller_user_id.try_into().unwrap(),
            bounty_id: r.bounty_id.try_into().unwrap(),
            case_details: markdown::to_html(&r.case_details),
            amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
            seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
            paid: r.paid,
            awarded: r.awarded,
            canceled_by_seller: r.canceled_by_seller,
            canceled_by_buyer: r.canceled_by_buyer,
            invoice_hash: r.invoice_hash,
            invoice_payment_request: r.invoice_payment_request,
            created_time_ms: r.created_time_ms.try_into().unwrap(),
            payment_time_ms: r.payment_time_ms.try_into().unwrap(),
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(cases)
    }

    pub async fn mark_as_paid(
        db: &mut PoolConnection<Sqlite>,
        case_id: i32,
        time_now_ms: u64,
    ) -> Result<(), sqlx::Error> {
        let time_now_ms_i64: i64 = time_now_ms.try_into().unwrap();

        sqlx::query!(
            "UPDATE cases SET paid = true, payment_time_ms = ? WHERE id = ?",
            time_now_ms_i64,
            case_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_awarded(
        db: &mut PoolConnection<Sqlite>,
        case_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE
 cases
SET
 awarded = true, case_details= ''
WHERE
 id = ?
AND
 paid
AND
 not (awarded OR canceled_by_seller OR canceled_by_buyer)
;",
            case_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_canceled_by_seller(
        db: &mut PoolConnection<Sqlite>,
        case_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE
 cases
SET
 canceled_by_seller = true, case_details= ''
WHERE
 id = ?
AND
 not (awarded OR canceled_by_seller OR canceled_by_buyer)
;",
            case_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_canceled_by_buyer(
        db: &mut PoolConnection<Sqlite>,
        case_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE
 cases
SET
 canceled_by_buyer = true, case_details= ''
WHERE
 id = ?
AND
 not (awarded OR canceled_by_seller OR canceled_by_buyer)
;",
            case_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn delete_expired_case(
        db: &mut PoolConnection<Sqlite>,
        case_id: i32,
        cancel_case_invoice_future: impl Future<
            Output = Result<tonic_openssl_lnd::invoicesrpc::CancelInvoiceResp, String>,
        >,
    ) -> Result<(), String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        sqlx::query!(
            "
DELETE FROM cases
WHERE
 id = ?
AND
 NOT paid
;",
            case_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete case from database.")?;

        cancel_case_invoice_future
            .await
            .map_err(|e| format!("failed to cancel case invoice: {:?}", e))?;

        tx.commit()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        Ok(())
    }

    pub async fn seller_info_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<SellerInfo, sqlx::Error> {
        let total_amount_sold_sat = sqlx::query(
            "
            select
             SUM(cases.amount_owed_sat) as total_amount_sold_sat
            FROM
             cases
            WHERE
             cases.awarded
            AND
             cases.seller_user_id = ?
            GROUP BY
             cases.seller_user_id
            ;",
        )
        .bind(user_id)
        .fetch_optional(&mut **db)
        .map_ok(|maybe_r| {
            maybe_r.map(|r| {
                let amount_sat_i64: i64 = r.try_get("total_amount_sold_sat").unwrap();
                let amount_sat_u64: u64 = amount_sat_i64.try_into().unwrap();
                amount_sat_u64
            })
        })
        .await?;

        let seller_info = SellerInfo {
            username: "".to_string(),
            total_amount_sold_sat: total_amount_sold_sat.unwrap_or(0),
        };

        // TODO: remove option from return type.
        Ok(seller_info)
    }

    pub async fn seller_info_for_all_users(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<SellerInfo>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let seller_infos = sqlx::query(
                    "
        SELECT total_amount_sold_sat, users.email
        FROM
         users
        LEFT JOIN
            (select
             SUM(cases.amount_owed_sat) as total_amount_sold_sat, cases.seller_user_id as awarded_seller_user_id
            FROM
             cases
            WHERE
             awarded 
            GROUP BY
             cases.seller_user_id) as seller_infos
        ON
         users.id = seller_infos.awarded_seller_user_id
        WHERE
         total_amount_sold_sat > 0
        ORDER BY
         total_amount_sold_sat DESC
        LIMIT ?
        OFFSET ?
            ;")
            .bind(limit)
            .bind(offset)
            .fetch(&mut **db)
            .map_ok(|r| {
                SellerInfo {
                    username: r.try_get("email").unwrap(),
                    total_amount_sold_sat: {
                        let amount_sat_i64: i64 = r.try_get("total_amount_sold_sat").unwrap();
                        let amount_sat_u64: u64 = amount_sat_i64.try_into().unwrap();
                        amount_sat_u64
                    },
                }})
            .try_collect::<Vec<_>>()
            .await?;

        Ok(seller_infos)
    }

    // TODO: implement this.
    pub async fn most_recent_paid_case(
        db: &mut PoolConnection<Sqlite>,
    ) -> Result<Option<String>, sqlx::Error> {
        let latest_paid_case_invoice_hash = sqlx::query!(
            "
SELECT
 invoice_hash
FROM
(SELECT invoice_hash, payment_time_ms FROM useraccounts
 UNION ALL
SELECT invoice_hash, payment_time_ms FROM cases)
WHERE
 payment_time_ms = (SELECT MAX(payment_time_ms) FROM
(SELECT invoice_hash, payment_time_ms FROM useraccounts
 UNION ALL
SELECT invoice_hash, payment_time_ms FROM cases))
LIMIT 1
;"
        )
        .fetch_optional(&mut **db)
        .map_ok(|maybe_r| maybe_r.map(|r| r.invoice_hash))
        .await?;

        Ok(latest_paid_case_invoice_hash)
    }

    pub async fn num_processing_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<u32, sqlx::Error> {
        let num_cases = sqlx::query!(
            "
select
 COUNT(cases.id) as num_processing_cases
from
 cases
WHERE
 cases.paid
AND
 not (cases.awarded OR cases.canceled_by_seller OR cases.canceled_by_buyer)
AND
 cases.seller_user_id = ?
;",
            user_id,
        )
        .fetch_one(&mut **db)
        .map_ok(|r| r.num_processing_cases as u32)
        .await?;

        Ok(num_cases)
    }
}

impl CaseCard {
    pub async fn all_unpaid_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<CaseCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let cases = sqlx::query!(
            "
select
 cases.id as case_id, cases.public_id as case_public_id, cases.buyer_user_id as case_buyer_user_id, cases.seller_user_id as case_seller_user_id, cases.quantity as case_quantity, cases.bounty_id as case_bounty_id, cases.case_details, cases.amount_owed_sat, cases.seller_credit_sat, cases.paid, cases.awarded, cases.canceled_by_seller, cases.canceled_by_buyer, cases.invoice_hash, cases.invoice_payment_request, cases.created_time_ms, cases.payment_time_ms, bounties.id, bounties.public_id as bounty_public_id, bounties.user_id as bounty_user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms as bounty_created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 cases
LEFT JOIN
 bounties
ON
 cases.bounty_id = bounties.id
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
LEFT JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 not cases.paid
AND
 case_buyer_user_id = ?
GROUP BY
 cases.id
ORDER BY cases.created_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Case {
                    id: Some(r.case_id.unwrap().try_into().unwrap()),
                    public_id: r.case_public_id.unwrap(),
                    quantity: r.case_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.case_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.case_seller_user_id.unwrap().try_into().unwrap(),
                    bounty_id: r.case_bounty_id.unwrap().try_into().unwrap(),
                    case_details: r.case_details.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    awarded: r.awarded.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|bounty_id| Bounty {
                    id: Some(bounty_id.try_into().unwrap()),
                    public_id: r.bounty_public_id.unwrap(),
                    user_id: r.bounty_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.bounty_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                CaseCard {
                    case: o,
                    bounty: l,
                    image: i,
                    user: u,
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(cases)
    }

    pub async fn all_paid_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<CaseCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let cases = sqlx::query!(
            "
select
 cases.id as case_id, cases.public_id as case_public_id, cases.buyer_user_id as case_buyer_user_id, cases.seller_user_id as case_seller_user_id, cases.quantity as case_quantity, cases.bounty_id as case_bounty_id, cases.case_details, cases.amount_owed_sat, cases.seller_credit_sat, cases.paid, cases.awarded, cases.canceled_by_seller, cases.canceled_by_buyer, cases.invoice_hash, cases.invoice_payment_request, cases.created_time_ms, cases.payment_time_ms, bounties.id, bounties.public_id as bounty_public_id, bounties.user_id as bounty_user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms as bounty_created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 cases
LEFT JOIN
 bounties
ON
 cases.bounty_id = bounties.id
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
LEFT JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 cases.paid
AND
 case_buyer_user_id = ?
GROUP BY
 cases.id
ORDER BY cases.payment_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Case {
                    id: Some(r.case_id.unwrap().try_into().unwrap()),
                    public_id: r.case_public_id.unwrap(),
                    quantity: r.case_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.case_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.case_seller_user_id.unwrap().try_into().unwrap(),
                    bounty_id: r.case_bounty_id.unwrap().try_into().unwrap(),
                    case_details: r.case_details.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    awarded: r.awarded.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|bounty_id| Bounty {
                    id: Some(bounty_id.try_into().unwrap()),
                    public_id: r.bounty_public_id.unwrap(),
                    user_id: r.bounty_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.bounty_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                CaseCard {
                    case: o,
                    bounty: l,
                    image: i,
                    user: u,
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(cases)
    }

    pub async fn all_received_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<CaseCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let cases = sqlx::query!(
            "
select
 cases.id as case_id, cases.public_id as case_public_id, cases.buyer_user_id as case_buyer_user_id, cases.seller_user_id as case_seller_user_id, cases.quantity as case_quantity, cases.bounty_id as case_bounty_id, cases.case_details, cases.amount_owed_sat, cases.seller_credit_sat, cases.paid, cases.awarded, cases.canceled_by_seller, cases.canceled_by_buyer, cases.invoice_hash, cases.invoice_payment_request, cases.created_time_ms, cases.payment_time_ms, bounties.id, bounties.public_id as bounty_public_id, bounties.user_id as bounty_user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms as bounty_created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 cases
LEFT JOIN
 bounties
ON
 cases.bounty_id = bounties.id
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
LEFT JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 cases.awarded
AND
 case_seller_user_id = ?
GROUP BY
 cases.id
ORDER BY cases.payment_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Case {
                    id: Some(r.case_id.unwrap().try_into().unwrap()),
                    public_id: r.case_public_id.unwrap(),
                    quantity: r.case_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.case_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.case_seller_user_id.unwrap().try_into().unwrap(),
                    bounty_id: r.case_bounty_id.unwrap().try_into().unwrap(),
                    case_details: r.case_details.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    awarded: r.awarded.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|bounty_id| Bounty {
                    id: Some(bounty_id.try_into().unwrap()),
                    public_id: r.bounty_public_id.unwrap(),
                    user_id: r.bounty_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.bounty_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                CaseCard {
                    case: o,
                    bounty: l,
                    image: i,
                    user: u,
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(cases)
    }

    pub async fn all_processing_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<CaseCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let cases = sqlx::query!(
            "
select
 cases.id as case_id, cases.public_id as case_public_id, cases.buyer_user_id as case_buyer_user_id, cases.seller_user_id as case_seller_user_id, cases.quantity as case_quantity, cases.bounty_id as case_bounty_id, cases.case_details, cases.amount_owed_sat, cases.seller_credit_sat, cases.paid, cases.awarded, cases.canceled_by_seller, cases.canceled_by_buyer, cases.invoice_hash, cases.invoice_payment_request, cases.created_time_ms, cases.payment_time_ms, bounties.id, bounties.public_id as bounty_public_id, bounties.user_id as bounty_user_id, bounties.title, bounties.description, bounties.price_sat, bounties.fee_rate_basis_points, bounties.submitted, bounties.viewed, bounties.approved, bounties.deactivated_by_seller, bounties.deactivated_by_admin, bounties.created_time_ms as bounty_created_time_ms, bountyimages.id as image_id, bountyimages.public_id as image_public_id, bountyimages.bounty_id, bountyimages.image_data, bountyimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 cases
LEFT JOIN
 bounties
ON
 cases.bounty_id = bounties.id
LEFT JOIN
 bountyimages
ON
 bounties.id = bountyimages.bounty_id
AND
 bountyimages.is_primary = (SELECT MAX(is_primary) FROM bountyimages WHERE bounty_id = bounties.id)
LEFT JOIN
 users
ON
 bounties.user_id = users.id
WHERE
 cases.paid
AND
 not (cases.awarded OR cases.canceled_by_seller OR cases.canceled_by_buyer)
AND
 cases.seller_user_id = ?
GROUP BY
 cases.id
ORDER BY cases.payment_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {

                let o = Case {
                    id: Some(r.case_id.unwrap().try_into().unwrap()),
                    public_id: r.case_public_id.unwrap(),
                    quantity: r.case_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.case_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.case_seller_user_id.unwrap().try_into().unwrap(),
                    bounty_id: r.case_bounty_id.unwrap().try_into().unwrap(),
                    case_details: r.case_details.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    awarded: r.awarded.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|bounty_id| Bounty {
                    id: Some(bounty_id.try_into().unwrap()),
                    public_id: r.bounty_public_id.unwrap(),
                    user_id: r.bounty_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    viewed: r.viewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.bounty_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| BountyImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    bounty_id: r.bounty_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                CaseCard {
                    case: o,
                    bounty: l,
                    image: i,
                    user: u,
                }

            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(cases)
    }
}

impl AccountInfo {
    pub async fn account_info_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<AccountInfo, sqlx::Error> {
        let account_balance_sat = AccountInfo::total_account_balance_for_user(db, user_id).await?;
        let num_unawarded_cases = Case::num_processing_for_user(db, user_id).await?;
        Ok(AccountInfo {
            account_balance_sat,
            num_unawarded_cases,
        })
    }

    pub async fn all_account_balance_changes_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<AccountBalanceChange>, sqlx::Error> {
        // TODO: Case by event time in SQL query. When this is fixed: https://github.com/launchbadge/sqlx/issues/1350
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let account_balance_changes = sqlx::query("
SELECT * FROM
(select cases.seller_user_id as user_id, cases.seller_credit_sat as amount_change_sat, 'received_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 cases.awarded
AND
 cases.seller_user_id = ?
UNION ALL
select cases.buyer_user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'refunded_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 (cases.canceled_by_seller OR cases.canceled_by_buyer)
AND
 cases.buyer_user_id = ?
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
WHERE
 withdrawals.user_id = ?)
ORDER BY event_time_ms DESC
LIMIT ?
OFFSET ?
;")
            .bind(user_id)
            .bind(user_id)
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch(&mut **db)
            .map_ok(|r| AccountBalanceChange {
                amount_change_sat: r.try_get("amount_change_sat").unwrap(),
                event_type: r.try_get("event_type").unwrap(),
                event_id: r.try_get("event_id").unwrap(),
                event_time_ms: {
                    let time_ms_i64: i64 = r.try_get("event_time_ms").unwrap();
                    time_ms_i64 as u64
                },
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(account_balance_changes)
    }

    pub async fn total_account_balance_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<i64, sqlx::Error> {
        let account_balance_sat = sqlx::query("
SELECT SUM(amount_change_sat) as total_account_balance_sat FROM
(select cases.seller_user_id as user_id, cases.seller_credit_sat as amount_change_sat, 'received_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 cases.awarded
AND
 cases.seller_user_id = ?
UNION ALL
select cases.buyer_user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'refunded_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 (cases.canceled_by_seller OR cases.canceled_by_buyer)
AND
 cases.buyer_user_id = ?
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
WHERE
 withdrawals.user_id = ?)
;")
            .bind(user_id)
            .bind(user_id)
            .bind(user_id)
            .fetch_one(&mut **db)
            .map_ok(|r|  {
                let balance_sat_i64: i64 = r.try_get("total_account_balance_sat").unwrap();
                balance_sat_i64
            })
            .await?;

        Ok(account_balance_sat)
    }

    pub async fn all_account_balance_changes(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<AccountBalanceChange>, sqlx::Error> {
        // TODO: Case by event time in SQL query. When this is fixed: https://github.com/launchbadge/sqlx/issues/1350
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let account_balance_changes = sqlx::query("
SELECT * FROM
(select cases.seller_user_id as user_id, cases.seller_credit_sat as amount_change_sat, 'received_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 cases.awarded
UNION ALL
select cases.buyer_user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'refunded_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 (cases.canceled_by_seller OR cases.canceled_by_buyer)
UNION ALL
select cases.buyer_user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'processing_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 NOT (cases.awarded OR cases.canceled_by_seller OR cases.canceled_by_buyer)
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
UNION ALL
select useraccounts.user_id as user_id, useraccounts.amount_owed_sat as amount_change_sat, 'user_activation' as event_type, useraccounts.public_id as event_id, useraccounts.created_time_ms as event_time_ms
from
 useraccounts
WHERE
 useraccounts.paid)
ORDER BY event_time_ms DESC
LIMIT ?
OFFSET ?
;")
            .bind(limit)
            .bind(offset)
            .fetch(&mut **db)
            .map_ok(|r| AccountBalanceChange {
                amount_change_sat: r.try_get("amount_change_sat").unwrap(),
                event_type: r.try_get("event_type").unwrap(),
                event_id: r.try_get("event_id").unwrap(),
                event_time_ms: {
                    let time_ms_i64: i64 = r.try_get("event_time_ms").unwrap();
                    time_ms_i64 as u64
                },
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(account_balance_changes)
    }

    pub async fn total_market_liabilities_sat(db: &mut Connection<Db>) -> Result<i64, sqlx::Error> {
        let market_liabilities_sat = sqlx::query("
SELECT SUM(amount_change_sat) as total_market_liabilities_sat FROM
(select cases.seller_user_id as user_id, cases.seller_credit_sat as amount_change_sat, 'received_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 cases.awarded
UNION ALL
select cases.buyer_user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'refunded_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 (cases.canceled_by_seller OR cases.canceled_by_buyer)
UNION ALL
select cases.buyer_user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'processing_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 NOT (cases.awarded OR cases.canceled_by_seller OR cases.canceled_by_buyer)
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
UNION ALL
select useraccounts.user_id as user_id, useraccounts.amount_owed_sat as amount_change_sat, 'user_activation' as event_type, useraccounts.public_id as event_id, useraccounts.created_time_ms as event_time_ms
from
 useraccounts
WHERE
 useraccounts.paid)
;")
            .fetch_one(&mut **db)
            .map_ok(|r|  {
                let balance_sat_i64: i64 = r.try_get("total_market_liabilities_sat").unwrap();
                balance_sat_i64
            })
            .await?;

        Ok(market_liabilities_sat)
    }

    // TODO: Use when sqlx is fixed.
    //     pub async fn account_balance(
    //         db: &mut Connection<Db>,
    //         user_id: i32,
    //     ) -> Result<u64, sqlx::Error> {
    //         let account_balance_result = sqlx::query!("
    // SELECT SUM(data.amount_change_sat) as account_balance, data.user_id as user_id
    // FROM
    // (select bounties.user_id as user_id, cases.seller_credit_sat as amount_change_sat, 'received_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
    // from
    //  cases
    // LEFT JOIN
    //  bounties
    // ON
    //  cases.bounty_id = bounties.id
    // WHERE
    //  cases.paid
    // AND
    //  cases.awarded
    // AND
    //  bounties.user_id = ?
    // UNION ALL
    // select cases.user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'refunded_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
    // from
    //  cases
    // WHERE
    //  cases.paid
    // AND
    //  not cases.awarded
    // AND
    //  cases.user_id = ?
    // ) data
    // GROUP BY user_id
    // ;",
    //         user_id, user_id)
    //         .fetch_optional(&mut **db)
    //         .await?;

    //         let account_balance = match account_balance_result {
    //             Some(r) => r.account_balance,
    //             None => 0,
    //         };

    //         Ok(account_balance)
    //     }
}

impl Withdrawal {
    pub async fn do_withdrawal(
        withdrawal: Withdrawal,
        db: &mut Connection<Db>,
        send_withdrawal_funds_future: impl Future<
            Output = Result<tonic_openssl_lnd::lnrpc::SendResponse, String>,
        >,
        max_withdrawals_per_interval: u32,
        interval_start_time_ms: u64,
    ) -> Result<i32, String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        // Insert the new withdrawal.
        let amount_sat: i64 = withdrawal.amount_sat.try_into().unwrap();
        let created_time_ms: i64 = withdrawal.created_time_ms.try_into().unwrap();
        let insert_result = sqlx::query!(
            "INSERT INTO withdrawals (public_id, user_id, amount_sat, invoice_hash, invoice_payment_request, created_time_ms) VALUES (?, ?, ?, ?, ?, ?)",
            withdrawal.public_id,
            withdrawal.user_id,
            amount_sat,
            withdrawal.invoice_hash,
            withdrawal.invoice_payment_request,
            created_time_ms,
        )
            .execute(&mut *tx)
            .await
            .map_err(|_| "failed to insert new withdrawal.")?;
        let new_withdrawal_id = insert_result.last_insert_rowid();

        // Check if any constraints are violated.
        let user_id = withdrawal.user_id;

        let start_time_ms_i64: i64 = interval_start_time_ms.try_into().unwrap();
        let withdrawal_count = sqlx::query!(
            "
select count(id) as withdrawal_count from withdrawals
WHERE
 user_id = ?
AND
 created_time_ms > ?
ORDER BY withdrawals.created_time_ms ASC;",
            user_id,
            start_time_ms_i64,
        )
        .fetch_one(&mut *tx)
        .map_ok(|r| r.withdrawal_count as u32)
        .await
        .map_err(|_| "failed to get withdrawal count.")?;

        if withdrawal_count > max_withdrawals_per_interval {
            return Err(format!(
                "More than {:?} withdrawals in a single day not allowed.",
                max_withdrawals_per_interval,
            ));
        }

        let account_balance_sat = sqlx::query("
SELECT SUM(amount_change_sat) as total_account_balance_sat FROM
(select cases.seller_user_id as user_id, cases.seller_credit_sat as amount_change_sat, 'received_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 cases.awarded
AND
 cases.seller_user_id = ?
UNION ALL
select cases.buyer_user_id as user_id, cases.amount_owed_sat as amount_change_sat, 'refunded_case' as event_type, cases.public_id as event_id, cases.created_time_ms as event_time_ms
from
 cases
WHERE
 cases.paid
AND
 (cases.canceled_by_seller OR cases.canceled_by_buyer)
AND
 cases.buyer_user_id = ?
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
WHERE
 withdrawals.user_id = ?)
;")
            .bind(user_id)
            .bind(user_id)
            .bind(user_id)
            .fetch_one(&mut *tx)
            .map_ok(|r|  {
                let balance_sat_i64: i64 = r.try_get("total_account_balance_sat").unwrap();
                balance_sat_i64
            })
            .await
            .map_err(|_| "failed to insert get account balance changes.")?;

        if account_balance_sat < 0 {
            return Err("Insufficient funds for withdrawal.".to_string());
        }

        let send_response = send_withdrawal_funds_future
            .await
            .map_err(|e| format!("failed to send withdrawal payment: {:?}", e))?;

        // Update the withdrawal row with the payment invoice hash.
        let payment_hash_hex = util::to_hex(&send_response.payment_hash);
        sqlx::query!(
            "UPDATE withdrawals SET invoice_hash = ? WHERE id = ?",
            payment_hash_hex,
            new_withdrawal_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to update new withdrawal payment hash.")?;

        tx.commit()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        Ok(new_withdrawal_id as _)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Withdrawal, sqlx::Error> {
        let withdrawal = sqlx::query!("select * from withdrawals WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Withdrawal {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                amount_sat: r.amount_sat.try_into().unwrap(),
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(withdrawal)
    }

    //     pub async fn count_for_user_since_time_ms(
    //         db: &mut Connection<Db>,
    //         user_id: i32,
    //         start_time_ms: u64,
    //     ) -> Result<u32, sqlx::Error> {
    //         let start_time_ms_i64: i64 = start_time_ms.try_into().unwrap();

    //         let withdrawal_count = sqlx::query!(
    //             "
    // select count(id) as withdrawal_count from withdrawals
    // WHERE
    //  user_id = ?
    // AND
    //  created_time_ms > ?
    // ORDER BY withdrawals.created_time_ms ASC;",
    //             user_id,
    //             start_time_ms_i64,
    //         )
    //         .fetch_one(&mut **db)
    //         .map_ok(|r| r.withdrawal_count)
    //         .await?;

    //         Ok(withdrawal_count.try_into().unwrap())
    //     }
}

impl AdminInfo {
    pub async fn admin_info(db: &mut Connection<Db>) -> Result<AdminInfo, sqlx::Error> {
        let num_pending_bounties = Bounty::num_pending(db).await?;
        Ok(AdminInfo {
            num_pending_bounties,
        })
    }
}

impl UserAccount {
    /// Returns the id of the inserted row.
    pub async fn insert(
        user_account: UserAccount,
        db: &mut Connection<Db>,
    ) -> Result<i32, sqlx::Error> {
        let amount_owed_sat: i64 = user_account.amount_owed_sat.try_into().unwrap();
        let created_time_ms: i64 = user_account.created_time_ms.try_into().unwrap();
        let payment_time_ms: i64 = user_account.payment_time_ms.try_into().unwrap();

        let insert_result = sqlx::query!(
            "INSERT INTO useraccounts (public_id, user_id, amount_owed_sat, paid, disabled, invoice_payment_request, invoice_hash, created_time_ms, payment_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            user_account.public_id,
            user_account.user_id,
            amount_owed_sat,
            user_account.paid,
            user_account.disabled,
            user_account.invoice_payment_request,
            user_account.invoice_hash,
            created_time_ms,
            payment_time_ms,
        )
            .execute(&mut **db)
            .await?;

        Ok(insert_result.last_insert_rowid() as _)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<UserAccount, sqlx::Error> {
        let user_account = sqlx::query!("select * from useraccounts WHERE user_id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| UserAccount {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                paid: r.paid,
                disabled: r.disabled,
                invoice_payment_request: r.invoice_payment_request,
                invoice_hash: r.invoice_hash,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(user_account)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<UserAccount, sqlx::Error> {
        let user_account =
            sqlx::query!("select * from useraccounts WHERE public_id = ?;", public_id)
                .fetch_one(&mut **db)
                .map_ok(|r| UserAccount {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                    paid: r.paid,
                    disabled: r.disabled,
                    invoice_payment_request: r.invoice_payment_request,
                    invoice_hash: r.invoice_hash,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.try_into().unwrap(),
                })
                .await?;

        Ok(user_account)
    }

    pub async fn single_by_invoice_hash(
        db: &mut PoolConnection<Sqlite>,
        invoice_hash: &str,
    ) -> Result<UserAccount, sqlx::Error> {
        let user_account = sqlx::query!(
            "select * from useraccounts WHERE invoice_hash = ?;",
            invoice_hash
        )
        .fetch_one(&mut **db)
        .map_ok(|r| UserAccount {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            user_id: r.user_id.try_into().unwrap(),
            amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
            paid: r.paid,
            disabled: r.disabled,
            invoice_payment_request: r.invoice_payment_request,
            invoice_hash: r.invoice_hash,
            created_time_ms: r.created_time_ms.try_into().unwrap(),
            payment_time_ms: r.payment_time_ms.try_into().unwrap(),
        })
        .await?;

        Ok(user_account)
    }

    pub async fn mark_as_paid(
        db: &mut PoolConnection<Sqlite>,
        user_account_id: i32,
        time_now_ms: u64,
    ) -> Result<(), sqlx::Error> {
        let time_now_ms_i64: i64 = time_now_ms.try_into().unwrap();

        sqlx::query!(
            "UPDATE useraccounts SET paid = true, payment_time_ms = ? WHERE id = ?",
            time_now_ms_i64,
            user_account_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_disabled(
        db: &mut PoolConnection<Sqlite>,
        user_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE useraccounts SET disabled = true WHERE user_id = ?",
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_enabled(
        db: &mut PoolConnection<Sqlite>,
        user_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE useraccounts SET disabled = false WHERE user_id = ?",
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn number_of_users(db: &mut Connection<Db>) -> Result<u64, sqlx::Error> {
        let num_users = sqlx::query!(
            "
select
 COUNT(users.id) as num_users
from
 users
WHERE
 NOT users.is_admin
;",
        )
        .fetch_one(&mut **db)
        .map_ok(|r| r.num_users as u64)
        .await?;

        Ok(num_users)
    }

    pub async fn all_older_than(
        db: &mut PoolConnection<Sqlite>,
        created_time_ms: u64,
    ) -> Result<Vec<UserAccount>, sqlx::Error> {
        let created_time_ms_i64: i64 = created_time_ms.try_into().unwrap();

        let user_accounts = sqlx::query!(
            "
select *
from
 useraccounts
WHERE
 created_time_ms < ?
AND
 NOT paid
;",
            created_time_ms_i64,
        )
        .fetch(&mut **db)
        .map_ok(|r| UserAccount {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            user_id: r.user_id.try_into().unwrap(),
            amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
            paid: r.paid,
            disabled: r.disabled,
            invoice_payment_request: r.invoice_payment_request,
            invoice_hash: r.invoice_hash,
            created_time_ms: r.created_time_ms.try_into().unwrap(),
            payment_time_ms: r.payment_time_ms.try_into().unwrap(),
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(user_accounts)
    }

    pub async fn delete_expired_user_account(
        db: &mut PoolConnection<Sqlite>,
        user_account_id: i32,
        cancel_user_account_invoice_future: impl Future<
            Output = Result<tonic_openssl_lnd::invoicesrpc::CancelInvoiceResp, String>,
        >,
    ) -> Result<(), String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        sqlx::query!(
            "
DELETE FROM useraccounts
WHERE
 user_id = ?
AND
 NOT paid
;",
            user_account_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user account from database.")?;

        sqlx::query!(
            "
DELETE FROM users
WHERE
 id = ?
;",
            user_account_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user from database.")?;

        cancel_user_account_invoice_future
            .await
            .map_err(|e| format!("failed to cancel user account invoice: {:?}", e))?;

        tx.commit()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        Ok(())
    }

    pub async fn delete_users_with_no_account(
        db: &mut PoolConnection<Sqlite>,
    ) -> Result<(), String> {
        sqlx::query!(
            "
DELETE FROM users
WHERE
 NOT is_admin
AND
 id IN
(SELECT users.id FROM users
LEFT JOIN
 useraccounts
ON
 users.id=useraccounts.user_id
WHERE
 useraccounts.user_id IS NULL);
;"
        )
        .execute(&mut **db)
        .await
        .map_err(|_| "failed to delete user account from database.")?;

        Ok(())
    }

    pub async fn do_deactivation(
        amount_sat: u64,
        user_account: UserAccount,
        db: &mut Connection<Db>,
        send_deactivation_funds_future: impl Future<
            Output = Result<tonic_openssl_lnd::lnrpc::SendResponse, String>,
        >,
    ) -> Result<(), String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        // Insert the new withdrawal.
        let activation_bond_amount_sat: i64 = user_account.amount_owed_sat.try_into().unwrap();
        let amount_sat: i64 = amount_sat.try_into().unwrap();
        let delete_user_account_result = sqlx::query!(
            "
DELETE FROM useraccounts
WHERE
 user_id = ?
AND
 paid = true
;",
            user_account.user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user account.")?;

        // Validate that at least one paid user account was deleted
        if delete_user_account_result.rows_affected() < 1 {
            return Err("No user bond found.".to_string());
        }

        sqlx::query!(
            "
DELETE FROM users
WHERE id = ?
;",
            user_account.user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user.")?;

        sqlx::query!(
            "
DELETE FROM bountyimages
WHERE
 bounty_id IN
(SELECT bounties.id FROM bounties
WHERE user_id = ?);
;",
            user_account.user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user bounty images.")?;

        sqlx::query!(
            "
DELETE FROM bounties
WHERE user_id = ?
;",
            user_account.user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user bounties.")?;

        if activation_bond_amount_sat - amount_sat < 0 {
            return Err("Insufficient funds for deactivation.".to_string());
        }

        send_deactivation_funds_future
            .await
            .map_err(|e| format!("failed to send deactivation payment: {:?}", e))?;

        tx.commit()
            .await
            .map_err(|_| "failed to commit transaction.")?;

        Ok(())
    }
}

impl UserCard {
    pub async fn all_active(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<UserCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let user_cards =
            sqlx::query!("
select
 users.id as rocket_auth_user_id, users.email as rocket_auth_user_username, useraccounts.id as useraccounts_id, useraccounts.public_id as useraccounts_public_id, useraccounts.user_id as useraccounts_user_id, useraccounts.amount_owed_sat as useraccounts_amount_owed_sat, useraccounts.paid as useraccounts_paid, useraccounts.disabled as useraccounts_disabled, useraccounts.invoice_payment_request as useraccounts_invoice_payment_request, useraccounts.invoice_hash as useraccounts_invoice_hash, useraccounts.created_time_ms as useraccounts_created_time_ms, useraccounts.payment_time_ms as useraccounts_payment_time_ms
from
 users
INNER JOIN
 useraccounts
ON
 users.id = useraccounts.user_id
WHERE
 useraccounts.paid
AND
 NOT useraccounts.disabled
GROUP BY
 users.id
ORDER BY useraccounts.created_time_ms DESC
LIMIT ?
OFFSET ?
;", limit, offset)
            .fetch(&mut **db)
            .map_ok(|r| {
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                let ua = r.useraccounts_id.map(|user_account_id| UserAccount {
                    id: Some(user_account_id.try_into().unwrap()),
                    public_id: r.useraccounts_public_id.unwrap(),
                    user_id: r.useraccounts_user_id.unwrap().try_into().unwrap(),
                    amount_owed_sat: r.useraccounts_amount_owed_sat.unwrap().try_into().unwrap(),
                    paid: r.useraccounts_paid.unwrap(),
                    disabled: r.useraccounts_disabled.unwrap(),
                    invoice_payment_request: r.useraccounts_invoice_payment_request.unwrap(),
                    invoice_hash: r.useraccounts_invoice_hash.unwrap(),
                    created_time_ms: r.useraccounts_created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.useraccounts_payment_time_ms.unwrap().try_into().unwrap(),
                });
                UserCard {
                    user: u.unwrap(),
                    user_account: ua.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(user_cards)
    }

    pub async fn all_disabled(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<UserCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let user_cards =
            sqlx::query!("
select
 users.id as rocket_auth_user_id, users.email as rocket_auth_user_username, useraccounts.id as useraccounts_id, useraccounts.public_id as useraccounts_public_id, useraccounts.user_id as useraccounts_user_id, useraccounts.amount_owed_sat as useraccounts_amount_owed_sat, useraccounts.paid as useraccounts_paid, useraccounts.disabled as useraccounts_disabled, useraccounts.invoice_payment_request as useraccounts_invoice_payment_request, useraccounts.invoice_hash as useraccounts_invoice_hash, useraccounts.created_time_ms as useraccounts_created_time_ms, useraccounts.payment_time_ms as useraccounts_payment_time_ms
from
 users
INNER JOIN
 useraccounts
ON
 users.id = useraccounts.user_id
WHERE
 useraccounts.paid
AND
 useraccounts.disabled
GROUP BY
 users.id
ORDER BY useraccounts.created_time_ms DESC
LIMIT ?
OFFSET ?
;", limit, offset)
            .fetch(&mut **db)
            .map_ok(|r| {
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                let ua = r.useraccounts_id.map(|user_account_id| UserAccount {
                    id: Some(user_account_id.try_into().unwrap()),
                    public_id: r.useraccounts_public_id.unwrap(),
                    user_id: r.useraccounts_user_id.unwrap().try_into().unwrap(),
                    amount_owed_sat: r.useraccounts_amount_owed_sat.unwrap().try_into().unwrap(),
                    paid: r.useraccounts_paid.unwrap(),
                    disabled: r.useraccounts_disabled.unwrap(),
                    invoice_payment_request: r.useraccounts_invoice_payment_request.unwrap(),
                    invoice_hash: r.useraccounts_invoice_hash.unwrap(),
                    created_time_ms: r.useraccounts_created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.useraccounts_payment_time_ms.unwrap().try_into().unwrap(),
                });
                UserCard {
                    user: u.unwrap(),
                    user_account: ua.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(user_cards)
    }
}
