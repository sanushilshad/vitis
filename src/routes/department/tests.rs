#[cfg(test)]
pub mod tests {
    use crate::constants::DUMMY_INTERNATIONAL_DIALING_CODE;

    use crate::routes::business::tests::tests::setup_business;
    use crate::routes::business::utils::get_business_account;
    use crate::routes::department::schemas::{
        CreateDepartmentAccount, DepartmentAccount, UpdateDepartmentAccount,
    };
    use crate::routes::department::utils::{
        associate_user_to_department, create_department_account,
        delete_user_department_relationship, fetch_associated_department_account_model,
        get_basic_department_accounts, get_basic_department_accounts_by_user_id,
        get_department_account, hard_delete_department_account, soft_delete_department_account,
        update_department_account, validate_department_account_active,
        validate_user_department_permission,
    };

    use crate::routes::role::utils::get_role;
    use crate::routes::user::schemas::UserRoleType;
    use crate::routes::user::tests::tests::setup_user;
    use crate::routes::user::utils::{
        get_user, hard_delete_business_account, hard_delete_user_account,
    };

    use crate::schemas::Status;

    use crate::tests::tests::get_test_pool;
    use chrono::Utc;
    use sqlx::PgPool;
    use tokio::join;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_validate_active_department_account() {
        let mut department_account = DepartmentAccount {
            id: Uuid::new_v4(),
            display_name: "Dev Department".to_string(),
            is_active: Status::Active,
            is_deleted: false,
            verified: true,
        };

        // Test case 5:department account is inactive
        department_account.is_active = Status::Inactive;
        let validate_response = validate_department_account_active(&department_account);
        assert_eq!(
            validate_response,
            Some("department Account is inactive".to_string())
        );

        // Test case 6:department account is deleted
        department_account.is_active = Status::Active;
        department_account.is_deleted = true;
        let validate_response = validate_department_account_active(&department_account);
        assert_eq!(
            validate_response,
            Some("department Account is deleted".to_string())
        );

        // Test case 7:department user relation is not verified
        department_account.is_deleted = false;
        department_account.verified = false;
        let validate_response = validate_department_account_active(&department_account);
        assert_eq!(
            validate_response,
            Some("department User relation is not verified".to_string())
        );

        // Test case 8: All conditions are met
        department_account.verified = true;
        let validate_response = validate_department_account_active(&department_account);
        assert_eq!(validate_response, None);
    }

    pub async fn setup_department(
        pool: &PgPool,
        mobile_no: &str,
        business_id: Uuid,
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
        let user = user_opt.unwrap();
        let business_account_res = get_business_account(&pool, user.id, business_id).await;
        let business_account = business_account_res.unwrap().unwrap();
        let create_department_obj = CreateDepartmentAccount {
            display_name: "Dev Account".to_string(),
            is_test_account: false,

            international_dialing_code: DUMMY_INTERNATIONAL_DIALING_CODE.to_string(),
        };
        let department_res_obj =
            create_department_account(pool, &user, &business_account, &create_department_obj)
                .await?;
        Ok(department_res_obj)
    }

    #[tokio::test]
    async fn test_department_and_fetch() {
        let pool = get_test_pool().await;

        let mobile_no = "1334567892";
        let final_mobile_no = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let user_res = setup_user(
            &pool,
            "testuser29",
            "testuser29@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;

        let business_id = business_res.unwrap();
        let department_res = setup_department(&pool, &mobile_no, business_id).await;
        assert!(department_res.is_ok());
        let department_id = department_res.unwrap();
        let fetch_basic_department_obj_res =
            get_basic_department_accounts_by_user_id(user_id, business_id, &pool).await;
        assert!(fetch_basic_department_obj_res.is_ok());

        let fetch_basic_department_obj_res =
            get_basic_department_accounts_by_user_id(user_id, business_id, &pool).await;
        assert!(fetch_basic_department_obj_res.is_ok());

        let fetch_basic_business_objs = get_basic_department_accounts(&pool).await;
        print!("aaa{:?}", fetch_basic_business_objs);
        assert!(fetch_basic_business_objs.is_ok());

        let fetch_department_obj_res =
            get_department_account(&pool, user_id, business_id, department_id).await;
        assert!(fetch_department_obj_res.is_ok());

        let (delete_dep_res, delete_business_res, delete_user_res) = join!(
            hard_delete_department_account(&pool, department_id),
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &final_mobile_no),
        );

        assert!(delete_dep_res.is_ok());
        assert!(delete_business_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    #[tokio::test]
    async fn test_department_permission_validation() {
        let pool = get_test_pool().await;

        let mobile_no = "13345678929";
        let final_mobile_no = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let user_res = setup_user(
            &pool,
            "testuser30",
            "testuser30@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;

        let business_id = business_res.unwrap();

        let user_id = user_res.unwrap();
        let department_res = setup_department(&pool, mobile_no, business_id).await;
        let department_id = department_res.unwrap();
        let permission_res = validate_user_department_permission(
            &pool,
            user_id,
            business_id,
            department_id,
            &vec!["create:department-setting".to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() > 0);
        let permission_res = validate_user_department_permission(
            &pool,
            user_id,
            business_id,
            department_id,
            &vec!["create:setting1".to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() == 0);

        let (delete_dep_res, delete_business_res, delete_user_res) = join!(
            hard_delete_department_account(&pool, department_id),
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &final_mobile_no),
        );

        assert!(delete_dep_res.is_ok());
        assert!(delete_business_res.is_ok());
        assert!(delete_user_res.is_ok());
        let permission_res = validate_user_department_permission(
            &pool,
            user_id,
            business_id,
            department_id,
            &vec!["create:setting1".to_string()],
        )
        .await;
        assert!(permission_res.unwrap().len() == 0);
    }

    #[tokio::test]
    async fn test_department_user_association() {
        let pool = get_test_pool().await;

        let mobile_no_1 = "13345678939";
        let mobile_no_2 = "13345678949";
        let mobile_with_code_1 = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_1);
        let mobile_with_code_2 = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no_2);
        // Create two users concurrently
        let (user_res_1, user_res_2) = join!(
            setup_user(
                &pool,
                "testuser32",
                "testuser32@example.com",
                mobile_no_1,
                "testuser@123",
            ),
            setup_user(
                &pool,
                "testuser35",
                "testuser35q@example.com",
                mobile_no_2,
                "testuser@123",
            )
        );

        let user_id_1 = user_res_1.unwrap();
        let user_id_2 = user_res_2.unwrap();

        let business_res = setup_business(&pool, &mobile_no_1, "business@example.com").await;

        let business_id = business_res.unwrap();
        //department and role fetching can happen concurrently
        let role = UserRoleType::Admin.to_lowercase_string();
        let (department_res, role_obj_opt) = join!(
            setup_department(&pool, mobile_no_1, business_id),
            get_role(&pool, &role),
        );

        let department_id = department_res.unwrap();
        let role_obj_opt = role_obj_opt.unwrap();
        assert!(role_obj_opt.is_some());
        let role_obj = role_obj_opt.unwrap();
        // Associate user todepartment

        let user_department_association = associate_user_to_department(
            &pool,
            user_id_2,
            business_id,
            department_id,
            role_obj.id,
            user_id_1,
        )
        .await;
        assert!(user_department_association.is_ok());

        // Fetch and assert association
        let fetched_association =
            get_department_account(&pool, user_id_2, business_id, department_id)
                .await
                .unwrap();
        assert!(fetched_association.is_some());

        // Perform deletions concurrently

        let (delete_dep_res, delete_business_res, delete_res_1, delete_res_2) = join!(
            hard_delete_department_account(&pool, department_id),
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &mobile_with_code_1,),
            hard_delete_user_account(&pool, &mobile_with_code_2),
        );

        assert!(delete_dep_res.is_ok());
        assert!(delete_business_res.is_ok());
        assert!(delete_res_1.is_ok());
        assert!(delete_res_2.is_ok());
    }

    #[tokio::test]
    async fn test_list_associated_department_accounts() {
        let pool = get_test_pool().await;

        let mobile_no = "13345678934";
        let final_mobile_no = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let user_res = setup_user(
            &pool,
            "testuser31",
            "testuser31@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        // .await;
        let business_res = setup_business(&pool, &mobile_no, "business@example.com").await;

        let business_id = business_res.unwrap();
        let user_id = user_res.unwrap();
        let department_res = setup_department(&pool, mobile_no, business_id).await;
        let department_id = department_res.unwrap();

        let department_account_list_res =
            get_basic_department_accounts_by_user_id(user_id, business_id, &pool).await;
        assert!(department_account_list_res.is_ok());
        let department_account_list = department_account_list_res.unwrap();
        let first_department_account = department_account_list.first().unwrap();
        assert!(first_department_account.id == department_id);

        let (delete_dep_res, delete_business_res, delete_user_res) = join!(
            hard_delete_department_account(&pool, department_id),
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &final_mobile_no),
        );

        assert!(delete_dep_res.is_ok());
        assert!(delete_business_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    #[tokio::test]
    async fn test_user_department_disassociation() {
        let pool = get_test_pool().await;

        let mobile_no = "12345618930";

        let user_res = setup_user(
            &pool,
            "testuser58",
            "testuser58@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let user_id = user_res.unwrap();
        let business_id = business_res.unwrap();
        let department_res = setup_department(&pool, mobile_no, business_id).await;
        let department_id = department_res.unwrap();
        let delete_association =
            delete_user_department_relationship(&pool, user_id, business_id, department_id).await;
        assert!(delete_association.is_ok());
        let fetch_association =
            fetch_associated_department_account_model(user_id, department_id, business_id, &pool)
                .await;
        assert!(fetch_association.is_ok());
        assert!(fetch_association.unwrap().is_none());
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_dep_res, delete_business_res, delete_user_res) = join!(
            hard_delete_department_account(&pool, department_id),
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile),
        );

        assert!(delete_dep_res.is_ok());
        assert!(delete_business_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    #[tokio::test]
    async fn test_soft_delete_department() {
        let pool = get_test_pool().await;

        let mobile_no = "52345618938";

        let user_res = setup_user(
            &pool,
            "testuser59",
            "testuser59@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let user_id = user_res.unwrap();
        let business_id = business_res.unwrap();
        let department_res = setup_department(&pool, mobile_no, business_id).await;
        let department_id = department_res.unwrap();
        let delete_association =
            soft_delete_department_account(&pool, department_id, user_id, Utc::now()).await;
        assert!(delete_association.is_ok());
        let fetch_association =
            fetch_associated_department_account_model(user_id, department_id, business_id, &pool)
                .await;
        assert!(fetch_association.is_ok());
        assert!(fetch_association.unwrap().unwrap().is_deleted == true);
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_dep_res, delete_business_res, delete_user_res) = join!(
            hard_delete_department_account(&pool, department_id),
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile),
        );
        assert!(delete_dep_res.is_ok());
        assert!(delete_business_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    #[tokio::test]
    async fn test_update_department() {
        let pool = get_test_pool().await;

        let mobile_no = "99345678938";

        let user_res = setup_user(
            &pool,
            "testuser60",
            "testuser60@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let user_id = user_res.unwrap();
        let business_id = business_res.unwrap();
        let department_res = setup_department(&pool, mobile_no, business_id).await;
        let department_id = department_res.unwrap();
        let update_req = UpdateDepartmentAccount {
            display_name: "testuser99".to_string(),
        };
        let department_account_res =
            get_department_account(&pool, user_id, business_id, department_id).await;
        let department_account = department_account_res.unwrap().unwrap();
        let update_res =
            update_department_account(&pool, &update_req, &department_account, user_id).await;
        assert!(update_res.is_ok());
        let business_account_res =
            get_department_account(&pool, user_id, business_id, department_id).await;
        let business_account = business_account_res.unwrap().unwrap();
        assert!(business_account.display_name == "testuser99");
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_dep_res, delete_business_res, delete_user_res) = join!(
            hard_delete_department_account(&pool, department_id),
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile),
        );
        assert!(delete_dep_res.is_ok());
        assert!(delete_business_res.is_ok());
        assert!(delete_user_res.is_ok());
    }
}
