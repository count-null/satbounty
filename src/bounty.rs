use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Bounty, BountyDisplay};
use crate::user_account::ActiveUser;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    bounty_display: BountyDisplay,
    user: Option<User>,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        bounty_id: &str,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let bounty_display = BountyDisplay::single_by_public_id(&mut db, bounty_id)
            .await
            .map_err(|_| "failed to get admin settings.")?;

        // Do not show bounty if it is not approved (unless user is seller or admin).
        if !(user.as_ref().map(|u| u.id()) == Some(bounty_display.bounty.user_id)
            || admin_user.is_some()
            || bounty_display.bounty.approved)
        {
            return Err("Bounty is not approved.".to_string());
        };

        Ok(Context {
            base_context,
            flash,
            bounty_display,
            user,
            admin_user,
        })
    }
}

#[put("/<id>/submit")]
async fn submit(
    id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match submit_bounty(&mut db, id, active_user.user).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/bounty", index(id))),
            "Marked as submitted".to_string(),
        )),
        Err(e) => {
            error_!("Mark submitted({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/bounty", index(id))), e))
        }
    }
}

async fn submit_bounty(db: &mut Connection<Db>, id: &str, user: User) -> Result<(), String> {
    let bounty = Bounty::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get bounty")?;
    if bounty.user_id != user.id() {
        return Err("Bounty belongs to a different user.".to_string());
    };
    if bounty.submitted {
        return Err("Bounty is already submitted.".to_string());
    };
    if bounty.approved {
        return Err("Bounty is already approved.".to_string());
    };

    Bounty::mark_as_submitted(db, id)
        .await
        .map_err(|_| "failed to update bounty")?;
    Ok(())
}

#[put("/<id>/approve")]
async fn approve(
    id: &str,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match approve_bounty(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/bounty", index(id))),
            "Marked as approved".to_string(),
        )),
        Err(e) => {
            error_!("Mark approved({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/bounty", index(id))), e))
        }
    }
}

async fn approve_bounty(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let bounty = Bounty::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get bounty")?;
    if !bounty.submitted {
        return Err("Bounty is not submitted.".to_string());
    };
    if bounty.viewed {
        return Err("Bounty is already viewed.".to_string());
    };

    Bounty::mark_as_approved(db, id)
        .await
        .map_err(|_| "failed to approve bounty")?;
    Ok(())
}

#[put("/<id>/reject")]
async fn reject(
    id: &str,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match reject_bounty(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/bounty", index(id))),
            "Marked as rejected".to_string(),
        )),
        Err(e) => {
            error_!("Mark rejected({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/bounty", index(id,))), e))
        }
    }
}

async fn reject_bounty(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let bounty = Bounty::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get bounty")?;
    if !bounty.submitted {
        return Err("bounty is not submitted.".to_string());
    };
    if bounty.viewed {
        return Err("bounty is already viewed.".to_string());
    };

    Bounty::mark_as_rejected(db, id)
        .await
        .map_err(|_| "failed to reject bounty")?;
    Ok(())
}

#[put("/<id>/deactivate_as_admin")]
async fn admin_deactivate(
    id: &str,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match deactivate_bounty_as_admin(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/bounty", index(id))),
            "Marked as deactivated by admin".to_string(),
        )),
        Err(e) => {
            error_!("Mark as deactivated error({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/bounty", index(id))), e))
        }
    }
}

#[put("/<id>/deactivate")]
async fn deactivate(
    id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match deactivate_bounty_as_seller(&mut db, id, active_user.user).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/bounty", index(id))),
            "Marked as deactivated by seller".to_string(),
        )),
        Err(e) => {
            error_!("Mark as deactivated error({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/bounty", index(id))), e))
        }
    }
}

async fn deactivate_bounty_as_seller(
    db: &mut Connection<Db>,
    id: &str,
    user: User,
) -> Result<(), String> {
    let bounty = Bounty::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get bounty")?;
    if bounty.user_id != user.id() {
        return Err("bounty belongs to a different user.".to_string());
    };
    if bounty.deactivated_by_seller || bounty.deactivated_by_admin {
        return Err("bounty is already deactivated.".to_string());
    };

    Bounty::mark_as_deactivated_by_seller(db, id)
        .await
        .map_err(|_| "failed to deactivate bounty by seller")?;
    Ok(())
}

async fn deactivate_bounty_as_admin(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let bounty = Bounty::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get bounty")?;
    if bounty.deactivated_by_seller || bounty.deactivated_by_admin {
        return Err("bounty is already deactivated.".to_string());
    };

    Bounty::mark_as_deactivated_by_admin(db, id)
        .await
        .map_err(|_| "failed to deactivate bounty by admin")?;
    Ok(())
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, id, flash, user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("bounty", context))
}

pub fn bounty_stage() -> AdHoc {
    AdHoc::on_ignite("bounty Stage", |rocket| async {
        rocket.mount(
            "/bounty",
            routes![index, submit, approve, reject, deactivate, admin_deactivate],
        )
    })
}
