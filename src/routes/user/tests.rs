#[cfg(test)]
pub mod tests {
    use crate::constants::INTERNATIONAL_DIALING_CODE;
    use crate::email::EmailObject;
    use crate::routes::user::schemas::{AuthenticationScope, CreateUserAccount, UserType};
    use crate::routes::user::utils::{
        get_stored_credentials, get_user, hard_delete_user_account, register_user, send_otp,
        verify_otp, verify_password,
    };

    use crate::tests::tests::get_test_pool;
    use secrecy::SecretString;
    use sqlx::PgPool;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_user_create_and_fetch() {
        let pool = get_test_pool().await;
        let mobile_no = "1234567890";
        let user_res = setup_user(
            &pool,
            "testuser1",
            "testuser@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        assert!(user_res.is_ok());
        assert!(
            get_user(
                vec![&format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no)],
                &pool
            )
            .await
            .is_ok()
        );
        let user_id = &user_res.unwrap();
        let user_res = setup_user(
            &pool,
            "testuser1",
            "testuser@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        assert!(user_res.is_err());
        let delete_res = hard_delete_user_account(&pool, &user_id.to_string()).await;
        assert!(delete_res.is_ok());
    }

    pub async fn setup_user(
        pool: &PgPool,
        username: &str,
        email: &str,
        mobile_no: &str,
        password: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let user_account = CreateUserAccount {
            username: username.to_string(),
            email: EmailObject::new(email.to_string()),
            mobile_no: mobile_no.to_string(),
            display_name: "Test User".to_string(),
            is_test_user: false,
            international_dialing_code: INTERNATIONAL_DIALING_CODE.to_string(),
            user_type: UserType::User,
            password: SecretString::from(password),
        };
        let user_result = register_user(pool, &user_account).await?;
        Ok(user_result)
    }

    #[tokio::test]
    async fn test_password_authentication() {
        let pool = get_test_pool().await;
        let passsword = "123";
        let mobile_no = "1234567893";
        let user_res = setup_user(
            &pool,
            "testuser4",
            "testuser4@example.com",
            mobile_no,
            passsword,
        )
        .await;
        assert!(user_res.is_ok());
        let auth_res = get_stored_credentials(
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
            &AuthenticationScope::Password,
            &pool,
        )
        .await;
        assert!(auth_res.is_ok());
        let auth_opt: Option<crate::routes::user::schemas::AuthMechanism> = auth_res.unwrap();
        assert!(auth_opt.is_some());
        let auth_obj = auth_opt.unwrap();
        let password_res = verify_password(SecretString::from(passsword), &auth_obj).await;
        assert!(password_res.is_ok());
        let password_res = verify_password(SecretString::from("abc"), &auth_obj).await;
        assert!(password_res.is_err());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_otp_authentication() {
        let pool = get_test_pool().await;
        let otp = "123";
        let mobile_no = "1234567897";
        let user_res = setup_user(
            &pool,
            "testuser11",
            "testuser11@example.com",
            mobile_no,
            otp,
        )
        .await;
        assert!(user_res.is_ok());

        let otp_res = get_stored_credentials(
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
            &AuthenticationScope::Otp,
            &pool,
        )
        .await;
        assert!(otp_res.is_ok());
        let otp_res_opt = otp_res.unwrap();
        assert!(otp_res_opt.is_some());
        let otp_res_obj = otp_res_opt.unwrap();
        let send_otp = send_otp(&pool, otp, 30, otp_res_obj).await;
        assert!(send_otp.is_ok());
        let otp_res = get_stored_credentials(
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
            &AuthenticationScope::Otp,
            &pool,
        )
        .await;
        let otp_obj = otp_res.unwrap().unwrap();
        let verify_otp = verify_otp(&pool, &SecretString::from(otp), &otp_obj).await;
        assert!(verify_otp.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }
}
