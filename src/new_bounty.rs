use crate::base::BaseContext;
use crate::db::Db;
use crate::models::AdminSettings;
use crate::models::{InitialBountyInfo, Bounty};
use crate::user_account::ActiveUser;
use crate::util;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

const MAX_UNAPPROVED_BOUNTIES: u32 = 5;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    admin_settings: AdminSettings,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let admin_settings = AdminSettings::single(&mut db)
            .await
            .map_err(|_| "failed to update market name.")?;
        Ok(Context {
            base_context,
            flash,
            admin_settings,
        })
    }
}

#[post("/", data = "<bounty_form>")]
async fn new(
    bounty_form: Form<InitialBountyInfo>,
    mut db: Connection<Db>,
    active_user: ActiveUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let bounty_info = bounty_form.into_inner();

    match create_bounty(bounty_info, &mut db, active_user.user.clone()).await {
        Ok(bounty_id) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "bounty", bounty_id)),
            "Bounty successfully added.",
        )),
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(Flash::error(Redirect::to(uri!("/new_bounty", index())), e))
        }
    }
}

async fn create_bounty(
    bounty_info: InitialBountyInfo,
    db: &mut Connection<Db>,
    user: User,
) -> Result<String, String> {
    let admin_settings = AdminSettings::single(db)
        .await
        .map_err(|_| "failed to update market name.")?;
    let now = util::current_time_millis();

    let price_sat = bounty_info.price_sat.unwrap_or(0);

    if bounty_info.title.is_empty() {
        return Err("Title cannot be empty.".to_string());
    };
    if bounty_info.description.is_empty() {
        return Err("Description cannot be empty.".to_string());
    };
    if bounty_info.title.len() > 64 {
        return Err("Title length is too long.".to_string());
    };
    if bounty_info.description.len() > 4096 {
        return Err("Description length is too long.".to_string());
    };
    if price_sat == 0 {
        return Err("Price must be a positive number.".to_string());
    };
    if user.is_admin {
        return Err("Admin user cannot create a bounty.".to_string());
    };

    let bounty = Bounty {
        id: None,
        public_id: util::create_uuid(),
        user_id: user.id(),
        title: bounty_info.title,
        description: bounty_info.description,
        price_sat,
        fee_rate_basis_points: admin_settings.fee_rate_basis_points,
        submitted: false,
        viewed: false,
    	approved: false,
        deactivated_by_seller: false,
        deactivated_by_admin: false,
        created_time_ms: now,
    };
    match Bounty::insert(bounty, MAX_UNAPPROVED_BOUNTIES, db).await {
        Ok(bounty_id) => match Bounty::single(db, bounty_id).await {
            Ok(new_bounty) => Ok(new_bounty.public_id),
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("New bounty could not be found after inserting.".to_string())
            }
        },
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(e)
        }
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(flash, db, Some(active_user.user), admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("newbounty", context))
}

pub fn new_bounty_stage() -> AdHoc {
    AdHoc::on_ignite("New Bounty Stage", |rocket| async {
        rocket.mount("/new_bounty", routes![index, new])
    })
}
