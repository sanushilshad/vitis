#[cfg(test)]
pub mod tests {
    use crate::{
        constants::DUMMY_INTERNATIONAL_DIALING_CODE,
        routes::{
            business::tests::tests::setup_business,
            department::{tests::tests::setup_department, utils::hard_delete_department_account},
            permission::utils::fetch_permissions_for_role,
            role::{
                schemas::{AccountRole, CreateRoleData},
                utils::{get_roles, save_role, soft_delete_role},
            },
            user::{
                tests::tests::setup_user,
                utils::{hard_delete_business_account, hard_delete_user_account},
            },
        },
        tests::tests::get_test_pool,
    };
    use chrono::Utc;
    use sqlx::PgPool;
    use tokio::join;
    use uuid::Uuid;
    pub async fn create_and_fetch_business_role_id(
        pool: &PgPool,
        role_name: &str,
        business_id: Uuid,
        user_id: Uuid,
    ) -> Result<AccountRole, anyhow::Error> {
        let data = vec![CreateRoleData {
            id: None,
            name: "Manager".to_string(),
        }];
        let role_res = save_role(&pool, &data, Some(business_id), None, user_id, Utc::now()).await;
        assert!(role_res.is_ok());

        let roles_res_by_name = get_roles(
            &pool,
            Some(business_id),
            None,
            None,
            Some(vec![role_name]),
            false,
        )
        .await;
        Ok(roles_res_by_name.unwrap().first().unwrap().clone())
    }

    #[tokio::test]
    async fn test_save_and_list_business_role() {
        let pool = get_test_pool().await;

        let mobile_no = "99345678933";
        let user_res = setup_user(
            &pool,
            "testuser56",
            "testuser56@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id: uuid::Uuid = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let data = vec![CreateRoleData {
            id: None,
            name: "Manager".to_string(),
        }];
        let role_res = save_role(&pool, &data, Some(business_id), None, user_id, Utc::now()).await;
        assert!(role_res.is_ok());

        let roles_res_by_name = get_roles(
            &pool,
            Some(business_id),
            None,
            None,
            Some(vec!["Manager"]),
            false,
        )
        .await;
        assert!(roles_res_by_name.is_ok());
        let role_list = roles_res_by_name.unwrap();
        assert!(!role_list.is_empty());

        assert!(role_list.first().unwrap().name == "Manager");

        let roles_res_by_id = get_roles(
            &pool,
            Some(business_id),
            None,
            Some(vec![role_list.first().unwrap().id]),
            None,
            false,
        )
        .await;
        assert!(roles_res_by_id.is_ok());
        let role_list = roles_res_by_id.unwrap();
        assert!(!role_list.is_empty());

        assert!(role_list.first().unwrap().name == "Manager");
        let permission_list_res =
            fetch_permissions_for_role(&pool, role_list.first().unwrap().id, business_id).await;
        assert!(permission_list_res.is_ok());
        assert!(permission_list_res.unwrap().first().is_none());
        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_business_account(&pool, business_id).await;
    }

    #[tokio::test]
    async fn test_delete_business_role() {
        let pool = get_test_pool().await;

        let mobile_no = "99395678933";
        let user_res = setup_user(
            &pool,
            "testuser57",
            "testuser57@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id: uuid::Uuid = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let role_res =
            create_and_fetch_business_role_id(&pool, "Manager", business_id, user_id).await;
        let role = role_res.unwrap();

        let delete_res = soft_delete_role(&pool, role.id, user_id, Utc::now()).await;
        assert!(delete_res.is_ok());
        let roles_res_by_id = get_roles(
            &pool,
            Some(business_id),
            None,
            Some(vec![role.id]),
            None,
            false,
        )
        .await;
        assert!(roles_res_by_id.is_ok());
        let role_list = roles_res_by_id.unwrap();
        assert!(role_list.is_empty());

        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_business_account(&pool, business_id).await;
    }

    #[tokio::test]
    async fn test_save_and_list_department_role() {
        let pool = get_test_pool().await;

        let mobile_no = "99945678933";
        let user_res = setup_user(
            &pool,
            "testuser61",
            "testuser61@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id: uuid::Uuid = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let department_res = setup_department(&pool, mobile_no, business_id).await;
        let department_id = department_res.unwrap();
        let data = vec![CreateRoleData {
            id: None,
            name: "Manager".to_string(),
        }];
        let role_res = save_role(
            &pool,
            &data,
            Some(business_id),
            Some(department_id),
            user_id,
            Utc::now(),
        )
        .await;
        assert!(role_res.is_ok());

        let roles_res_by_name = get_roles(
            &pool,
            Some(business_id),
            Some(department_id),
            None,
            Some(vec!["Manager"]),
            false,
        )
        .await;
        assert!(roles_res_by_name.is_ok());
        let role_list = roles_res_by_name.unwrap();
        assert!(!role_list.is_empty());

        assert!(role_list.first().unwrap().name == "Manager");

        let roles_res_by_id = get_roles(
            &pool,
            Some(business_id),
            Some(department_id),
            Some(vec![role_list.first().unwrap().id]),
            None,
            false,
        )
        .await;
        assert!(roles_res_by_id.is_ok());
        let role_list = roles_res_by_id.unwrap();
        assert!(!role_list.is_empty());

        assert!(role_list.first().unwrap().name == "Manager");
        let permission_list_res =
            fetch_permissions_for_role(&pool, role_list.first().unwrap().id, business_id).await;
        assert!(permission_list_res.is_ok());
        assert!(permission_list_res.unwrap().first().is_none());
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
    async fn test_delete_department_role() {
        let pool = get_test_pool().await;

        let mobile_no = "993995678933";
        let user_res = setup_user(
            &pool,
            "testuser62",
            "testuser62@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id: uuid::Uuid = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let department_res = setup_department(&pool, mobile_no, business_id).await;
        let department_id = department_res.unwrap();
        let role_res =
            create_and_fetch_business_role_id(&pool, "Manager", business_id, user_id).await;
        let role = role_res.unwrap();

        let delete_res = soft_delete_role(&pool, role.id, user_id, Utc::now()).await;
        assert!(delete_res.is_ok());
        let roles_res_by_id = get_roles(
            &pool,
            Some(business_id),
            Some(department_id),
            Some(vec![role.id]),
            None,
            false,
        )
        .await;
        assert!(roles_res_by_id.is_ok());
        let role_list = roles_res_by_id.unwrap();
        assert!(role_list.is_empty());

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
