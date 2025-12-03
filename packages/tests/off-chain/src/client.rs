//! Abstraction specifically for the off-chain multi-test environment
pub mod proxy;
pub mod service_handler;
pub mod user_registry;
use app_client::{executor::AnyExecutor, querier::AnyQuerier};
use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{Addr, Api, CanonicalAddr, Coin};
use cw_multi_test::App;

#[derive(Clone)]
pub struct AppClient {
    pub querier: AnyQuerier,
    pub executor: AnyExecutor,
    inner: Rc<RefCell<App>>,
}

impl AppClient {
    pub fn new(admin: &str) -> Self {
        let app = Rc::new(RefCell::new(App::new(|router, api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &api.addr_make(admin),
                    vec![Coin {
                        denom: "utoken".to_string(),
                        amount: 1_000_000u128.into(),
                    }],
                )
                .unwrap();
        })));

        let admin = app.borrow().api().addr_make(admin);

        Self {
            querier: app.clone().into(),
            executor: (app.clone(), admin).into(),
            inner: app,
        }
    }

    pub fn with_app<T>(&self, f: impl FnOnce(&App) -> T) -> T {
        f(&self.inner.borrow())
    }

    pub fn with_app_mut<T>(&self, f: impl FnOnce(&mut App) -> T) -> T {
        f(&mut *self.inner.borrow_mut())
    }

    pub fn clone_app(&self) -> Rc<RefCell<App>> {
        match &self.executor {
            AnyExecutor::MultiTest { app, .. } => app.clone(),
            _ => unreachable!(),
        }
    }

    pub fn admin(&self) -> Addr {
        match &self.executor {
            AnyExecutor::MultiTest { admin, .. } => admin.clone(),
            _ => unreachable!(),
        }
    }

    pub fn admin_canonical(&self) -> CanonicalAddr {
        self.with_app(|app| app.api().addr_canonicalize(self.admin().as_str()).unwrap())
    }
}
