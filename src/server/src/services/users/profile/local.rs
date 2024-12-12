use crate::extract::SessionUser;

pub struct LocalProfile;

impl LocalProfile {
    #[tracing::instrument(skip(self), name = "services.users.profile.me")]
    pub async fn perform(self, user: SessionUser) -> LocalProfileResponse {
        LocalProfileResponse { user }
    }
}

#[must_use]
pub struct LocalProfileResponse {
    pub user: SessionUser,
}
