#[cfg(test)]
pub mod tests {
    use crate::constants::INTERNATIONAL_DIALING_CODE;
    use crate::email::EmailObject;
    use crate::routes::project::schemas::{CreateprojectAccount, ProjectAccount};
    use crate::routes::project::utils::{
        create_project_account, fetch_project_account_model_by_id, get_basic_project_accounts,
        get_basic_project_accounts_by_user_id, get_project_account,
        validate_project_account_active, validate_user_project_permission,
    };

    use crate::routes::user::tests::tests::setup_user;
    use crate::routes::user::utils::{
        get_user, hard_delete_project_account, hard_delete_user_account,
    };

    use crate::schemas::Status;

    use crate::tests::tests::get_test_pool;
    use sqlx::PgPool;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_validate_active_project_account() {
        let mut project_account = ProjectAccount {
            id: Uuid::new_v4(),
            name: "SANU PRIVATE LIMITED".to_string(),
            vectors: vec![],
            is_active: Status::Active,
            is_deleted: false,
            verified: true,
        };

        // Test case 5: project account is inactive
        project_account.is_active = Status::Inactive;
        let validate_response = validate_project_account_active(&project_account);
        assert_eq!(
            validate_response,
            Some("project Account is inactive".to_string())
        );

        // Test case 6: project account is deleted
        project_account.is_active = Status::Active;
        project_account.is_deleted = true;
        let validate_response = validate_project_account_active(&project_account);
        assert_eq!(
            validate_response,
            Some("project Account is deleted".to_string())
        );

        // Test case 7: project user relation is not verified
        project_account.is_deleted = false;
        project_account.verified = false;
        let validate_response = validate_project_account_active(&project_account);
        assert_eq!(
            validate_response,
            Some("project User relation is not verified".to_string())
        );

        // Test case 8: All conditions are met
        project_account.verified = true;
        let validate_response = validate_project_account_active(&project_account);
        assert_eq!(validate_response, None);
    }

    pub async fn setup_project(
        pool: &PgPool,
        mobile_no: &str,
        email: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let user_obj = get_user(
            vec![&format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no)],
            pool,
        )
        .await;
        let create_project_obj = CreateprojectAccount {
            name: "Test Company".to_string(),
            is_test_account: false,

            mobile_no: mobile_no.to_string(),
            email: EmailObject::new(email.to_string()),

            international_dialing_code: INTERNATIONAL_DIALING_CODE.to_string(),
        };
        let project_res_obj =
            create_project_account(pool, &user_obj.unwrap(), &create_project_obj).await?;
        Ok(project_res_obj)
    }

    #[tokio::test]
    async fn test_project_and_fetch() {
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
        let project_res = setup_project(&pool, &mobile_no, "project@example.com").await;
        assert!(project_res.is_ok());
        let project_id = project_res.unwrap();
        let fetch_basic_project_obj_res =
            get_basic_project_accounts_by_user_id(user_id, &pool).await;
        assert!(fetch_basic_project_obj_res.is_ok());

        let fetch_basic_project_obj_res =
            get_basic_project_accounts_by_user_id(user_id, &pool).await;
        assert!(fetch_basic_project_obj_res.is_ok());

        let fetch_basic_business_objs = get_basic_project_accounts(&pool).await;
        eprint!("Basic Project Accounts: {:?}", fetch_basic_business_objs);
        assert!(fetch_basic_business_objs.is_ok());

        let fetch_project_obj_res = get_project_account(&pool, user_id, project_id).await;
        assert!(fetch_project_obj_res.is_ok());

        let fetch_business_obj_by_id =
            fetch_project_account_model_by_id(&pool, Some(project_id)).await;
        eprint!("Business Account List: {:?}", fetch_business_obj_by_id);

        let delete_bus_res = hard_delete_project_account(&pool, project_id).await;
        assert!(delete_bus_res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_project_permission_validation() {
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
        let project_res = setup_project(&pool, mobile_no, "project@example.com").await;
        let project_id = project_res.unwrap();
        let permission_res = validate_user_project_permission(
            &pool,
            user_id,
            project_id,
            &vec!["create:setting".to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() > 0);
        let permission_res = validate_user_project_permission(
            &pool,
            user_id,
            project_id,
            &vec!["create:setting1".to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() == 0);
        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_project_account(&pool, project_id).await;

        let permission_res = validate_user_project_permission(
            &pool,
            user_id,
            project_id,
            &vec!["create:setting1".to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() == 0);
    }

    #[tokio::test]
    async fn test_list_associated_project_accounts() {
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
        let project_res = setup_project(&pool, mobile_no, "project@example.com").await;
        let project_id = project_res.unwrap();
        let project_account_list_res = get_basic_project_accounts_by_user_id(user_id, &pool).await;
        assert!(project_account_list_res.is_ok());
        let project_account_list = project_account_list_res.unwrap();
        let frst_project_account = project_account_list.first().unwrap();
        assert!(frst_project_account.id == project_id);
        let delete_bus_res = hard_delete_project_account(&pool, project_id).await;
        assert!(delete_bus_res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }
}
