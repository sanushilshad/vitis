#[cfg(test)]
pub mod tests {
    use crate::constants::DUMMY_INTERNATIONAL_DIALING_CODE;
    use crate::email::EmailObject;
    use crate::routes::user::schemas::{AuthenticationScope, CreateUserAccount, RoleType};
    use crate::routes::user::utils::{
        get_minimal_user_list, get_stored_credentials, get_user, hard_delete_user_account,
        reactivate_user_account, register_user, send_otp, soft_delete_user_account, verify_otp,
        verify_password,
    };

    use crate::tests::tests::get_test_pool;
    use secrecy::SecretString;
    use sqlx::PgPool;
    use tokio::join;
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
                vec![&format!(
                    "{}{}",
                    DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
                )],
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
            international_dialing_code: DUMMY_INTERNATIONAL_DIALING_CODE.to_string(),
            user_type: RoleType::Developer,
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
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
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
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
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
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
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
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
            &AuthenticationScope::Otp,
            &pool,
        )
        .await;
        let otp_obj = otp_res.unwrap().unwrap();
        let verify_otp = verify_otp(&pool, &SecretString::from(otp), &otp_obj).await;
        assert!(verify_otp.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_user_user_soft_delete() {
        let pool = get_test_pool().await;
        let passsword = "123";
        let mobile_no = "1234567899";
        let user_res = setup_user(
            &pool,
            "testuser12",
            "testuser12@example.com",
            mobile_no,
            passsword,
        )
        .await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let _ = soft_delete_user_account(&pool, &user_id.to_string(), user_id).await;
        let user_obj = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            &pool,
        )
        .await
        .unwrap();
        assert!(user_obj.is_deleted == true);

        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_reactivate_soft_deleted_user() {
        let pool = get_test_pool().await;
        let passsword = "123";
        let mobile_no = "1234567812";
        let user_res = setup_user(
            &pool,
            "testuser15",
            "testuser15@example.com",
            mobile_no,
            passsword,
        )
        .await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let _ = soft_delete_user_account(&pool, &user_id.to_string(), user_id).await;
        let user_obj = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            &pool,
        )
        .await
        .unwrap();
        assert!(user_obj.is_deleted == true);

        let reactivate_res = reactivate_user_account(&pool, user_obj.id, user_obj.id).await;
        assert!(reactivate_res.is_ok());
        let user_obj = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            &pool,
        )
        .await
        .unwrap();
        assert!(user_obj.is_deleted == false);

        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_user_list() {
        let pool = get_test_pool().await;
        let passsword = "123";
        let mobile_no_1 = "1234567814";
        let mobile_no_2 = "1234567815";
        let (user_res_1, user_res_2) = join!(
            setup_user(
                &pool,
                "testuser23",
                "testuser23@example.com",
                mobile_no_1,
                passsword,
            ),
            setup_user(
                &pool,
                "testuser24",
                "testuser24@example.com",
                mobile_no_2,
                passsword,
            ),
        );
        assert!(user_res_1.is_ok());
        assert!(user_res_2.is_ok());
        let user_list = get_minimal_user_list(&pool, None, 1, 0).await;
        assert!(user_list.is_ok());
        assert!(user_list.unwrap().len() == 2);

        let user_list = get_minimal_user_list(&pool, Some("testuser24"), 1, 0).await;
        assert!(user_list.is_ok());
        assert!(user_list.unwrap().len() == 1);

        let full_mobile_no_1 = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_1);
        let full_mobile_no_2 = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_2);
        let (delete_res_1, delete_res_2) = join!(
            hard_delete_user_account(&pool, &full_mobile_no_1),
            hard_delete_user_account(&pool, &full_mobile_no_2),
        );

        assert!(delete_res_1.is_ok());
        assert!(delete_res_2.is_ok());
    }
}
