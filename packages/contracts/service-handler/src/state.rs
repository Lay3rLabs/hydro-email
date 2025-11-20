use app_contract_api::service_handler::msg::{Email, EmailMessageOnly};
use cosmwasm_std::{Addr, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};

/// Only set if we take ServiceHandler interface
pub const SERVICE_MANAGER: Item<Addr> = Item::new("service_manager");
/// Only set in the test approach
pub const ADMIN: Item<Addr> = Item::new("admin");

const EMAILS_FROM: Map<(&str, u64), EmailMessageOnly> = Map::new("emails-from");
const EMAILS_IN_ORDER: Map<u64, Email> = Map::new("emails-in-order");
const EMAIL_ADDRESSES: Map<&str, ()> = Map::new("email-addresses");
const EMAIL_PAGINATION_ID_COUNT: Item<u64> = Item::new("email-pagination-id-count");

pub fn initialize(store: &mut dyn Storage) -> StdResult<()> {
    EMAIL_PAGINATION_ID_COUNT.save(store, &0u64)
}

pub fn push_email(store: &mut dyn Storage, email: &Email) -> StdResult<u64> {
    let pagination_id =
        EMAIL_PAGINATION_ID_COUNT.update(store, |id| -> StdResult<u64> { Ok(id + 1) })?;

    EMAILS_FROM.save(store, (&email.from, pagination_id), &email.into())?;
    EMAILS_IN_ORDER.save(store, pagination_id, email)?;
    EMAIL_ADDRESSES.save(store, &email.from, &())?;

    Ok(pagination_id)
}

pub fn list_email_addresses(
    store: &dyn Storage,
    start_after: Option<&str>,
    limit: Option<u32>,
) -> StdResult<Vec<String>> {
    let iter = EMAIL_ADDRESSES.range(
        store,
        start_after.map(Bound::exclusive),
        None,
        Order::Ascending,
    );

    let take_limit = limit.unwrap_or(u32::MAX) as usize;

    let addrs = iter
        .take(take_limit)
        .map(|item| item.map(|(addr, _)| addr.to_string()))
        .collect::<StdResult<Vec<String>>>()?;

    Ok(addrs)
}

pub fn list_emails_from(
    store: &dyn Storage,
    from: &str,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<(EmailMessageOnly, u64)>> {
    let iter = EMAILS_FROM.range(
        store,
        start_after.map(|id| Bound::exclusive((from, id))),
        None,
        Order::Ascending,
    );

    let take_limit = limit.unwrap_or(u32::MAX) as usize;

    let emails = iter
        .take(take_limit)
        .map(|item| item.map(|((_, id), email)| (email, id)))
        .collect::<StdResult<Vec<(EmailMessageOnly, u64)>>>()?;

    Ok(emails)
}

pub fn list_emails(
    store: &dyn Storage,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<(Email, u64)>> {
    let iter = EMAILS_IN_ORDER.range(
        store,
        start_after.map(Bound::exclusive),
        None,
        Order::Ascending,
    );

    let take_limit = limit.unwrap_or(u32::MAX) as usize;

    let emails = iter
        .take(take_limit)
        .map(|item| item.map(|(id, email)| (email, id)))
        .collect::<StdResult<Vec<(Email, u64)>>>()?;

    Ok(emails)
}
