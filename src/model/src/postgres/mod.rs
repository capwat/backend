mod follower;
mod instance;
mod post;
mod users;

use sea_query::{Alias, Iden, IntoColumnRef, IntoIden, SelectExpr, SimpleExpr};

#[derive(sea_query::Iden)]
pub struct RegistrationMode;

fn into_view_aliases<
    A: Clone + Iden + 'static,
    B: Clone + Iden + 'static,
    T: Iterator<Item = (A, B)>,
>(
    iter: T,
) -> Vec<SelectExpr> {
    iter.map(|(a, b)| SelectExpr {
        expr: SimpleExpr::Column((a.clone(), b.clone()).into_column_ref()),
        alias: Some(Alias::new(format!("{}.{}", a.to_string(), b.to_string())).into_iden()),
        window: None,
    })
    .collect::<Vec<_>>()
}
