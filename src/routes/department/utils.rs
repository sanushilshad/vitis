use anyhow::Context;
use chrono::Utc;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    routes::{
        business::schemas::BusinessAccount,
        department::models::UserDepartmentRelationAccountModel,
        role::utils::get_role,
        user::schemas::{UserAccount, UserRoleType},
    },
    schemas::Status,
};

use super::{
    errors::DepartmentAccountError,
    models::DepartmentAccountModel,
    schemas::{BasicDepartmentAccount, CreateDepartmentAccount, DepartmentAccount},
};

#[tracing::instrument(name = "create user department relation", skip(transaction))]
pub async fn save_department_account(
    transaction: &mut Transaction<'_, Postgres>,
    user_account: &UserAccount,
    create_department_obj: &CreateDepartmentAccount,
) -> Result<uuid::Uuid, DepartmentAccountError> {
    let department_account_id = Uuid::new_v4();
    let department_name = create_department_obj.display_name.clone(); // Assuming this maps to `name`

    let query = sqlx::query!(
        r#"
        INSERT INTO department_account (id, display_name, created_by, created_on)
        VALUES ($1, $2, $3, $4)
        "#,
        department_account_id,
        department_name,
        user_account.id,
        Utc::now()
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        DepartmentAccountError::DatabaseError(
            "A database failure occurred while saving department account".to_string(),
            e.into(),
        )
    })?;

    Ok(department_account_id)
}
#[tracing::instrument(name = "create user department relation", skip(transaction))]
pub async fn save_user_department_relation(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    business_id: Uuid,
    department_id: Uuid,
    role_id: Uuid,
) -> Result<uuid::Uuid, DepartmentAccountError> {
    let user_role_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO business_user_department_relationship (id, user_id, business_id, department_id, role_id, created_by, created_on)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_role_id,
        user_id,
        business_id,
        department_id,
        role_id,
        user_id,
        Utc::now(),
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        DepartmentAccountError::DatabaseError(
            "A database failure occured while saving user department relation".to_string(),
            e.into(),
        )
    })?;
    Ok(user_role_id)
}

#[tracing::instrument(name = "create department account", skip(pool))]
pub async fn create_department_account(
    pool: &PgPool,
    user_account: &UserAccount,
    business_account: &BusinessAccount,
    create_department_obj: &CreateDepartmentAccount,
) -> Result<uuid::Uuid, DepartmentAccountError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let department_account_id =
        save_department_account(&mut transaction, user_account, create_department_obj).await?;
    if let Some(role_obj) = get_role(pool, &UserRoleType::Admin.to_lowercase_string()).await? {
        if role_obj.is_deleted || role_obj.status == Status::Inactive {
            return Err(DepartmentAccountError::InvalidRoleError(
                "Role is deleted / Inactive".to_string(),
            ));
        }
        save_user_department_relation(
            &mut transaction,
            user_account.id,
            business_account.id,
            department_account_id,
            role_obj.id,
        )
        .await?;
    } else {
        tracing::info!("Invalid role for department account");
    }

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new department account.")?;
    Ok(department_account_id)
}

#[tracing::instrument(name = "Get department Account By User id", skip(pool))]
pub async fn fetch_department_account_models_by_user_account(
    pool: &PgPool,
    user_id: Uuid,
    business_id: Uuid,
) -> Result<Vec<DepartmentAccountModel>, anyhow::Error> {
    let rows = sqlx::query_as!(
        DepartmentAccountModel,
        r#"
        SELECT
            ba.id,
            ba.display_name,
            ba.is_active as "is_active!:Status",
            ba.is_deleted,
            bur.verified
        FROM business_user_department_relationship bur
        INNER JOIN department_account ba ON bur.department_id = ba.id
        WHERE bur.user_id = $1
          AND bur.business_id = $2
        "#,
        user_id,
        business_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[tracing::instrument(name = "Get department Account", skip(pool))]
pub async fn fetch_department_account_model_by_id(
    pool: &PgPool,
    department_id: Option<Uuid>,
) -> Result<Option<DepartmentAccountModel>, anyhow::Error> {
    use sqlx::QueryBuilder;

    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT
            id,
            display_name,
            is_active,
            is_deleted,
            TRUE AS verified
        FROM  department_account
        "#,
    );
    if let Some(department_id) = department_id {
        query_builder.push(" WHERE id = ");
        query_builder.push_bind(department_id);
    }

    // query_builder.push_bind(department_id);

    let query = query_builder.build_query_as::<DepartmentAccountModel>();

    let row = query.fetch_optional(pool).await?;
    Ok(row)
}

#[tracing::instrument(name = "Get department Account By User Id", skip(pool))]
pub async fn fetch_associated_department_account_model(
    user_id: Uuid,
    department_id: Uuid,
    business_id: Uuid,
    pool: &PgPool,
) -> Result<Option<UserDepartmentRelationAccountModel>, anyhow::Error> {
    let row = sqlx::query_as!(
        UserDepartmentRelationAccountModel,
        r#"
        SELECT
            ba.id,
            ba.display_name,
            ba.is_active as "is_active!:Status",
            ba.is_deleted,
            bur.verified
        FROM business_user_department_relationship AS bur
        INNER JOIN department_account ba ON bur.department_id = ba.id
        WHERE bur.user_id = $1
          AND bur.department_id = $2
          AND bur.business_id = $3
        "#,
        user_id,
        department_id,
        business_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
#[tracing::instrument(name = "Get department Account by department id", skip(pool))]
pub async fn get_department_account(
    pool: &PgPool,
    user_id: Uuid,
    business_id: Uuid,
    department_id: Uuid,
) -> Result<Option<DepartmentAccount>, anyhow::Error> {
    let department_account_model =
        fetch_associated_department_account_model(user_id, department_id, business_id, pool)
            .await?;
    match department_account_model {
        Some(model) => {
            let department_account = model.into_schema();
            Ok(Some(department_account))
        }
        None => Ok(None),
    }
}

#[tracing::instrument(name = "Get Basic department account by user id")]
pub async fn get_basic_department_accounts_by_user_id(
    user_id: Uuid,
    business_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<BasicDepartmentAccount>, anyhow::Error> {
    let department_account_models =
        fetch_department_account_models_by_user_account(pool, user_id, business_id).await?;
    let department_account_list = department_account_models
        .into_iter()
        .map(|e| e.into_basic_schema())
        .collect();

    Ok(department_account_list)
}

#[allow(dead_code)]
pub fn validate_department_account_active(department_obj: &DepartmentAccount) -> Option<String> {
    match (
        &department_obj.is_active,
        department_obj.is_deleted,
        department_obj.verified,
    ) {
        (Status::Inactive, _, _) => Some("department Account is inactive".to_string()),
        (_, true, _) => Some("department Account is deleted".to_string()),
        (_, _, false) => Some("department User relation is not verified".to_string()),
        _ => None,
    }
}

#[allow(dead_code)]
pub async fn validate_user_department_permission(
    pool: &PgPool,
    user_id: Uuid,
    business_account: Uuid,
    department_id: Uuid,
    action_list: &Vec<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let permission_list = sqlx::query_scalar!(
        r#"
        SELECT  p.name
        FROM business_user_department_relationship bur
        INNER JOIN role_permission rp ON bur.role_id = rp.role_id
        INNER JOIN permission p ON rp.permission_id = p.id
        WHERE bur.user_id = $1
          AND bur.department_id = $2
          AND bur.business_id = $3
          AND rp.is_deleted = FALSE
          AND p.is_deleted = FALSE
          AND p.name = ANY($4::text[])
        "#,
        user_id,
        department_id,
        business_account,
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

#[tracing::instrument(name = "Get Basic Business account by user id")]
pub async fn get_basic_department_accounts(
    pool: &PgPool,
) -> Result<Vec<BasicDepartmentAccount>, anyhow::Error> {
    let business_account_models = fetch_department_account_model_by_id(pool, None).await?;
    let business_account_list = business_account_models
        .into_iter()
        .map(|e| e.into_basic_schema())
        .collect();

    Ok(business_account_list)
}

#[tracing::instrument(name = "associate user to department", skip(pool))]
pub async fn associate_user_to_department(
    pool: &PgPool,
    user_id: Uuid,
    business_id: Uuid,
    department_id: Uuid,
    role_id: Uuid,
    created_by: Uuid,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO business_user_department_relationship
        (id, user_id, business_id, department_id, role_id, verified, created_on, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        Uuid::new_v4(),
        user_id,
        business_id,
        department_id,
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
            .context("A database failure occurred while associating user to department")
    })?;
    Ok(())
}

#[tracing::instrument(name = "delete project account", skip(pool))]
pub async fn hard_delete_department_account(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query("DELETE FROM department_account WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;
    Ok(())
}
