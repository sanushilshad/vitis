
use actix_web::web;
use anyhow::Context;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{
 encode, Algorithm as JWTAlgorithm, EncodingKey, Header
};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;
use tera::{Context as TeraContext, Tera};
use crate::configuration::Jwt;
use crate::email::EmailObject;
use crate::email_client::{GenericEmailService, SmtpEmailClient};
use crate::routes::setting::schemas::{SettingKey, Settings, SettingsExt};
// use crate::routes::business::utils::get_basic_business_account_by_user_id;
use crate::routes::user::errors::UserRegistrationError;
// use crate::routes::user::schemas::HasFullMobileNumber;
use crate::schemas::{  MaskingType, Status};
use crate::utils::{spawn_blocking_with_tracing, to_title_case};
use sqlx::{Transaction, Postgres, Executor};
use super::errors::AuthError;
use super::models::{AuthMechanismModel, MinimalUserAccountModel, UserAccountModel, RoleModel};
use super::schemas::{AccountRole, AuthData, AuthMechanism, AuthenticateRequest, AuthenticationScope, BulkAuthMechanismInsert, BulkAuthMechanismUpdate, CreateUserAccount, EditUserAccount, JWTClaims, MinimalUserAccount, EmailOTPContext, UserRoleType, UserAccount, UserVector, VectorType};
use anyhow::anyhow;

#[tracing::instrument(
    name = "Validate credentials",
    skip(expected_password, password_candidate)
)]
fn verify_password_hash(
    expected_password: SecretString,
    password_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(|e|AuthError::InvalidCredentials(e.to_string()))
}


#[tracing::instrument(name = "Get Auth Mechanism model", skip(username, pool))]
async fn get_auth_mechanism_model(username: &str,
    scope: &AuthenticationScope,
    pool: &PgPool,
) -> Result<Option<AuthMechanismModel>, anyhow::Error> {
    let row: Option<AuthMechanismModel> = sqlx::query_as!(AuthMechanismModel, 
        r#"SELECT a.id as id, user_id, auth_identifier, retry_count, secret, a.is_active as "is_active: Status", auth_scope as "auth_scope: AuthenticationScope", valid_upto from auth_mechanism
        as a inner join user_account as b on a.user_id = b.id where (b.username = $1 OR b.mobile_no = $1 OR  b.email = $1)  AND auth_scope = $2"#,
        username,
        scope as &AuthenticationScope,
        // auth_context as &AuthContextType
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}


#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
pub async fn get_stored_credentials(
    username: &str,
    scope: &AuthenticationScope,
    pool: &PgPool,
    // auth_context: &AuthContextType
) -> Result<Option<AuthMechanism>, anyhow::Error> {


    if let Some(row) = get_auth_mechanism_model(username, scope, pool).await? {
        Ok(Some(row.get_schema()))
    } else {
        Ok(None)
    }
}
#[tracing::instrument(name = "Verify Password")]
pub async fn verify_password(
    password: SecretString,
    auth_mechanism: &AuthMechanism,
) -> Result<(), AuthError> {
    let mut expected_password_hash = SecretString::from(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            
    );
    expected_password_hash = auth_mechanism.secret.clone().unwrap_or(expected_password_hash);
    spawn_blocking_with_tracing(move || verify_password_hash(expected_password_hash.to_owned(), password))
        .await
        .context("Failed to spawn blocking task.")?
}

#[tracing::instrument(name = "Reset OTP")]
pub async fn reset_otp(pool: &PgPool, auth_mechanism: &AuthMechanism) -> Result<(), anyhow::Error> {
    let _updated_tutor_result = sqlx::query!(
        r#"UPDATE auth_mechanism SET
        valid_upto = $1, secret = $2, retry_count=0
        WHERE id = $3"#,
        None::<DateTime<Utc>>,
        None::<String>,
        auth_mechanism.id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute update query: {:?}", e);
        anyhow::anyhow!("Database error")
    })?;
    Ok(())
}


#[tracing::instrument(name = "Increment auth retry count")]
pub async fn increment_auth_retry_count(pool: &PgPool, auth_mechanism_id: Uuid, retry_count: i32) -> Result<(), anyhow::Error> {
    let _updated_tutor_result = sqlx::query!(
        r#"UPDATE auth_mechanism SET
        retry_count = $1
        WHERE id = $2"#,
        retry_count,
        auth_mechanism_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute update query: {:?}", e);
        anyhow::anyhow!("Database error")
    })?;
    Ok(())
}

#[tracing::instrument(name = "Verify OTP")]
pub async fn verify_otp(
    pool: &PgPool,
    secret: &SecretString,
    auth_mechanism: &AuthMechanism,
) -> Result<(), AuthError> {
    let otp = auth_mechanism
        .secret
        .as_ref()
        .ok_or_else(|| AuthError::InvalidCredentials("Please Send the OTP".to_string()))?;

    let retry_count = auth_mechanism.retry_count.unwrap_or(0);
    if retry_count > 5 {
        return Err(AuthError::TooManyRequest(
            "Max limit reached".to_string()
        ));
    }
    if let Some(valid_upto) = &auth_mechanism.valid_upto {
        if valid_upto <= &Utc::now() {
            return Err(AuthError::InvalidCredentials(
                "OTP has expired".to_string(),
            ));
        }
    }
    
    if otp.expose_secret() != secret.expose_secret() {

        increment_auth_retry_count(pool, auth_mechanism.id, retry_count+1).await.map_err(|_|
            AuthError::UnexpectedCustomError("Something went wrong while updating failed authentication count".to_string()))?;
        return Err(AuthError::InvalidCredentials(
            "Invalid OTP".to_string(),
        ))?;
    }
    
    reset_otp(pool, auth_mechanism).await.map_err(|e| {
        tracing::error!("Failed to execute verify_otp database query: {:?}", e);
        AuthError::DatabaseError(
            "Something went wrong while resetting OTP".to_string(),
            e,
        )
    })?;
    Ok(())
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_user_credentials(
    credentials: &AuthenticateRequest,
    pool: &PgPool,
) -> Result<Option<uuid::Uuid>, AuthError> {
    let mut user_id = None;

    if let Some(auth_mechanism) =
        get_stored_credentials(&credentials.identifier, &credentials.scope, pool).await?
    {
        if auth_mechanism.is_active == Status::Inactive{
            return Err(AuthError::InvalidCredentials(format!(
                "{:?} is not enabled for {}",
                credentials.scope, credentials.identifier
            )));
        }

        user_id = Some(auth_mechanism.user_id);
        match credentials.scope {
            AuthenticationScope::Password => {
                verify_password(credentials.secret.to_owned(), &auth_mechanism).await?;
            }
            AuthenticationScope::Otp => {
                verify_otp(pool, &credentials.secret, &auth_mechanism).await?;
            }
            _ => {
            }
        }
    }
    Ok(user_id)

}



// test case not needed
#[tracing::instrument(name = "Get user Account", skip(pool))]
pub async fn fetch_user(
    value_list: Vec<&str>,
    pool: &PgPool,
) -> Result<Option<UserAccountModel>, anyhow::Error> {
    let val_list: Vec<String> = value_list.iter().map(|&s| s.to_string()).collect();

    let row: Option<UserAccountModel> = sqlx::query_as!(
        UserAccountModel,
        r#"SELECT 
            ua.id, username, is_test_user, mobile_no, email, is_active as "is_active!:Status", 
            vectors as "vectors:sqlx::types::Json<Vec<UserVector>>", display_name, 
            ua.is_deleted, r.role_name FROM user_account as ua
            INNER JOIN user_role ur ON ua.id = ur.user_id
            INNER JOIN role r ON ur.role_id = r.id
        WHERE ua.email = ANY($1) OR ua.mobile_no = ANY($1) OR ua.id::text = ANY($1)
        "#,
        &val_list
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}






#[tracing::instrument(name = "Get User by value list")]
pub async fn get_user(value_list: Vec<&str>, pool: &PgPool) -> Result<Option<UserAccount>, anyhow::Error> {
    let user = fetch_user(value_list, pool).await?.map(|a|a.into_schema());
    Ok(user)
}



// test case not needed
#[tracing::instrument(name = "create user account")]
pub fn create_vector_from_create_account(
    user_account: &CreateUserAccount,
) -> Result<Vec<UserVector>, anyhow::Error> {
    let vector_list = vec![
        UserVector {
            key: VectorType::Email,
            value: user_account.email.get().to_string(),
            masking: MaskingType::NA,
            verified: false,
        },
        UserVector {
            key: VectorType::MobileNo,
            value: user_account.get_full_mobile_no(),
            masking: MaskingType::NA,
            verified: false,
        },
    ];
    return Ok(vector_list);
}


// test case not needed
#[tracing::instrument(name = "create user account", skip(transaction))]
pub async fn save_user(
    transaction: &mut Transaction<'_, Postgres>,
    user_account: &CreateUserAccount,
) -> Result<Uuid, anyhow::Error> {
    let user_id = Uuid::new_v4();
    let vector_list = create_vector_from_create_account(user_account)?;
    let query = sqlx::query!(
        r#"
        INSERT INTO user_account (id, username, email, mobile_no, created_by, created_on, display_name, vectors, is_active, is_test_user)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        user_id,
        &user_account.username,
        &user_account.email.get(),
        &user_account.get_full_mobile_no(),
        user_id,
        Utc::now(),
        &user_account.display_name,
        sqlx::types::Json(vector_list) as sqlx::types::Json<Vec<UserVector>>,
        Status::Active as Status,
        &user_account.is_test_user,
        // &user_account.international_dialing_code,
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure occured while saving user account".to_string(),
            e.into(),
        )
    })?;
    Ok(user_id)
}


// test case not needed
#[tracing::instrument(name = "get_role_model_by_type", skip(pool))]
pub async fn get_role_model(pool: &PgPool, query: &str) -> Result<Option<RoleModel>, anyhow::Error> {
    let row: Option<RoleModel> = sqlx::query_as!(
        RoleModel,
        r#"SELECT id, role_name, role_status as "role_status!:Status", created_on, created_by, is_deleted from role where role_name = $1 OR id::TEXT=$1"#,
        query
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}


// test case not needed
#[tracing::instrument(name = "get_role", skip(pool))]
pub async fn get_role(pool: &PgPool,  query: &str) -> Result<Option<AccountRole>, anyhow::Error> {
    let role_model = get_role_model(pool, query).await?;
    match role_model {
        Some(role) => {
            Ok(Some(role.int_schema()))
        }
        None => Ok(None),
    }
}

// test case not needed
#[tracing::instrument(name = "save user account role", skip(transaction))]
pub async fn save_user_role(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<Uuid, anyhow::Error> {
    let user_role_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO user_role (id, user_id, role_id, created_on, created_by)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        user_role_id,
        user_id,
        role_id,
        Utc::now(),
        user_id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure occured while saving user account role".to_string(),
            e.into(),
        )
    })?;
    Ok(user_role_id)
}


// no test case added
#[tracing::instrument(name = "hard delete user account", skip(pool))]
pub async fn hard_delete_user_account(
    pool: &PgPool,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query("DELETE FROM user_account WHERE id::text = $1 OR mobile_no = $1 OR username = $1")
    .bind(user_id)
    .execute(pool)
    .await;
    Ok(())
}

#[tracing::instrument(name = "soft delete user account", skip(pool))]
pub async fn soft_delete_user_account(
    pool: &PgPool,
    user_id: &str,
    created_by: Uuid
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE user_account 
        SET is_deleted = true,
        deleted_on = $2,
        deleted_by = $3
        WHERE id::text = $1 OR mobile_no = $1 OR username = $1
        "#,
        user_id,
        Utc::now(),
        created_by
    )
    .execute(pool).await.map_err(|e| { 
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while soft deleting user from database")
        })?;
    Ok(())
}

#[tracing::instrument(name = "delete business account", skip(pool))]
pub async fn hard_delete_business_account(
    pool: &PgPool,
    business_id: Uuid,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query("DELETE FROM business_account WHERE id = $1")
    .bind(business_id)
    .execute(pool)
    .await;
    Ok(())
}

// #[tracing::instrument(name = "delete user account", skip(pool))]
// pub async fn hard_delete_user_account_by_(
//     pool: &PgPool,
//     user_id: &Uuid,
// ) -> Result<(), anyhow::Error> {
//     let _ = sqlx::query("DELETE FROM user_account WHERE id = $1")
//     .bind(user_id)
//     .execute(pool)
//     .await;
//     Ok(())
// }


// test case not needed
fn compute_password_hash(password: SecretString) -> Result<SecretString, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(SecretString::from(password_hash))
}


// test case not needed
#[tracing::instrument(name = "prepare auth mechanism data", skip(user_id, user_account))]
pub async fn prepare_auth_mechanism_data_for_user_account(
    user_id: Uuid,
    user_account: &CreateUserAccount,
) -> Result<BulkAuthMechanismInsert, anyhow::Error> {
    let current_utc = Utc::now();
    let password = user_account.password.clone();
    let password_hash = spawn_blocking_with_tracing(move || {
        compute_password_hash(password)
    })
    .await?
    .context("Failed to hash password")?;

    let id = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let user_id_list = vec![user_id, user_id, user_id];
    let auth_scope = vec![
        AuthenticationScope::Password,
        AuthenticationScope::Otp,
        AuthenticationScope::Email,
    ];
    let auth_identifier: Vec<String> = vec![
        user_account.username.to_owned(),
        user_account.get_full_mobile_no(),
        user_account.email.get().to_owned(),
    ];
    let secret = vec![password_hash.expose_secret().to_string()];
    let is_active = vec![Status::Active, Status::Active, Status::Active];
    let created_on = vec![current_utc, current_utc, current_utc];
    let created_by = vec![user_id, user_id, user_id];
    // let auth_context = vec![
    //     AuthContextType::UserAccount,
    //     AuthContextType::UserAccount,
    //     AuthContextType::UserAccount,
    // ];

    Ok(BulkAuthMechanismInsert {
        id,
        user_id_list,
        auth_scope,
        auth_identifier,
        secret,
        is_active,
        created_on,
        created_by,
        // auth_context,
    })
}

// test case not needed
#[tracing::instrument(name = "save auth mechanism", skip(transaction, auth_data))]
pub async fn save_auth_mechanism(
    transaction: &mut Transaction<'_, Postgres>,
    auth_data: BulkAuthMechanismInsert,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO auth_mechanism (id, user_id, auth_scope, auth_identifier, secret, is_active, created_on, created_by)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::user_auth_identifier_scope[], $4::text[], $5::text[], $6::status[], $7::TIMESTAMP[], $8::text[])
        "#,
        &auth_data.id[..] as &[Uuid],
        &auth_data.user_id_list[..] as &[Uuid],
        &auth_data.auth_scope[..] as &[AuthenticationScope],
        &auth_data.auth_identifier[..] as &[String],
        &auth_data.secret[..],
        // &auth_data.auth_context[..] as &[AuthContextType],
        &auth_data.is_active[..] as &[Status],
        &auth_data.created_on[..] as &[DateTime<Utc>],
        &auth_data.created_by[..] as &[Uuid]
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        UserRegistrationError::DatabaseError(
            "A database failure was encountered while saving auth mechanisms".to_string(),
            e.into(),
        )
    })?;

    Ok(())
}

// test case added
#[tracing::instrument(name = "register user", skip(pool))]
pub async fn register_user(
    pool: &PgPool,
    user_account: &CreateUserAccount
) -> Result<Uuid, UserRegistrationError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    if let Some(existing_user_obj) = fetch_user(
        vec![user_account.email.get(), &user_account.mobile_no],
        pool,
    )
    .await?
    {
        if user_account.get_full_mobile_no() == existing_user_obj.mobile_no {
            let message = format!(
                "User Already exists with the given mobile number: {}",
                user_account.mobile_no
            );
            tracing::error!(message);
            return Err(UserRegistrationError::DuplicateMobileNo(message));
        } else {
            let message = format!(
                "User Already exists with the given email: {}",
                user_account.email
            );
            tracing::error!(message);
            return Err(UserRegistrationError::DuplicateEmail(message));
        }
    }
    let uuid = save_user(&mut transaction, user_account).await.map_err(UserRegistrationError::UnexpectedError)?;
    let bulk_auth_data = prepare_auth_mechanism_data_for_user_account(uuid, user_account).await?;
    save_auth_mechanism(&mut transaction, bulk_auth_data).await?;
    if  let Some(role_obj) = get_role(pool, &UserRoleType::User.to_lowercase_string()).await?{
        if role_obj.is_deleted || role_obj.role_status == Status::Inactive {
            return Err(UserRegistrationError::InvalidRoleError("Role is deleted / Inactive".to_string()))
        }
        save_user_role(&mut transaction, uuid, role_obj.id).await?;
    }
    else{
        tracing::info!("Invalid Role for user account {}", uuid);
        Err(UserRegistrationError::InvalidRoleError(format!("Invalid Role for user account {}", uuid)))?
        
    }

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new user account.")?;

    Ok(uuid)
}




pub async fn _get_user_id_from_mobile(pool: &PgPool, username: &str) ->  Result<Option<Uuid>, anyhow::Error>{
    let result = sqlx::query_scalar!(
        "SELECT id FROM user_account WHERE mobile_no = $1",
        username
    )
    .fetch_optional(pool)
    .await?;
    Ok(result)
}



#[tracing::instrument(name = "Generate JWT token for user")]
pub fn generate_jwt_token_for_user(
    user_id: Uuid,
    expiry_time: i64,
    secret: &SecretString,
) -> Result<SecretString, anyhow::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expiry_time))
        .expect("valid timestamp")
        .timestamp() as usize;
    let claims = JWTClaims {
        sub: user_id,
        exp: expiration,
    };
    let header = Header::new(JWTAlgorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
    let token: String = encode(&header, &claims, &encoding_key).expect("Failed to generate token");
    Ok(SecretString::new(token.into()))
}

#[tracing::instrument(name = "Get Auth Data")]
pub  fn get_auth_data(
    user_account: UserAccount,
    jwt_obj: &Jwt,
) -> Result<AuthData, anyhow::Error> {
    let token = generate_jwt_token_for_user(user_account.id, jwt_obj.expiry, &jwt_obj.secret)?;

    Ok(AuthData {
        user: user_account,
        token
    })
}


#[tracing::instrument(name = "Update User Verification Status", skip(pool))]
pub async fn update_user_verification_status(
    pool: &PgPool,
    vector_type: VectorType,
    user_id: Uuid,
    verified: bool
) -> Result<(), anyhow::Error> {
        sqlx::query!(
            r#"
            UPDATE user_account
            SET vectors = (
                SELECT jsonb_agg(
                    CASE
                        WHEN elem->>'key' = $1
                        THEN jsonb_set(elem, '{verified}', to_jsonb($2::boolean))
                        ELSE elem
                    END
                )
                FROM jsonb_array_elements(vectors) AS elem
            )
            WHERE id = $3
            "#,
            vector_type.to_string(),
            verified,
            user_id,
        )
        .execute(pool)
        .await.map_err(|e|{
             tracing::error!("Failed to execute query: {:?}", e);
             anyhow!(e)
        })?;

    Ok(())
}


#[tracing::instrument(name = "Send OTP", skip(pool))]
pub async fn update_otp(pool: &PgPool, otp: &str, expiry_in_sec: i64, credential: AuthMechanism) -> Result<(), anyhow::Error> {
    let valid_upto = Utc::now() + Duration::seconds(expiry_in_sec);

    sqlx::query!(
        r#"UPDATE auth_mechanism SET
        valid_upto = $1, secret = $2,
        retry_count = 0
        WHERE id = $3"#,
        valid_upto,
        otp,
        credential.id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to set otp query: {:?}", e);
        anyhow::anyhow!("Database error")
    })?;

    Ok(())
}


#[tracing::instrument(name = "reactivate user account", skip(pool))]
pub async fn reactivate_user_account(
    pool: &PgPool,
    user_id: Uuid,
    updated_by: Uuid
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE user_account 
        SET is_deleted = false,
        updated_on = $2,
        updated_by = $3
        WHERE id = $1
        "#,
        user_id,
        Utc::now(),
        updated_by
    )
    .execute(pool).await.map_err(|e| { 
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while reactivating user")
        })?;
    Ok(())
}



#[tracing::instrument(name = "Fetch minimal user list", skip(pool))]
pub async fn fetch_minimal_user_list(
    pool: &PgPool,
    query: Option<&str>,
    offset: i32,
    limit: i32,
    id_list: Option<&Vec<Uuid>>
) -> Result<Vec<MinimalUserAccountModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT display_name, mobile_no, id
        FROM user_account
        WHERE is_deleted=false
        "#,
    );

    if let Some(query) = query {
        let pattern = format!("%{}%", query);
        query_builder.push(" AND (");
        query_builder.push("LOWER(email) ILIKE ");
        query_builder.push_bind(pattern.clone());
        query_builder.push(" OR LOWER(display_name) ILIKE ");
        query_builder.push_bind(pattern.clone());
        query_builder.push(" OR LOWER(mobile_no) ILIKE ");
        query_builder.push_bind(pattern.clone());
        query_builder.push(" OR LOWER(username) ILIKE ");
        query_builder.push_bind(pattern);
        query_builder.push(")");
    }
    if let Some(ids) = id_list {
        if !ids.is_empty() {
            query_builder.push(" AND id IN (");
            let mut separated = query_builder.separated(", ");
            for id in ids {
                separated.push_bind(id);
            }
            query_builder.push(")");
        }
    }
    let query = query_builder.build_query_as::<MinimalUserAccountModel>();
    let rows = query.fetch_all(pool).await.map_err(|e| { 
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while fetching users")
        })?;
    Ok(rows)
}




pub async fn get_minimal_user_list(pool: &PgPool, query: Option<&str>, limit: i32, offset: i32, id_list: Option<&Vec<Uuid>>) -> Result<Vec<MinimalUserAccount>, anyhow::Error>{
    let data_models = fetch_minimal_user_list(pool, query, limit, offset, id_list).await?;
    let data = data_models.into_iter().map(|a|a.into_schema()).collect();
    Ok(data)
}



#[tracing::instrument(name = "prepare auth mechanism update data", skip(user_account))]
pub fn prepare_auth_mechanism_update(
    data: &EditUserAccount,
    user_account: &UserAccount
) -> BulkAuthMechanismUpdate {
    let current_utc = Utc::now();
    let id = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let user_id_list = vec![user_account.id, user_account.id, user_account.id];
    let auth_scope = vec![
        AuthenticationScope::Password,
        AuthenticationScope::Otp,
        AuthenticationScope::Email,
    ];
    let auth_identifier: Vec<String> = vec![
        data.username.to_owned(),
        data.mobile_no.to_owned(),
        data.email.get().to_owned(),
    ];
    let updated_on = vec![current_utc, current_utc, current_utc];
    let updated_by = vec![user_account.id, user_account.id, user_account.id];
    BulkAuthMechanismUpdate {
        id,
        user_id: user_id_list,
        auth_scope,
        auth_identifier,
        updated_on,
        updated_by,
    }
}

#[tracing::instrument(name = "save auth mechanism", skip(transaction))]
pub async fn update_auth_mechanism(
    transaction: &mut Transaction<'_, Postgres>,
    auth_data: &BulkAuthMechanismUpdate,
    user_account: &UserAccount
    // auth_data: BulkAuthMechanismUpdate,
    
) -> Result<(), anyhow::Error> {
    let query =sqlx::query!(
        r#"
        UPDATE auth_mechanism
        SET
            updated_on = t.updated_on,
            updated_by = t.updated_by,
            auth_identifier = t.auth_identifier
        FROM UNNEST(
            $1::timestamptz[],
            $2::uuid[],
            $3::text[],
            $4::uuid[],
            $5::user_auth_identifier_scope[]
        ) AS t(
            updated_on,
            updated_by,
            auth_identifier,
            user_id,
            auth_scope
        )
        WHERE auth_mechanism.user_id = t.user_id
        AND auth_mechanism.auth_scope = t.auth_scope;
        "#,
        &auth_data.updated_on,
        &auth_data.updated_by,
        &auth_data.auth_identifier,
        &auth_data.user_id,
        &auth_data.auth_scope as &[AuthenticationScope],
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while updating auth_mechanism to database request")
    })?;

    Ok(())
}


fn generate_updated_user_vectors(
    existing_vectors: &[UserVector],
    mobile: &str,
    email: &EmailObject,
) -> Vec<UserVector> {
    let mut updated_vectors = vec![];

    let mut has_mobile = false;
    let mut has_email = false;

    for vector in existing_vectors {
        match vector.key {
            VectorType::MobileNo => {
                has_mobile = true;
                if vector.value != mobile {
                    updated_vectors.push(UserVector {
                        key: VectorType::MobileNo,
                        value: mobile.to_owned(),
                        masking: vector.masking.clone(),
                        verified: false,
                    });
                } else {
                    updated_vectors.push(vector.clone());
                }
            }
            VectorType::Email => {
                has_email = true;
                if vector.value != email.get() {
                    updated_vectors.push(UserVector {
                        key: VectorType::Email,
                        value: email.get().to_string(),
                        masking: vector.masking.clone(),
                        verified: false,
                    });
                } else {
                    updated_vectors.push(vector.clone());
                }
            }
        }
    }

    if !has_mobile {
        updated_vectors.push(UserVector {
            key: VectorType::MobileNo,
            value: mobile.to_owned(),
            masking: MaskingType::NA, // adjust if needed
            verified: false,
        });
    }

    if !has_email {
        updated_vectors.push(UserVector {
            key: VectorType::Email,
            value: email.get().to_owned(),
            masking: MaskingType::NA, // adjust if needed
            verified: false,
        });
    }

    updated_vectors
}

#[tracing::instrument(name = "update user account", skip(tx))]
pub async fn update_user_account(tx: &mut Transaction<'_, Postgres>, data: &EditUserAccount, user_account: &UserAccount) -> Result<(), anyhow::Error>{
    let vector_list: Vec<UserVector> = generate_updated_user_vectors(&user_account.vectors, &data.mobile_no, &data.email);
    let query= sqlx::query!(
        r#"UPDATE user_account SET
            username = $1,
            mobile_no = $2,
            email = $3,
            display_name = $4,
            vectors = $5,
            updated_on = $6,
            updated_by = $7
        WHERE id = $8"#,
        data.username,
        // data.international_dialing_code,
        data.mobile_no,
        data.email.get(),
        data.display_name,
        sqlx::types::Json(vector_list) as sqlx::types::Json<Vec<UserVector>>,
        Utc::now(),
        user_account.id,
        user_account.id
    );

    tx.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute user update query: {:?}", e);
        anyhow::anyhow!("Something went wrong while updating user details.")
    })?;

    Ok(())
}

#[tracing::instrument(name = "update user account", skip(pool))]
pub async fn update_user(pool: &PgPool, data: &EditUserAccount, user_account: &UserAccount) -> Result<(), anyhow::Error>{
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    update_user_account(&mut transaction, data, user_account).await?;
    let auth_data = prepare_auth_mechanism_update(data, user_account);
    update_auth_mechanism(&mut transaction, &auth_data, user_account).await?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store an order")?;
   Ok(()) 
}





pub async fn reset_password(pool: &PgPool, password: SecretString, user_account: &UserAccount)  -> Result<(), anyhow::Error>{
    let password = password.clone();
    let password_hash = spawn_blocking_with_tracing(move || {
        compute_password_hash(password)
    })
    .await?
    .context("Failed to hash password")?;
    let _ = sqlx::query!(
        r#"UPDATE auth_mechanism SET
        updated_on = $1, updated_by = $2, secret=$3
        WHERE user_id = $2 AND auth_scope='password'"#,
        &Utc::now(),
        user_account.id,
        password_hash.expose_secret()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute update password query: {:?}", e);
        anyhow::anyhow!("Database error")
    })?;
    Ok(())
}

pub async fn verify_vector_if_needed(
    pool: &PgPool,
    user_account: &UserAccount,
    scope: &AuthenticationScope,
) -> Result<(), anyhow::Error> {
    if let Some(vector_type)= scope.get_vector(){
        if !user_account.is_vector_verified(&vector_type) {
            update_user_verification_status(pool, vector_type, user_account.id, true).await?;
        }
    }
    Ok(())
}

pub async fn send_email_otp(email_client:  web::Data<SmtpEmailClient>, user: &UserAccount, otp: &str, configs: &Vec<Settings>) -> Result<(), anyhow::Error>{

    let html_template: String = configs
        .get_setting(&SettingKey::EmailOTPTemplate.to_string())
        .ok_or_else(|| {
            anyhow!(format!(
                "Please set the {}",
                SettingKey::EmailOTPTemplate
            ))
        })?;
    let receiver = to_title_case(&user.display_name);
    let sender = to_title_case("Vitis");
    let context_data = EmailOTPContext {
        name: &sender,
        otp,
        receiver: &receiver,
    };
    let context = TeraContext::from_serialize(&context_data).map_err(|e: tera::Error| {
        tracing::error!("{}", e);
        anyhow!(
            "Something went wrong while rendering the email html data".to_string(),
        )
    })?;
    let rendered_string = Tera::one_off(&html_template, &context, true).map_err(|e| {
        tracing::error!("Error while rendering html {} error: {}", html_template, e);
        anyhow!(
            "Something went wrong while rendering the email html data".to_string(),
        )
    })?;
    let _ = email_client
        .send_html_email(
            &user.email,            
            &None,                   
            "Email OTP",
            rendered_string,       
            None,                     
            None,
        )
        .await;
    Ok(())
}