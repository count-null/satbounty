use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{Bounty, Case, RocketAuthUser};
use crate::user_account::ActiveUser;
use crate::util;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket::State;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    case: Case,
    maybe_bounty: Option<Bounty>,
    maybe_seller_user: Option<RocketAuthUser>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    qr_svg_base64: String,
    lightning_node_pubkey: String,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        case_id: &str,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
        config: &Config,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let case = Case::single_by_public_id(&mut db, case_id)
            .await
            .map_err(|_| "failed to get case.")?;
        let maybe_bounty = Bounty::single(&mut db, case.bounty_id).await.ok();
        // .map_err(|_| "failed to get bounty.")?;
        // {
        //     Ok(bounty) => Some(bounty),
        //     Err(_) => None
        // };
        let maybe_seller_user = RocketAuthUser::single(&mut db, case.seller_user_id)
            .await
            .ok();
        let qr_svg_bytes = util::generate_qr(&case.invoice_payment_request);
        let qr_svg_base64 = util::to_base64(&qr_svg_bytes);
        let lightning_node_pubkey = get_lightning_node_pubkey(config)
            .await
            .unwrap_or_else(|_| "".to_string());
        Ok(Context {
            base_context,
            flash,
            case,
            maybe_bounty,
            maybe_seller_user,
            user,
            admin_user,
            qr_svg_base64,
            lightning_node_pubkey,
        })
    }
}

async fn get_lightning_node_pubkey(config: &Config) -> Result<String, String> {
    let mut lightning_client = lightning::get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let get_info_resp = lightning_client
        // All calls require at least empty parameter
        .get_info(tonic_openssl_lnd::lnrpc::GetInfoRequest {})
        .await
        .expect("failed to get lightning node info")
        .into_inner();
    Ok(get_info_resp.identity_pubkey)
}

#[put("/<id>/award")]
async fn award(
    id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_case_as_awarded(id, &mut db, active_user.user.clone(), admin_user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "case", id)),
            "Case has been awarded.",
        )),
        Err(e) => {
            error_!("DB update({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "case", id)),
                "Failed to mark case as awarded.",
            ))
        }
    }
}

async fn mark_case_as_awarded(
    case_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let case = Case::single_by_public_id(db, case_id)
        .await
        .map_err(|_| "failed to get case.")?;

    if case.seller_user_id != user.id() {
        return Err("User is not the case seller.".to_string());
    };
    if case.awarded {
        return Err("case has already been rewarded.".to_string());
    };
    if case.canceled_by_seller || case.canceled_by_buyer {
        return Err("case is already widthdrawn.".to_string());
    }

    Case::mark_as_awarded(&mut *db, case.id.unwrap())
        .await
        .map_err(|_| "failed to mark case as awarded.".to_string())
}

#[put("/<id>/seller_cancel")]
async fn seller_cancel(
    id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_case_as_canceled_by_seller(
        id,
        &mut db,
        active_user.user.clone(),
        admin_user.clone(),
    )
    .await
    {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "case", id)),
            "Case marked as canceled by seller.",
        )),
        Err(e) => {
            error_!("DB update({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "case", id)),
                "Failed to mark case as canceled by seller.",
            ))
        }
    }
}

async fn mark_case_as_canceled_by_seller(
    case_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let case = Case::single_by_public_id(db, case_id)
        .await
        .map_err(|_| "failed to get case.")?;

    if case.seller_user_id != user.id() {
        return Err("User is not the case seller.".to_string());
    };
    if case.awarded {
        return Err("case has already been awarded.".to_string());
    };
    if case.canceled_by_seller || case.canceled_by_buyer {
        return Err("case has already been canceled.".to_string());
    }

    Case::mark_as_canceled_by_seller(&mut *db, case.id.unwrap())
        .await
        .map_err(|_| "failed to mark case as canceled by seller.".to_string())
}

#[put("/<id>/buyer_cancel")]
async fn buyer_cancel(
    id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_case_as_canceled_by_buyer(id, &mut db, active_user.user.clone(), admin_user.clone())
        .await
    {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "case", id)),
            "Case marked as canceled by buyer.",
        )),
        Err(e) => {
            error_!("DB update({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "case", id)),
                "Failed to mark case as canceled by buyer.",
            ))
        }
    }
}

async fn mark_case_as_canceled_by_buyer(
    case_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let case = Case::single_by_public_id(db, case_id)
        .await
        .map_err(|_| "failed to get case.")?;

    if case.buyer_user_id != user.id() {
        return Err("User is not the case buyer.".to_string());
    };
    if case.awarded{
        return Err("case has already been rewarded".to_string());
    };
    if case.canceled_by_seller || case.canceled_by_buyer {
        return Err("case is already canceled.".to_string());
    };

    Case::mark_as_canceled_by_buyer(&mut *db, case.id.unwrap())
        .await
        .map_err(|_| "failed to mark case as canceled by buyer.".to_string())
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, id, flash, user, admin_user, config)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("case", context))
}

pub fn case_stage() -> AdHoc {
    AdHoc::on_ignite("Case Stage", |rocket| async {
        rocket.mount(
            "/case",
            routes![index, award, seller_cancel, buyer_cancel],
        )
    })
}
