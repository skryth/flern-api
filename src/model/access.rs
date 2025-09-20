use crate::{
    model::{
        ModelManager,
        error::{DatabaseError, DatabaseResult},
    },
    web::{AuthenticatedUser, UserRole},
};

#[async_trait::async_trait]
pub trait HasOwner {
    type OwnerId: PartialEq + Send + Sync;
    async fn get_owner_id(
        &self,
        mm: &ModelManager,
        ctx: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId>;
}

pub async fn check_access<T: HasOwner<OwnerId = O>, O: PartialEq + Send + Sync>(
    mm: &ModelManager,
    ctx: &AuthenticatedUser,
    resource: &T,
    expected: O,
) -> DatabaseResult<()> {
    let actual_owner = resource.get_owner_id(mm, ctx).await?;

    // admin can get all resources
    if ctx.user_role() == UserRole::Admin {
        return Ok(());
    }

    if actual_owner == expected {
        Ok(())
    } else {
        Err(DatabaseError::Forbidden)
    }
}
