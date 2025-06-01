#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use crate::{
        constants::INTERNATIONAL_DIALING_CODE,
        routes::{
            project::tests::tests::setup_project,
            setting::{
                models::SettingModel,
                schemas::{
                    CreateProjectSettingRequest, CreateSettingData, CreateUserSettingRequest,
                },
                utils::{
                    create_project_setting, create_user_setting, fetch_setting, get_setting_value,
                },
            },
            user::{
                tests::tests::setup_user,
                utils::{hard_delete_project_account, hard_delete_user_account},
            },
        },
        tests::tests::get_test_pool,
    };

    #[tokio::test]
    async fn test_project_setting_create_fetch() {
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
        let req_user_level = CreateProjectSettingRequest {
            user_id: Some(user_id),
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "Asia/Kolkata".to_string(),
            }],
        };

        let req_project_level = CreateProjectSettingRequest {
            user_id: None,
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "Asia/Kolkata".to_string(),
            }],
        };

        let (create_setting_res_user, create_setting_res_project) = tokio::join!(
            create_project_setting(&pool, &req_user_level, user_id, project_id, &setting_map),
            create_project_setting(&pool, &req_project_level, user_id, project_id, &setting_map),
        );

        assert!(create_setting_res_user.is_ok());
        assert!(create_setting_res_project.is_ok());
        let data_res = get_setting_value(
            &pool,
            &vec![setting_key.to_string()],
            Some(project_id),
            user_id,
        )
        .await;
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

    #[tokio::test]
    async fn test_user_setting_creation_and_fetch() {
        let pool = get_test_pool().await;
        let setting_key = "time_zone";
        let mobile_no = "12345678937";
        let user_res = setup_user(
            &pool,
            "testuser16",
            "testuser16@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();

        let valid_settings = fetch_setting(&pool, &vec![setting_key.to_string()])
            .await
            .unwrap();
        let setting_map: HashMap<String, SettingModel> = valid_settings
            .into_iter()
            .map(|setting| (setting.key.to_owned(), setting))
            .collect();
        let req = CreateUserSettingRequest {
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "Asia/Kolkata".to_string(),
            }],
        };

        let create_setting_res = create_user_setting(&pool, &req, user_id, &setting_map).await;
        assert!(create_setting_res.is_ok());
        let data_res =
            get_setting_value(&pool, &vec![setting_key.to_string()], None, user_id).await;
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        assert!(data[0].user_level.len() == 1);

        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
    }
}
