use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Bounty, BountyDisplay};
use rocket::fairing::AdHoc;
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
    bounty_display: Option<BountyDisplay>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        bounty_id: &str,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let bounty_display = BountyDisplay::single_by_public_id(&mut db, bounty_id)
            .await
            .map_err(|_| "failed to get bounty display.")?;

        if bounty_display.bounty.user_id != user.id() && admin_user.is_none() {
            return Err("User does not have permission to delete bounty.".to_string());
        };

        Ok(Context {
            base_context,
            flash,
            bounty_display: Some(bounty_display),
        })
    }
}

#[delete("/<id>")]
async fn delete(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete_bounty(id, &mut db, user.clone(), admin_user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/")),
            "Bounty was deleted.",
        )),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/update_bounty_images", index(id))),
                "Failed to delete bounty image.",
            ))
        }
    }
}

async fn delete_bounty(
    bounty_id: &str,
    db: &mut Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let bounty = Bounty::single_by_public_id(&mut *db, bounty_id)
        .await
        .map_err(|_| "failed to get bounty")?;

    if bounty.user_id != user.id() && admin_user.is_none() {
        return Err("User does not have permission to delete bounty.".to_string());
    };

    Bounty::delete(bounty.id.unwrap(), &mut *db)
        .await
        .map_err(|_| "failed to delete bounty.".to_string())?;

    Ok(())
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, id, flash, user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("deletebounty", context))
}

pub fn delete_bounty_stage() -> AdHoc {
    AdHoc::on_ignite("Delete Bounty Stage", |rocket| async {
        rocket.mount("/delete_bounty", routes![index, delete])
    })
}
