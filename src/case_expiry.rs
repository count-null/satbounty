use crate::config::Config;
use crate::lightning::get_lnd_invoices_client;
use crate::models::Case;
use crate::util;
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;
use tonic_openssl_lnd::LndInvoicesClient;

const ORDER_EXPIRY_INTERVAL_MS: u64 = 86400000;

pub async fn remove_expired_cases(
    config: Config,
    mut conn: PoolConnection<Sqlite>,
) -> Result<(), String> {
    let mut lightning_invoices_client = get_lnd_invoices_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .map_err(|e| format!("failed to get lightning client: {:?}", e))?;

    // Get all cases older than expiry time limit.
    let now = util::current_time_millis();
    let expiry_cutoff = now - ORDER_EXPIRY_INTERVAL_MS;
    let expired_cases = Case::all_older_than(&mut conn, expiry_cutoff)
        .await
        .map_err(|_| "failed to expired cases.")?;

    for case in expired_cases {
        remove_case(&mut conn, &case, &mut lightning_invoices_client)
            .await
            .ok();
    }
    Ok(())
}

async fn remove_case(
    conn: &mut PoolConnection<Sqlite>,
    case: &Case,
    lightning_invoices_client: &mut LndInvoicesClient,
) -> Result<(), String> {
    println!("deleting expired case: {:?}", case);
    let cancel_case_invoice_ret = cancel_case_invoice(
        lightning_invoices_client,
        util::from_hex(&case.invoice_hash),
    );
    Case::delete_expired_case(conn, case.id.unwrap(), cancel_case_invoice_ret)
        .await
        .expect("failed to delete expired user account.");
    Ok(())
}

async fn cancel_case_invoice(
    lightning_invoices_client: &mut LndInvoicesClient,
    payment_hash: Vec<u8>,
) -> Result<tonic_openssl_lnd::invoicesrpc::CancelInvoiceResp, String> {
    let cancel_response = lightning_invoices_client
        .cancel_invoice(tonic_openssl_lnd::invoicesrpc::CancelInvoiceMsg { payment_hash })
        .await
        .expect("failed to cancel invoice")
        .into_inner();
    Ok(cancel_response)
}
