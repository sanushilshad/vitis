#[cfg(test)]
pub mod tests {
    use crate::{
        constants::DUMMY_INTERNATIONAL_DIALING_CODE,
        routes::{
            business::tests::tests::setup_business,
            permission::{
                schemas::PermissionLevel,
                utils::{
                    associate_permission_to_role, delete_role_permission_associations,
                    fetch_permissions_by_scope, fetch_permissions_for_role,
                },
            },
            role::tests::tests::create_and_fetch_business_role_id,
            user::{
                tests::tests::setup_user,
                utils::{hard_delete_business_account, hard_delete_user_account},
            },
        },
        tests::tests::get_test_pool,
    };

    #[tokio::test]
    async fn test_list_permissions() {
        let pool = get_test_pool().await;
        let permissions =
            fetch_permissions_by_scope(&pool, vec![PermissionLevel::Business], None).await;
        assert!(permissions.is_ok());
        assert!(!permissions.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_associate_permission_to_role() {
        let pool = get_test_pool().await;

        let mobile_no = "92345678933";
        let user_res = setup_user(
            &pool,
            "testuser55",
            "testuser55@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let permissions =
            fetch_permissions_by_scope(&pool, vec![PermissionLevel::Business], None).await;
        let permission_id = permissions.unwrap().first().unwrap().id;
        let role_res =
            create_and_fetch_business_role_id(&pool, "Manager", business_id, user_id).await;
        let role = role_res.unwrap();
        let association =
            associate_permission_to_role(&pool, role.id, vec![permission_id], user_id).await;
        assert!(association.is_ok());
        let permission_list_res = fetch_permissions_for_role(&pool, role.id, business_id).await;
        assert!(permission_list_res.is_ok());
        assert!(permission_list_res.unwrap().first().unwrap().id == permission_id);

        let disassociation =
            delete_role_permission_associations(&pool, vec![permission_id], role.id).await;
        assert!(disassociation.is_ok());
        let permission_list_res = fetch_permissions_for_role(&pool, role.id, business_id).await;
        assert!(permission_list_res.is_ok());
        assert!(permission_list_res.unwrap().first().is_none());
        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_business_account(&pool, business_id).await;
    }
}
