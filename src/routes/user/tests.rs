#[cfg(test)]
pub mod tests {
    use crate::constants::DUMMY_INTERNATIONAL_DIALING_CODE;
    use crate::email::EmailObject;
    use crate::routes::user::schemas::{
        AuthenticationScope, CreateUserAccount, EditUserAccount, RoleType, VectorType,
    };
    use crate::routes::user::utils::{
        get_minimal_user_list, get_stored_credentials, get_user, hard_delete_user_account,
        reactivate_user_account, register_user, reset_password, soft_delete_user_account,
        update_otp, update_user, verify_otp, verify_password,
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
        let password = "123";
        let mobile_no = "1234567893";
        let user_res = setup_user(
            &pool,
            "testuser4",
            "testuser4@example.com",
            mobile_no,
            password,
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
        let auth_opt = auth_res.unwrap();
        assert!(auth_opt.is_some());
        let auth_obj = auth_opt.unwrap();
        let password_res = verify_password(SecretString::from(password), &auth_obj).await;
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
        let send_otp = update_otp(&pool, otp, 30, otp_res_obj).await;
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
        let password = "123";
        let mobile_no = "1234567899";
        let user_res = setup_user(
            &pool,
            "testuser12",
            "testuser12@example.com",
            mobile_no,
            password,
        )
        .await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let _ = soft_delete_user_account(&pool, &user_id.to_string(), user_id).await;
        let user_obj_opt = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            &pool,
        )
        .await
        .unwrap();
        let user_obj = user_obj_opt.unwrap();
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
        let password = "123";
        let mobile_no = "1234567812";
        let user_res = setup_user(
            &pool,
            "testuser15",
            "testuser15@example.com",
            mobile_no,
            password,
        )
        .await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let _ = soft_delete_user_account(&pool, &user_id.to_string(), user_id).await;
        let user_obj_opt = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            &pool,
        )
        .await
        .unwrap();
        let user_obj = user_obj_opt.unwrap();
        assert!(user_obj.is_deleted == true);

        let reactivate_res = reactivate_user_account(&pool, user_obj.id, user_obj.id).await;
        assert!(reactivate_res.is_ok());
        let user_obj_opt = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            &pool,
        )
        .await
        .unwrap();
        let user_obj = user_obj_opt.unwrap();
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
        let password = "123";
        let mobile_no_1 = "1234567814";
        let mobile_no_2 = "1234567815";
        let (user_res_1, user_res_2) = join!(
            setup_user(
                &pool,
                "testuser23",
                "testuser23@example.com",
                mobile_no_1,
                password,
            ),
            setup_user(
                &pool,
                "testuser24",
                "testuser24@example.com",
                mobile_no_2,
                password,
            ),
        );
        assert!(user_res_1.is_ok());
        assert!(user_res_2.is_ok());
        let user_list = get_minimal_user_list(&pool, None, 1, 0, None).await;
        assert!(user_list.is_ok());
        assert!(user_list.unwrap().len() > 1);

        let user_list = get_minimal_user_list(&pool, Some("testuser24"), 1, 0, None).await;
        assert!(user_list.is_ok());
        assert!(user_list.unwrap().len() == 1);
        let user_id = user_res_1.unwrap();
        let user_list = get_minimal_user_list(&pool, None, 1, 0, Some(&vec![user_id])).await;
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

    #[tokio::test]
    async fn test_user_account_update() {
        let pool = get_test_pool().await;
        let password = "123";
        let mobile_no_1 = "1234567817";
        let mobile_no_2 = "1234567813";
        let email_1 = "testuser25@example.com";
        let username_1 = "testuser25";
        let username_2 = "testuser26";
        let email_2 = "testuser26@example.com";
        let display_name_2 = "mango";
        let complete_mobile_2 = &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_2);
        let user_res = setup_user(&pool, username_1, email_1, mobile_no_1, password).await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let user_obj_opt = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_1
            )],
            &pool,
        )
        .await
        .unwrap();
        assert!(user_obj_opt.is_some());
        let user_obj = user_obj_opt.unwrap();
        let edit_req = EditUserAccount {
            username: username_2.to_string(),
            mobile_no: mobile_no_2.to_string(),
            international_dialing_code: DUMMY_INTERNATIONAL_DIALING_CODE.to_string(),
            email: EmailObject::new(email_2.to_string()),
            display_name: display_name_2.to_string(),
        };

        let update_res = update_user(&pool, &edit_req, &user_obj).await;
        assert!(update_res.is_ok());
        let user_obj_opt = get_user(vec![&user_id.to_string()], &pool).await.unwrap();
        assert!(user_obj_opt.is_some());
        let user_obj = user_obj_opt.unwrap();
        assert!(user_obj.username == username_2);
        assert!(user_obj.display_name == display_name_2);
        assert!(user_obj.email.get() == email_2);
        assert!(&user_obj.mobile_no == complete_mobile_2);
        let mobile_vector = user_obj
            .vectors
            .iter()
            .find(|a| a.key == VectorType::MobileNo);

        let email_vector = user_obj.vectors.iter().find(|a| a.key == VectorType::Email);
        assert!(mobile_vector.is_some());
        assert!(email_vector.is_some());
        assert!(&mobile_vector.unwrap().value == complete_mobile_2);
        assert!(&email_vector.unwrap().value == email_2);

        let (auth_res_password, auth_res_email, auth_res_otp) = join!(
            get_stored_credentials(&username_2, &AuthenticationScope::Password, &pool),
            get_stored_credentials(&email_2, &AuthenticationScope::Email, &pool),
            get_stored_credentials(complete_mobile_2, &AuthenticationScope::Otp, &pool)
        );

        assert!(auth_res_password.is_ok());
        assert!(auth_res_email.is_ok());
        assert!(auth_res_otp.is_ok());

        assert!(auth_res_password.unwrap().unwrap().auth_identifier == username_2);
        assert!(auth_res_email.unwrap().unwrap().auth_identifier == email_2);
        assert!(&auth_res_otp.unwrap().unwrap().auth_identifier == complete_mobile_2);
        let delete_res = hard_delete_user_account(&pool, &complete_mobile_2).await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_reset_password() {
        let pool = get_test_pool().await;
        let password = "123";
        let mobile_no = "123456785";
        let new_password = "456";
        let user_res = setup_user(
            &pool,
            "testuser27",
            "testuser27@example.com",
            mobile_no,
            password,
        )
        .await;
        let user_id = user_res.unwrap();
        let user_account_res = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            &pool,
        )
        .await;
        assert!(user_account_res.is_ok());
        let user_opt = user_account_res.unwrap();
        let reset_res = reset_password(&pool, new_password.into(), &user_opt.unwrap()).await;
        assert!(reset_res.is_ok());
        let auth_res = get_stored_credentials(
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
            &AuthenticationScope::Password,
            &pool,
        )
        .await;

        assert!(auth_res.is_ok());
        let auth_opt = auth_res.unwrap();
        let password_res =
            verify_password(SecretString::from(new_password), &auth_opt.unwrap()).await;
        assert!(password_res.is_ok());
        let delete_res = hard_delete_user_account(&pool, &user_id.to_string()).await;
        assert!(delete_res.is_ok());
    }
}
