use super::models::UserprojectRelationAccountModel;
use super::schemas::{BasicprojectAccount, ProjectAccount};
use super::{
    errors::ProjectAccountError, models::ProjectAccountModel, schemas::CreateprojectAccount,
};
use crate::routes::user::schemas::{
    AuthenticationScope, BulkAuthMechanismInsert, UserAccount, UserType, UserVector, VectorType,
};
use crate::routes::user::utils::get_role;
use crate::schemas::{MaskingType, Status};
use anyhow::Context;
use chrono::Utc;
use sqlx::{Execute, Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(name = "create user account")]
pub fn create_vector_from_project_account(
    project_account: &CreateprojectAccount,
) -> Result<Vec<UserVector>, ProjectAccountError> {
    let vector_list = vec![
        UserVector {
            key: VectorType::Email,
            value: project_account.email.get().to_string(),
            masking: MaskingType::NA,
            verified: false,
        },
        UserVector {
            key: VectorType::MobileNo,
            value: project_account.get_full_mobile_no(),
            masking: MaskingType::NA,
            verified: false,
        },
    ];
    return Ok(vector_list);
}

#[tracing::instrument(name = "create user project relation", skip(transaction))]
pub async fn save_project_account(
    transaction: &mut Transaction<'_, Postgres>,
    user_account: &UserAccount,
    create_project_obj: &CreateprojectAccount,
) -> Result<uuid::Uuid, ProjectAccountError> {
    let project_account_id = Uuid::new_v4();
    let project_name = create_project_obj.name.clone(); // Assuming this maps to `name`
    let vector_list = create_vector_from_project_account(create_project_obj)?; // JSONB column

    let query = sqlx::query!(
        r#"
        INSERT INTO project_account (id, name, vectors, created_by, created_on)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        project_account_id,
        project_name,
        sqlx::types::Json(vector_list) as sqlx::types::Json<Vec<UserVector>>,
        user_account.id,
        Utc::now()
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        ProjectAccountError::DatabaseError(
            "A database failure occurred while saving project account".to_string(),
            e.into(),
        )
    })?;

    Ok(project_account_id)
}
#[tracing::instrument(name = "create user project relation", skip(transaction))]
pub async fn save_user_project_relation(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    project_id: Uuid,
    role_id: Uuid,
) -> Result<uuid::Uuid, ProjectAccountError> {
    let user_role_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO project_user_relationship (id, user_id, project_id, role_id, created_by, created_on)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_role_id,
        user_id,
        project_id,
        role_id,
        user_id,
        Utc::now(),
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        ProjectAccountError::DatabaseError(
            "A database failure occured while saving user project relation".to_string(),
            e.into(),
        )
    })?;
    Ok(user_role_id)
}

#[tracing::instrument(name = "Save Auth Mechanism for project account")]
pub fn prepare_auth_mechanism_data_for_project_account(
    user_id: Uuid,
    user_account: &CreateprojectAccount,
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
    //     AuthContextType::projectAccount,
    //     AuthContextType::projectAccount,
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
#[tracing::instrument(name = "create project account", skip(pool))]
pub async fn create_project_account(
    pool: &PgPool,
    user_account: &UserAccount,
    create_project_obj: &CreateprojectAccount,
) -> Result<uuid::Uuid, ProjectAccountError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let project_account_id =
        save_project_account(&mut transaction, user_account, create_project_obj).await?;
    if let Some(role_obj) = get_role(pool, &UserType::Admin).await? {
        if role_obj.is_deleted || role_obj.role_status == Status::Inactive {
            return Err(ProjectAccountError::InvalidRoleError(
                "Role is deleted / Inactive".to_string(),
            ));
        }
        save_user_project_relation(
            &mut transaction,
            user_account.id,
            project_account_id,
            role_obj.id,
        )
        .await?;
    } else {
        tracing::info!("Invalid role for project account");
    }

    // let bulk_auth_data =
    //     prepare_auth_mechanism_data_for_project_account(user_account.id, create_project_obj);
    // save_auth_mechanism(&mut transaction, bulk_auth_data).await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new project account.")?;
    Ok(project_account_id)
}

#[tracing::instrument(name = "Get project Account By User id", skip(pool))]
pub async fn fetch_project_account_model_by_user_account(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ProjectAccountModel>, anyhow::Error> {
    use sqlx::QueryBuilder;

    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT 
            ba.id, ba.name, 
            vectors,
            ba.is_active
        FROM project_user_relationship as bur
            INNER JOIN project_account ba ON bur.project_id = ba.id
        WHERE bur.user_id = "#,
    );

    query_builder.push_bind(user_id);

    // Conditionally add customer_type filter

    let query = query_builder.build_query_as::<ProjectAccountModel>();

    let rows = query.fetch_all(pool).await?;
    Ok(rows)
}

#[tracing::instrument(name = "Get project Account By User Id", skip(pool))]
pub async fn fetch_project_account_model(
    user_id: Uuid,
    project_account_id: Uuid,
    pool: &PgPool,
) -> Result<Option<UserprojectRelationAccountModel>, anyhow::Error> {
    use sqlx::QueryBuilder;

    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT 
            ba.id, ba.name,
            vectors,
            ba.is_active,
            bur.verified, ba.is_deleted
        FROM project_user_relationship as bur
            INNER JOIN project_account ba ON bur.project_id = ba.id
        WHERE bur.user_id = "#,
    );

    query_builder.push_bind(user_id);
    query_builder.push(" AND bur.project_id = ");
    query_builder.push_bind(project_account_id);

    let query = query_builder.build_query_as::<UserprojectRelationAccountModel>();
    let query_string = query.sql();
    println!(
        "Generated SQL query for user_id {} and project_id {}: {}",
        user_id, project_account_id, query_string
    );
    let row = query.fetch_optional(pool).await?;
    Ok(row)
}

#[tracing::instrument(name = "Get project Account by project id", skip(pool))]
pub async fn get_project_account(
    pool: &PgPool,
    user_id: Uuid,
    project_account_id: Uuid,
) -> Result<Option<ProjectAccount>, anyhow::Error> {
    let project_account_model =
        fetch_project_account_model(user_id, project_account_id, pool).await?;
    match project_account_model {
        Some(model) => {
            let project_account = model.into_schema();
            Ok(Some(project_account))
        }
        None => Ok(None),
    }
}

#[tracing::instrument(name = "Get Basic project account by user id")]
pub async fn get_basic_project_account_by_user_id(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<BasicprojectAccount>, anyhow::Error> {
    let project_account_models = fetch_project_account_model_by_user_account(pool, user_id).await?;
    let project_account_list = project_account_models
        .into_iter()
        .map(|e| e.into_schema())
        .collect();

    Ok(project_account_list)
}

// test case added
pub fn validate_project_account_active(project_obj: &ProjectAccount) -> Option<String> {
    match (
        &project_obj.is_active,
        project_obj.is_deleted,
        project_obj.verified,
    ) {
        (Status::Inactive, _, _) => Some("project Account is inactive".to_string()),
        (_, true, _) => Some("project Account is deleted".to_string()),
        (_, _, false) => Some("project User relation is not verified".to_string()),
        _ => None,
    }
}
pub async fn validate_user_project_permission(
    pool: &PgPool,
    user_id: Uuid,
    project_id: Uuid,
    action_list: &Vec<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let permission_list = sqlx::query_scalar!(
        r#"
        SELECT  p.permission_name
        FROM project_user_relationship bur
        INNER JOIN role_permission rp ON bur.role_id = rp.role_id
        INNER JOIN permission p ON rp.permission_id = p.id
        WHERE bur.user_id = $1
          AND bur.project_id = $2
          AND rp.is_deleted = FALSE
          AND p.is_deleted = FALSE
          AND p.permission_name = ANY($3::text[])
        "#,
        user_id,
        project_id,
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
