use crate::base::BaseContext;
use crate::db::Db;
use crate::models::BountyCardDisplay;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

const PAGE_SIZE: u32 = 10;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    bounty_cards: Vec<BountyCardDisplay>,
    page_num: u32,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        maybe_page_num: Option<u32>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let page_num = maybe_page_num.unwrap_or(1);
        let bounty_cards = BountyCardDisplay::all_pending(&mut db, PAGE_SIZE, page_num)
            .await
            .map_err(|_| "failed to get pending bounties.")?;
        Ok(Context {
            base_context,
            flash,
            bounty_cards,
            page_num,
        })
    }
}

#[get("/?<page_num>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    page_num: Option<u32>,
    user: Option<User>,
    admin_user: AdminUser,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(flash, db, page_num, user, Some(admin_user))
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("viewpendingbounties", context))
}

pub fn view_pending_bounties_stage() -> AdHoc {
    AdHoc::on_ignite("Pending Bounties Stage", |rocket| async {
        rocket.mount("/view_pending_bounties", routes![index])
        // .mount("/bounty", routes![new])
    })
}
