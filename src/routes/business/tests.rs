#[cfg(test)]
pub mod tests {
    use crate::constants::DUMMY_INTERNATIONAL_DIALING_CODE;
    use crate::email::EmailObject;
    use crate::routes::business::schemas::{BusinessAccount, CreateBusinessAccount};
    use crate::routes::business::utils::{
        associate_user_to_business, create_business_account, delete_invite_by_id,
        delete_user_business_relationship, fetch_associated_business_account_model,
        fetch_business_account_model_by_id, fetch_business_invite, get_basic_business_accounts,
        get_basic_business_accounts_by_user_id, get_business_account, mark_invite_as_verified,
        save_business_invite_request, save_user_business_relation, soft_delete_business_account,
        validate_business_account_active, validate_user_business_permission,
    };

    use crate::routes::user::schemas::UserRoleType;
    use crate::routes::user::tests::tests::setup_user;
    use crate::routes::user::utils::{
        fetch_user_account_by_business_account, get_role, get_user, hard_delete_business_account,
        hard_delete_user_account,
    };

    use crate::schemas::{PermissionType, Status};

    use crate::tests::tests::get_test_pool;
    use anyhow::Context;
    use chrono::Utc;
    use sqlx::PgPool;
    use tokio::join;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_validate_active_business_account() {
        let mut business_account = BusinessAccount {
            id: Uuid::new_v4(),
            display_name: "SANU PRIVATE LIMITED".to_string(),
            vectors: vec![],
            is_active: Status::Active,
            is_deleted: false,
            verified: true,
        };

        // Test case 5: business account is inactive
        business_account.is_active = Status::Inactive;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(
            validate_response,
            Some("business Account is inactive".to_string())
        );

        // Test case 6: business account is deleted
        business_account.is_active = Status::Active;
        business_account.is_deleted = true;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(
            validate_response,
            Some("business Account is deleted".to_string())
        );

        // Test case 7: business user relation is not verified
        business_account.is_deleted = false;
        business_account.verified = false;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(
            validate_response,
            Some("business User relation is not verified".to_string())
        );

        // Test case 8: All conditions are met
        business_account.verified = true;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(validate_response, None);
    }

    pub async fn setup_business(
        pool: &PgPool,
        mobile_no: &str,
        email: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let user_res = get_user(
            vec![&format!(
                "{}{}",
                DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no
            )],
            pool,
        )
        .await;
        let user_opt = user_res.unwrap();
        let create_business_obj = CreateBusinessAccount {
            name: "Test Company".to_string(),
            is_test_account: false,

            mobile_no: mobile_no.to_string(),
            email: EmailObject::new(email.to_string()),

            international_dialing_code: DUMMY_INTERNATIONAL_DIALING_CODE.to_string(),
        };
        let business_res_obj =
            create_business_account(pool, &user_opt.unwrap(), &create_business_obj).await?;
        Ok(business_res_obj)
    }

    #[tokio::test]
    async fn test_business_and_fetch() {
        let pool = get_test_pool().await;

        let mobile_no = "1234567892";
        let user_res = setup_user(
            &pool,
            "testuser3",
            "testuser3@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, &mobile_no, "business@example.com").await;
        assert!(business_res.is_ok());
        let business_id = business_res.unwrap();
        let fetch_basic_business_obj_res =
            get_basic_business_accounts_by_user_id(user_id, &pool).await;
        assert!(fetch_basic_business_obj_res.is_ok());

        let fetch_basic_business_obj_res =
            get_basic_business_accounts_by_user_id(user_id, &pool).await;
        assert!(fetch_basic_business_obj_res.is_ok());

        let fetch_basic_business_objs = get_basic_business_accounts(&pool).await;
        eprint!("Basic business Accounts: {:?}", fetch_basic_business_objs);
        assert!(fetch_basic_business_objs.is_ok());

        let fetch_business_obj_res = get_business_account(&pool, user_id, business_id).await;
        assert!(fetch_business_obj_res.is_ok());

        let fetch_business_obj_by_id =
            fetch_business_account_model_by_id(&pool, Some(business_id)).await;
        eprint!("Business Account List: {:?}", fetch_business_obj_by_id);

        let delete_bus_res = hard_delete_business_account(&pool, business_id).await;
        assert!(delete_bus_res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_business_permission_validation() {
        let pool = get_test_pool().await;

        let mobile_no = "12345678929";
        let user_res = setup_user(
            &pool,
            "testuser5",
            "testuser5@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let permission_res = validate_user_business_permission(
            &pool,
            user_id,
            business_id,
            &vec![PermissionType::AssociateUserBusiness.to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() > 0);
        let permission_res = validate_user_business_permission(
            &pool,
            user_id,
            business_id,
            &vec!["create:setting1".to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() == 0);
        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_business_account(&pool, business_id).await;

        let permission_res = validate_user_business_permission(
            &pool,
            user_id,
            business_id,
            &vec![PermissionType::CreateBusinessSetting.to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() == 0);
    }

    #[tokio::test]
    async fn test_list_associated_business_accounts() {
        let pool = get_test_pool().await;

        let mobile_no = "12345678934";
        let user_res = setup_user(
            &pool,
            "testuser10",
            "testuser10@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let business_account_list_res =
            get_basic_business_accounts_by_user_id(user_id, &pool).await;
        assert!(business_account_list_res.is_ok());
        let business_account_list = business_account_list_res.unwrap();
        let first_business_account = business_account_list.first().unwrap();
        assert!(first_business_account.id == business_id);
        let delete_bus_res = hard_delete_business_account(&pool, business_id).await;
        assert!(delete_bus_res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_business_user_association() {
        let pool = get_test_pool().await;

        let mobile_no_1 = "12345678939";
        let mobile_no_2 = "12345678949";
        let mobile_with_code_1 = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_1);
        let mobile_with_code_2 = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_2);
        // Create two users concurrently
        let (user_res_1, user_res_2) = join!(
            setup_user(
                &pool,
                "testuser13",
                "testuser13@example.com",
                mobile_no_1,
                "testuser@123",
            ),
            setup_user(
                &pool,
                "testuser14",
                "testuser14@example.com",
                mobile_no_2,
                "testuser@123",
            )
        );

        let user_id_1 = user_res_1.unwrap();
        let user_id_2 = user_res_2.unwrap();

        // business and role fetching can happen concurrently
        let role = UserRoleType::User.to_lowercase_string();
        let (business_res, role_obj_opt) = join!(
            setup_business(&pool, mobile_no_1, "business@example.com"),
            get_role(&pool, &role),
        );

        let business_id = business_res.unwrap();
        let role_obj_opt = role_obj_opt.unwrap();
        assert!(role_obj_opt.is_some());
        let role_obj = role_obj_opt.unwrap();
        // Associate user to business
        let user_business_association =
            associate_user_to_business(&pool, user_id_2, business_id, role_obj.id, user_id_1).await;
        assert!(user_business_association.is_ok());

        // Fetch and assert association
        let fetched_association = get_business_account(&pool, user_id_2, business_id)
            .await
            .unwrap();
        assert!(fetched_association.is_some());

        // Perform deletions concurrently

        let (delete_bus_res, delete_res_1, delete_res_2) = join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &mobile_with_code_1,),
            hard_delete_user_account(&pool, &mobile_with_code_2),
        );

        assert!(delete_bus_res.is_ok());
        assert!(delete_res_1.is_ok());
        assert!(delete_res_2.is_ok());
    }

    #[tokio::test]
    async fn test_list_business_associated_user_accounts() {
        let pool = get_test_pool().await;

        let mobile_no = "12345618934";
        let user_res = setup_user(
            &pool,
            "testuser34",
            "testuser34@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let user_account_list_res =
            fetch_user_account_by_business_account(&pool, business_id).await;
        assert!(user_account_list_res.is_ok());
        let user_account_list = user_account_list_res.unwrap();
        let first_user_account = user_account_list.first().unwrap();
        assert!(first_user_account.id == user_id);
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }

    #[tokio::test]
    async fn test_invite_create_accept_fetch_and_delete() {
        let pool = get_test_pool().await;

        let mobile_no = "12345618935";
        let mobile_no_2 = "12345618932";
        let user_res = setup_user(
            &pool,
            "testuser39",
            "testuser39@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_res_2 = setup_user(
            &pool,
            "testuser40",
            "testuser40@example.com",
            mobile_no_2,
            "testuser@123",
        )
        .await;

        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let invite_email = EmailObject::new("sanushilshad@gmail.com".to_string());
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to create transaction.")
            .unwrap();
        let role = get_role(&pool, &UserRoleType::User.to_lowercase_string())
            .await
            .unwrap();
        let user_id = user_res.unwrap();
        let user_id_2 = user_res_2.unwrap();
        let role_id = role.unwrap().id;
        let id_res = save_business_invite_request(
            &mut transaction,
            user_id,
            business_id,
            role_id,
            &invite_email,
        )
        .await;
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")
            .unwrap();
        assert!(id_res.is_ok());
        let invite_id = id_res.unwrap();
        let data = fetch_business_invite(&pool, None, None, None, Some(vec![invite_id])).await;
        assert!(data.is_ok());
        let invite_data = data.unwrap();
        assert!(invite_data.len() == 1);
        let mut transaction_2 = pool
            .begin()
            .await
            .context("Failed to create transaction.")
            .unwrap();
        let verify_res =
            mark_invite_as_verified(&mut transaction_2, invite_id, user_id, Utc::now()).await;
        let user_business_res =
            save_user_business_relation(&mut transaction_2, user_id_2, business_id, role_id).await;
        transaction_2
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")
            .unwrap();
        print!("aaaaaa{:?}", user_business_res);
        assert!(user_business_res.is_ok());
        assert!(verify_res.is_ok());
        let fetch_association =
            fetch_associated_business_account_model(user_id, business_id, &pool).await;
        assert!(fetch_association.is_ok());
        assert!(fetch_association.unwrap().is_some());

        let invite_delete = delete_invite_by_id(&pool, invite_id).await;
        assert!(invite_delete.is_ok());
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let delete_mobile_2 = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_2);
        let (delete_business_account_res, delete_user_account_res, delete_user_account_res_2) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile),
            hard_delete_user_account(&pool, &delete_mobile_2)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
        assert!(delete_user_account_res_2.is_ok());
    }

    #[tokio::test]
    async fn test_user_business_disassociation() {
        let pool = get_test_pool().await;

        let mobile_no = "12345618936";

        let user_res = setup_user(
            &pool,
            "testuser41",
            "testuser41@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let user_id = user_res.unwrap();
        let business_id = business_res.unwrap();
        let delete_association =
            delete_user_business_relationship(&pool, user_id, business_id).await;
        assert!(delete_association.is_ok());
        let fetch_association =
            fetch_associated_business_account_model(user_id, business_id, &pool).await;
        assert!(fetch_association.is_ok());
        assert!(fetch_association.unwrap().is_none());
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile),
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }

    #[tokio::test]
    async fn test_soft_delete_business() {
        let pool = get_test_pool().await;

        let mobile_no = "12345618938";

        let user_res = setup_user(
            &pool,
            "testuser43",
            "testuser42@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let user_id = user_res.unwrap();
        let business_id = business_res.unwrap();
        let delete_association =
            soft_delete_business_account(&pool, business_id, user_id, Utc::now()).await;
        assert!(delete_association.is_ok());
        let fetch_association =
            fetch_associated_business_account_model(user_id, business_id, &pool).await;
        assert!(fetch_association.is_ok());
        assert!(fetch_association.unwrap().unwrap().is_deleted == true);
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile),
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }
}
