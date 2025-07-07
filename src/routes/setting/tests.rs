#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use tokio::join;

    use crate::{
        constants::DUMMY_INTERNATIONAL_DIALING_CODE,
        routes::{
            business::tests::tests::setup_business,
            setting::{
                models::SettingModel,
                schemas::{
                    CreateBusinessSettingRequest, CreateSettingData, SettingKey, SettingType,
                },
                utils::{
                    create_setting_with_scope, delete_global_setting, fetch_setting,
                    fetch_setting_enums, get_setting_value,
                },
            },
            user::{
                tests::tests::setup_user,
                utils::{hard_delete_business_account, hard_delete_user_account},
            },
        },
        tests::tests::get_test_pool,
    };

    #[tokio::test]
    async fn test_business_setting_create_fetch() {
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
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let valid_settings =
            fetch_setting(&pool, &vec![setting_key.to_string()], SettingType::Business)
                .await
                .unwrap();
        let setting_map: HashMap<String, &SettingModel> = valid_settings
            .iter()
            .map(|setting| (setting.key.to_owned(), setting))
            .collect();
        // let req_user_level = CreateBusinessSettingRequest {
        //     user_id: Some(user_id),
        //     settings: vec![CreateSettingData {
        //         key: setting_key.to_owned(),
        //         value: "Asia/Kolkata".to_string(),
        //     }],
        // };

        let req_business_level = CreateBusinessSettingRequest {
            user_id: None,
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "Asia/Kolkata".to_string(),
            }],
        };
        let create_setting_res_business = create_setting_with_scope(
            &pool,
            &req_business_level.settings,
            None,
            Some(business_id),
            user_id,
            &setting_map,
        )
        .await;

        assert!(create_setting_res_business.is_ok());
        let data_res = get_setting_value(
            &pool,
            &vec![setting_key.to_string()],
            Some(business_id),
            Some(user_id),
            true,
        )
        .await;
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        assert!(data[0].business_level.len() == 1);
        // assert!(data[0].user_level.len() == 1);
        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_business_account(&pool, business_id).await;
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

        let valid_settings =
            fetch_setting(&pool, &vec![setting_key.to_string()], SettingType::User)
                .await
                .unwrap();
        let setting_map: HashMap<String, &SettingModel> = valid_settings
            .iter()
            .map(|setting| (setting.key.to_owned(), setting))
            .collect();
        let setting = vec![CreateSettingData {
            key: setting_key.to_owned(),
            value: "Asia/Kolkata".to_string(),
        }];

        let create_setting_res =
            create_setting_with_scope(&pool, &setting, Some(user_id), None, user_id, &setting_map)
                .await;
        assert!(create_setting_res.is_ok());
        let data_res = get_setting_value(
            &pool,
            &vec![setting_key.to_string()],
            None,
            Some(user_id),
            true,
        )
        .await;
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        assert!(data[0].user_level.len() == 1);

        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
    }

    #[tokio::test]
    async fn test_global_setting_creation_and_fetch() {
        let pool = get_test_pool().await;
        let setting_key = "time_zone";
        let mobile_no = "12345671937";
        let user_res = setup_user(
            &pool,
            "testuser28",
            "testuser28@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();

        let valid_settings =
            fetch_setting(&pool, &vec![setting_key.to_string()], SettingType::Global)
                .await
                .unwrap();
        let setting_map: HashMap<String, &SettingModel> = valid_settings
            .iter()
            .map(|setting| (setting.key.to_owned(), setting))
            .collect();
        let setting = vec![CreateSettingData {
            key: setting_key.to_owned(),
            value: "Asia/Kolkata".to_string(),
        }];

        let create_setting_res =
            create_setting_with_scope(&pool, &setting, None, None, user_id, &setting_map).await;
        assert!(create_setting_res.is_ok());
        let data_res =
            get_setting_value(&pool, &vec![setting_key.to_string()], None, None, true).await;
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        assert!(data[0].global_level.len() == 1);
        let full_mobile_no = &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_setting_res, delete_user_res) = join!(
            delete_global_setting(&pool),
            hard_delete_user_account(&pool, full_mobile_no)
        );

        assert!(delete_setting_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    #[tokio::test]
    async fn test_enum_fetch() {
        let pool = get_test_pool().await;
        let setting_key = vec![SettingKey::TimeZone.to_string()];
        let setting_res = fetch_setting(&pool, &setting_key, SettingType::Global).await;
        assert!(setting_res.is_ok());
        let setting_list = setting_res.unwrap();
        let setting_opt = setting_list.first();
        assert!(setting_opt.is_some());
        let setting = setting_opt.unwrap();
        assert!(setting.enum_id.is_some());
        let enum_id = setting.enum_id.unwrap();
        let enums = fetch_setting_enums(&pool, &vec![enum_id]).await;
        assert!(enums.is_ok());
        assert!(enums.unwrap().first().is_some());
    }

    #[tokio::test]
    async fn test_user_business_setting_create_fetch() {
        let pool = get_test_pool().await;
        let setting_key = "time_zone";
        let mobile_no = "12245668933";
        let user_res = setup_user(
            &pool,
            "testuser33",
            "testuser33@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;

        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let valid_settings =
            fetch_setting(&pool, &vec![setting_key.to_string()], SettingType::Business)
                .await
                .unwrap();
        let setting_map: HashMap<String, &SettingModel> = valid_settings
            .iter()
            .map(|setting| (setting.key.to_owned(), setting))
            .collect();
        // let req_user_level = CreateBusinessSettingRequest {
        //     user_id: Some(user_id),
        //     settings: vec![CreateSettingData {
        //         key: setting_key.to_owned(),
        //         value: "Asia/Kolkata".to_string(),
        //     }],
        // };

        let req_business_level = CreateBusinessSettingRequest {
            user_id: None,
            settings: vec![CreateSettingData {
                key: setting_key.to_owned(),
                value: "Asia/Kolkata".to_string(),
            }],
        };
        let create_setting_res_business = create_setting_with_scope(
            &pool,
            &req_business_level.settings,
            Some(user_id),
            Some(business_id),
            user_id,
            &setting_map,
        )
        .await;
        assert!(create_setting_res_business.is_ok());
        let data_res = get_setting_value(
            &pool,
            &vec![setting_key.to_string()],
            Some(business_id),
            Some(user_id),
            true,
        )
        .await;
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        // assert!(data[0].business_level.len() == 1);
        // assert!(data[0].user_level.len() == 1);
        assert!(data[0].user_business_level.len() == 1);
        let _ = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        let _ = hard_delete_business_account(&pool, business_id).await;
    }
}
