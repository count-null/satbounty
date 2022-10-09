use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, MarketInfoInput};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    admin_settings: AdminSettings,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let admin_settings = AdminSettings::single(&mut db)
            .await
            .map_err(|_| "failed to get admin settings.")?;
        Ok(Context {
            base_context,
            flash,
            admin_settings,
        })
    }
}

#[post("/change", data = "<market_info_form>")]
async fn update(
    market_info_form: Form<MarketInfoInput>,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Flash<Redirect> {
    let market_info_input = market_info_form.into_inner();
    let new_market_info = market_info_input.market_info;

    match change_market_info(new_market_info, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_market_info", index())),
            "Market info successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_market_info", index())), e),
    }
}

async fn change_market_info(
    new_market_info: String,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    if new_market_info.is_empty() {
        return Err("Market info cannot be empty.".to_string());
    };
    AdminSettings::set_market_info(db, &new_market_info)
        .await
        .map_err(|_| "failed to update market info.")?;

    Ok(())
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, flash, user, Some(admin_user))
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("updatemarketinfo", context))
}

pub fn update_market_info_stage() -> AdHoc {
    AdHoc::on_ignite("Update Market Info Stage", |rocket| async {
        rocket.mount("/update_market_info", routes![index, update])
    })
}
