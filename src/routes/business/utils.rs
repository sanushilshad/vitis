use super::models::UserBusinessRelationAccountModel;
use super::schemas::{BasicBusinessAccount, BusinessAccount, UserBusinessInvitation};
use super::{
    errors::BusinessAccountError, models::BusinessAccountModel, schemas::CreateBusinessAccount,
};
use crate::email::EmailObject;
use crate::routes::business::models::UserBusinessInvitationModel;
use crate::routes::user::schemas::{
    AuthenticationScope, BulkAuthMechanismInsert, UserAccount, UserRoleType, UserVector, VectorType,
};
use crate::routes::user::utils::get_role;
use crate::schemas::{MaskingType, Status};
use anyhow::Context;
use anyhow::anyhow;
use chrono::Utc;
use sqlx::{Executor, PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;
#[tracing::instrument(name = "create user account")]
pub fn create_vector_from_business_account(
    business_account: &CreateBusinessAccount,
) -> Vec<UserVector> {
    let vector_list: Vec<UserVector> = vec![
        UserVector {
            key: VectorType::Email,
            value: business_account.email.get().to_string(),
            masking: MaskingType::NA,
            verified: false,
        },
        UserVector {
            key: VectorType::MobileNo,
            value: business_account.get_full_mobile_no(),
            masking: MaskingType::NA,
            verified: false,
        },
    ];
    vector_list
}

#[tracing::instrument(name = "create user business relation", skip(transaction))]
pub async fn save_business_account(
    transaction: &mut Transaction<'_, Postgres>,
    user_account: &UserAccount,
    create_business_obj: &CreateBusinessAccount,
) -> Result<uuid::Uuid, BusinessAccountError> {
    let business_account_id = Uuid::new_v4();
    let business_name = create_business_obj.name.clone(); // Assuming this maps to `name`
    let vector_list = create_vector_from_business_account(create_business_obj); // JSONB column

    let query = sqlx::query!(
        r#"
        INSERT INTO business_account (id, display_name, vectors, created_by, created_on, is_active, email)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        business_account_id,
        business_name,
        sqlx::types::Json(vector_list) as sqlx::types::Json<Vec<UserVector>>,
        user_account.id,
        Utc::now(),
        Status::Active as Status,
        create_business_obj.email.get()
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        BusinessAccountError::DatabaseError(
            "A database failure occurred while saving business account".to_string(),
            e.into(),
        )
    })?;

    Ok(business_account_id)
}

#[tracing::instrument(name = "create user business relation", skip(transaction))]
pub async fn save_user_business_relation(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    business_id: Uuid,
    role_id: Uuid,
) -> Result<uuid::Uuid, BusinessAccountError> {
    let user_role_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO business_user_relationship (id, user_id, business_id, role_id, created_by, created_on, verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_role_id,
        user_id,
        business_id,
        role_id,
        user_id,
        Utc::now(),
        true
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        BusinessAccountError::DatabaseError(
            "A database failure occured while saving user business relation".to_string(),
            e.into(),
        )
    })?;
    Ok(user_role_id)
}

#[tracing::instrument(name = "Save Auth Mechanism for business account")]
pub fn prepare_auth_mechanism_data_for_business_account(
    user_id: Uuid,
    user_account: &CreateBusinessAccount,
) -> BulkAuthMechanismInsert {
    let current_utc = Utc::now();

    // Prepare data for auth mechanism
    let id = vec![Uuid::new_v4(), Uuid::new_v4()];
    let user_id_list = vec![user_id, user_id];
    let auth_scope = vec![AuthenticationScope::Otp, AuthenticationScope::Email];
    let auth_identifier = vec![
        user_account.get_full_mobile_no(),
        user_account.email.get().to_owned(),
    ];
    let secret = vec![];
    let is_active = vec![Status::Active, Status::Active];
    let created_on = vec![current_utc, current_utc];
    let created_by = vec![user_id, user_id];
    // let auth_context = vec![
    //     AuthContextType::businessAccount,
    //     AuthContextType::businessAccount,
    // ];

    BulkAuthMechanismInsert {
        id,
        user_id_list,
        auth_scope,
        auth_identifier,
        secret,
        is_active,
        created_on,
        created_by,
        // auth_context,
    }
}
#[tracing::instrument(name = "create business account", skip(pool))]
pub async fn create_business_account(
    pool: &PgPool,
    user_account: &UserAccount,
    create_business_obj: &CreateBusinessAccount,
) -> Result<uuid::Uuid, BusinessAccountError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let business_account_id =
        save_business_account(&mut transaction, user_account, create_business_obj).await?;
    if let Some(role_obj) = get_role(pool, &UserRoleType::Admin.to_lowercase_string()).await? {
        if role_obj.is_deleted || role_obj.role_status == Status::Inactive {
            return Err(BusinessAccountError::InvalidRoleError(
                "Role is deleted / Inactive".to_string(),
            ));
        }
        save_user_business_relation(
            &mut transaction,
            user_account.id,
            business_account_id,
            role_obj.id,
        )
        .await?;
    } else {
        tracing::info!("Invalid role for business account");
    }

    // let bulk_auth_data =
    //     prepare_auth_mechanism_data_for_business_account(user_account.id, create_business_obj);
    // save_auth_mechanism(&mut transaction, bulk_auth_data).await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new business account.")?;
    Ok(business_account_id)
}

#[tracing::instrument(name = "Get business Account By User id", skip(pool))]
pub async fn fetch_business_account_models_by_user_account(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<BusinessAccountModel>, anyhow::Error> {
    let rows = sqlx::query_as!(
        BusinessAccountModel,
        r#"
        SELECT 
            ba.id, 
            ba.display_name, 
            ba.vectors as "vectors:sqlx::types::Json<Vec<UserVector>>",
            ba.is_active as "is_active!:Status",
            ba.is_deleted,
            bur.verified
        FROM business_user_relationship AS bur
        INNER JOIN business_account ba ON bur.business_id = ba.id
        WHERE bur.user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[tracing::instrument(name = "Get business Account", skip(pool))]
pub async fn fetch_business_account_model_by_id(
    pool: &PgPool,
    business_id: Option<Uuid>,
) -> Result<Vec<BusinessAccountModel>, anyhow::Error> {
    use sqlx::QueryBuilder;

    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT 
            id,
            display_name, 
            vectors,
            is_active,
            is_deleted,
            TRUE AS verified
        FROM  business_account
        "#,
    );
    if let Some(business_id) = business_id {
        query_builder.push(" WHERE id = ");
        query_builder.push_bind(business_id);
    }

    let query = query_builder.build_query_as::<BusinessAccountModel>();

    let row = query.fetch_all(pool).await?;
    Ok(row)
}

#[tracing::instrument(name = "Get business Account By User Id", skip(pool))]
pub async fn fetch_associated_business_account_model(
    user_id: Uuid,
    business_account_id: Uuid,
    pool: &PgPool,
) -> Result<Option<UserBusinessRelationAccountModel>, anyhow::Error> {
    let row = sqlx::query_as!(
        UserBusinessRelationAccountModel,
        r#"
        SELECT 
            ba.id, 
            ba.display_name,
            ba.vectors as "vectors:sqlx::types::Json<Vec<UserVector>>",
            ba.is_active as "is_active!:Status",
            bur.verified, 
            ba.is_deleted
        FROM business_user_relationship AS bur
        INNER JOIN business_account AS ba ON bur.business_id = ba.id
        WHERE bur.user_id = $1
          AND bur.business_id = $2
        "#,
        user_id,
        business_account_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
#[tracing::instrument(name = "Get business Account by business id", skip(pool))]
pub async fn get_business_account(
    pool: &PgPool,
    user_id: Uuid,
    business_account_id: Uuid,
) -> Result<Option<BusinessAccount>, anyhow::Error> {
    let business_account_model =
        fetch_associated_business_account_model(user_id, business_account_id, pool).await?;
    match business_account_model {
        Some(model) => {
            let business_account = model.into_schema();
            Ok(Some(business_account))
        }
        None => Ok(None),
    }
}

#[tracing::instrument(name = "Get Basic business account by user id")]
pub async fn get_basic_business_accounts_by_user_id(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<BasicBusinessAccount>, anyhow::Error> {
    let business_account_models =
        fetch_business_account_models_by_user_account(pool, user_id).await?;
    let business_account_list = business_account_models
        .into_iter()
        .map(|e| e.into_basic_schema())
        .collect();

    Ok(business_account_list)
}

// test case added
pub fn validate_business_account_active(business_obj: &BusinessAccount) -> Option<String> {
    match (
        &business_obj.is_active,
        business_obj.is_deleted,
        business_obj.verified,
    ) {
        (Status::Inactive, _, _) => Some("business Account is inactive".to_string()),
        (_, true, _) => Some("business Account is deleted".to_string()),
        (_, _, false) => Some("business User relation is not verified".to_string()),
        _ => None,
    }
}
pub async fn validate_user_business_permission(
    pool: &PgPool,
    user_id: Uuid,
    business_id: Uuid,
    action_list: &Vec<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let permission_list = sqlx::query_scalar!(
        r#"
        SELECT  p.permission_name
        FROM business_user_relationship bur
        INNER JOIN role_permission rp ON bur.role_id = rp.role_id
        INNER JOIN permission p ON rp.permission_id = p.id
        WHERE bur.user_id = $1
          AND bur.business_id = $2
          AND rp.is_deleted = FALSE
          AND p.is_deleted = FALSE
          AND p.permission_name = ANY($3::text[])
        "#,
        user_id,
        business_id,
        action_list
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while fetching permission to database")
    })?;

    Ok(permission_list)
}

#[tracing::instrument(name = "Get Basic business account by user id")]
pub async fn get_basic_business_accounts(
    pool: &PgPool,
) -> Result<Vec<BasicBusinessAccount>, anyhow::Error> {
    let business_account_models = fetch_business_account_model_by_id(pool, None).await?;
    let business_account_list = business_account_models
        .into_iter()
        .map(|e| e.into_basic_schema())
        .collect();

    Ok(business_account_list)
}

#[tracing::instrument(name = "associate user to business", skip(pool))]
pub async fn associate_user_to_business(
    pool: &PgPool,
    user_id: Uuid,
    business_id: Uuid,
    role_id: Uuid,
    created_by: Uuid,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO business_user_relationship 
        (id, user_id, business_id, role_id, verified, created_on, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        Uuid::new_v4(),
        user_id,
        business_id,
        role_id,
        false,
        Utc::now(),
        created_by
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while associating user to business")
    })?;
    Ok(())
}

pub async fn validate_user_permission(
    pool: &PgPool,
    user_id: Uuid,
    action_list: &Vec<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let permission_list = sqlx::query_scalar!(
        r#"
        SELECT  p.permission_name
        FROM user_role bur
        INNER JOIN role_permission rp ON bur.role_id = rp.role_id
        INNER JOIN permission p ON rp.permission_id = p.id
        WHERE bur.user_id = $1
          AND rp.is_deleted = FALSE
          AND p.is_deleted = FALSE
          AND p.permission_name = ANY($2::text[])
        "#,
        user_id,
        action_list
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while fetching permission to database")
    })?;

    Ok(permission_list)
}

#[tracing::instrument(name = "save invite request", skip(transaction))]
pub async fn save_business_invite_request(
    transaction: &mut Transaction<'_, Postgres>,
    created_by: Uuid,
    business_id: Uuid,
    role_id: Uuid,
    email: &EmailObject,
) -> Result<uuid::Uuid, anyhow::Error> {
    // let business_account_id = Uuid::new_v4();

    let query = sqlx::query!(
        r#"
        INSERT INTO business_account_invitation_request (id, email, business_id, role_id, created_on, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (email, business_id)
        DO UPDATE SET
            created_by = EXCLUDED.created_by,
            created_on = EXCLUDED.created_on
        RETURNING id
        "#,
        Uuid::new_v4(),
        email.get(),
        business_id,
        role_id,
        Utc::now(),
        created_by
    );

    // let result = query.fetch_one(&mut *transaction).await.map_err(|e| {
    //     tracing::error!("Failed to execute query: {:?}", e);
    //     BusinessAccountError::DatabaseError(
    //         "A database failure occurred while saving business account invite requst".to_string(),
    //         e.into(),
    //     )
    // })?;
    let result = query.fetch_one(&mut **transaction).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving RFQ to database request")
    })?;

    Ok(result.id)
}

pub async fn fetch_business_models(
    pool: &PgPool,
    email: Option<&EmailObject>,
    business_id: Option<Uuid>,
    created_by: Option<Uuid>,
    id_list: Option<Vec<Uuid>>,
) -> Result<Vec<UserBusinessInvitationModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::<Postgres>::new(
        "SELECT id, verified, email, business_id, role_id, created_on, created_by FROM business_account_invitation_request WHERE 1=1",
    );

    if let Some(business_id) = business_id {
        query_builder
            .push(" AND business_id = ")
            .push_bind(business_id);
    }

    if let Some(created_by) = created_by {
        query_builder
            .push(" AND created_by = ")
            .push_bind(created_by);
    }

    if let Some(email) = email {
        query_builder.push(" AND email = ").push_bind(email.get());
    }

    if let Some(id_list) = id_list {
        if !id_list.is_empty() {
            query_builder.push(" AND id = ANY(");
            query_builder.push_bind(id_list);
            query_builder.push(")");
        }
    }

    let query = query_builder.build_query_as::<UserBusinessInvitationModel>();

    let rows = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while fetching business invite")
    })?;

    Ok(rows)
}
pub async fn fetch_business_invite(
    pool: &PgPool,
    email: Option<&EmailObject>,
    business_id: Option<Uuid>,
    created_by: Option<Uuid>,
    id_list: Option<Vec<Uuid>>,
) -> Result<Vec<UserBusinessInvitation>, anyhow::Error> {
    let models = fetch_business_models(pool, email, business_id, created_by, id_list).await?;
    let business_invite_list = models.into_iter().map(|e| e.into_schema()).collect();
    Ok(business_invite_list)
}

#[tracing::instrument(name = "mark invite as verified", skip(transaction))]
pub async fn mark_invite_as_verified(
    transaction: &mut Transaction<'_, Postgres>,
    invite_id: Uuid,
    updated_by: Uuid,
    updated_on: chrono::DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE business_account_invitation_request
        SET verified = TRUE,
        updated_by = $1,
        updated_on = $2
        WHERE id = $3
        "#,
        updated_by,
        updated_on,
        invite_id,
    );

    query.execute(&mut **transaction).await.map_err(|e| {
        tracing::error!("Failed to mark invite as verified: {:?}", e);
        anyhow::anyhow!(e).context("Failed to update 'verified' field for invite")
    })?;

    Ok(())
}

#[tracing::instrument(name = "delete business invite", skip(pool))]
pub async fn delete_invite_by_id(pool: &PgPool, invite_id: Uuid) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM business_account_invitation_request
        WHERE id = $1
        "#,
        invite_id
    );

    query.execute(pool).await.map_err(|e| {
        tracing::error!("Failed to delete invite: {:?}", e);
        anyhow!(e).context("Failed to delete invite from business_account_invitation_request")
    })?;

    Ok(())
}
