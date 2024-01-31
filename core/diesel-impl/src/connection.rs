use std::ops::{Deref, DerefMut};
use tokio::sync::MutexGuard;

use crate::internal::{PgConnection, PooledConn};

pub enum Connection<'a> {
    Pool(PooledConn),
    Test(MutexGuard<'a, PgConnection>),
}

impl<'a> Deref for Connection<'a> {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Pool(conn) => conn,
            Self::Test(conn) => conn,
        }
    }
}

impl<'a> DerefMut for Connection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Pool(conn) => conn,
            Self::Test(conn) => conn,
        }
    }
}
