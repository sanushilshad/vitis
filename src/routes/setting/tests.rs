#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use crate::{
        constants::INTERNATIONAL_DIALING_CODE,
        routes::{
            project::tests::tests::setup_project,
            setting::{
                models::SettingModel,
                schemas::{CreateSettingData, CreateSettingRequest},
                utils::{create_setting, fetch_setting, get_setting_value},
            },
            user::{
                tests::tests::setup_user,
                utils::{hard_delete_project_account, hard_delete_user_account},
            },
        },
        tests::tests::get_test_pool,
    };

    #[tokio::test]
    async fn test_setting_create() {
        let pool = get_test_pool().await;
        let setting_key = "order_no_prefix";
        let mobile_no = "12345678932";
        let user_res = setup_user(
            &pool,
            "testuser8",
            "testuser8@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();
        let project_res = setup_project(&pool, mobile_no, "project@example.com").await;
        let project_id = project_res.unwrap();
        let valid_settings = fetch_setting(&pool, &vec![setting_key.to_string()])
            .await
            .unwrap();
        let setting_map: HashMap<String, SettingModel> = valid_settings
            .into_iter()
            .map(|setting| (setting.key.to_owned(), setting))
            .collect();
        let req = CreateSettingRequest {
            user_id: Some(user_id),
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "RAP-".to_string(),
            }],
        };

        let create_address_res =
            create_setting(&pool, &req, user_id, project_id, &setting_map).await;
        eprint!("{:?}", create_address_res);
        assert!(create_address_res.is_ok());

        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_project_account(&pool, project_id).await;
    }

    #[tokio::test]
    async fn test_setting_fetch() {
        let pool = get_test_pool().await;
        let setting_key = "time_zone";
        let mobile_no = "12345678933";
        let user_res = setup_user(
            &pool,
            "testuser9",
            "testuser9@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();
        let project_res = setup_project(&pool, mobile_no, "project@example.com").await;
        let project_id = project_res.unwrap();
        let valid_settings = fetch_setting(&pool, &vec![setting_key.to_string()])
            .await
            .unwrap();
        let setting_map: HashMap<String, SettingModel> = valid_settings
            .into_iter()
            .map(|setting| (setting.key.to_owned(), setting))
            .collect();
        let req = CreateSettingRequest {
            user_id: Some(user_id),
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "RAP-".to_string(),
            }],
        };

        let create_address_res =
            create_setting(&pool, &req, user_id, project_id, &setting_map).await;
        assert!(create_address_res.is_ok());
        let req = CreateSettingRequest {
            user_id: None,
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "RAP-".to_string(),
            }],
        };

        let create_address_res =
            create_setting(&pool, &req, user_id, project_id, &setting_map).await;
        assert!(create_address_res.is_ok());
        let data_res =
            get_setting_value(&pool, &vec![setting_key.to_string()], project_id, user_id).await;
        print!("{:?}", &data_res);
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        assert!(data[0].project_level.len() == 1);
        assert!(data[0].user_level.len() == 1);
        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_project_account(&pool, project_id).await;
    }
}
