#[cfg(test)]
pub mod tests {
    use crate::{
        constants::{DUMMY_INTERNATIONAL_DIALING_CODE, DUMMY_TIMEZONE},
        email::EmailObject,
        routes::{
            business::tests::tests::setup_business,
            leave::{
                schemas::{
                    CreateLeaveData, CreateLeaveRequest, FetchLeaveQuery,
                    LeaveGroupCreationRequest, LeavePeriodCreationData, LeavePeriodData,
                    LeaveStatus, LeaveTypeCreationData, UserLeave, UserLeaveCreationData,
                    UserLeaveGroup, UserLeaveType,
                },
                utils::{
                    delete_leave,
                    delete_leave_group,
                    delete_leave_period,
                    delete_leave_type,
                    delete_user_leave,
                    fetch_user_leaves,
                    get_leave_group,
                    get_leave_period,
                    get_leave_type,
                    get_leaves,
                    save_leave_group,
                    save_leave_period,
                    save_leave_request,
                    save_leave_type,
                    save_user_leave,
                    update_leave_request_status,
                    validate_leave_request_creation,
                    validate_leave_status_update, //  delete_leave, get_leaves,
                                                  // save_leave_request, update_leave_status,
                                                  // validate_leave_request, validate_leave_status_update,
                },
            },
            user::{
                tests::tests::setup_user,
                utils::{hard_delete_business_account, hard_delete_user_account},
            },
        },
        schemas::{AllowedPermission, PermissionType, Status},
        tests::tests::get_test_pool,
    };
    use anyhow::Context;
    use bigdecimal::{BigDecimal, FromPrimitive};
    use chrono::{DateTime, Duration, NaiveDate, Utc};
    use chrono_tz::Tz;
    use sqlx::PgPool;
    use tokio::join;
    use uuid::Uuid;
    #[tokio::test]
    async fn test_leave_group_create_fetch_and_delete() {
        let pool = get_test_pool().await;
        let email = "testuser36@example.com";
        let mobile_no = "1234567905";
        let user_res = setup_user(&pool, "testuser36", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);
        let leave_data = LeaveGroupCreationRequest {
            id: None,
            label: "2025".to_string(),
            start_date,
            end_date,
        };

        let res = save_leave_group(&pool, &leave_data, business_id, user_id).await;
        assert!(res.is_ok());

        let leave_group_res = get_leave_group(
            &pool,
            business_id,
            Some(&vec![res.unwrap()]),
            None,
            None,
            None,
        )
        .await;
        assert!(leave_group_res.is_ok());
        let leave_group = leave_group_res.unwrap();
        assert!(leave_group.first().is_some());
        let group_id = leave_group.first().unwrap().id;

        let updated_leave_data = LeaveGroupCreationRequest {
            id: Some(group_id),
            label: "2026".to_string(),
            start_date,
            end_date,
        };

        let res = save_leave_group(&pool, &updated_leave_data, user_id, business_id).await;
        assert!(res.is_ok());

        let leave_group_res = get_leave_group(
            &pool,
            business_id,
            Some(&vec![res.unwrap()]),
            None,
            None,
            None,
        )
        .await;
        assert!(leave_group_res.is_ok());
        let leave_group = leave_group_res.unwrap();
        assert!(leave_group.first().is_some());
        let leave_type = leave_group.first().unwrap();
        assert!(leave_type.label == "2026");

        let delete_res = delete_leave_group(&pool, leave_type.id).await;
        assert!(delete_res.is_ok());
        let delete_business_account_task = hard_delete_business_account(&pool, business_id);
        let mobile_no = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let delete_user_task = hard_delete_user_account(&pool, &mobile_no);
        let (delete_business_res, delete_user_res) =
            join!(delete_business_account_task, delete_user_task);
        assert!(delete_business_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    #[tokio::test]
    async fn test_leave_period_create_fetch_and_deletion() {
        let pool = get_test_pool().await;
        let email = "testuser46@example.com";
        let mobile_no = "1334567903";
        let user_res = setup_user(&pool, "testuser46", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let leave_data = vec![
            LeavePeriodCreationData {
                id: None,
                label: "Full Day".to_string(),
                value: BigDecimal::from_f32(1.0).unwrap(),
            },
            LeavePeriodCreationData {
                id: None,
                label: "Half Day".to_string(),
                value: BigDecimal::from_f32(0.5).unwrap(),
            },
        ];

        let res = save_leave_period(&pool, &leave_data, user_id, business_id).await;
        assert!(res.is_ok());

        let leave_period_res = get_leave_period(
            &pool,
            business_id,
            None,
            Some(vec!["Full Day", "Half Day"]),
            None,
        )
        .await;
        assert!(leave_period_res.is_ok());
        let leave_data = leave_period_res.unwrap();
        assert!(leave_data.first().is_some());
        let leave_id = leave_data.first().unwrap().id;

        let update_leave_data = vec![LeavePeriodCreationData {
            id: Some(leave_id),
            label: "Full Day".to_string(),
            value: BigDecimal::from_f32(0.5).unwrap(),
        }];

        let res = save_leave_period(&pool, &update_leave_data, user_id, business_id).await;
        assert!(res.is_ok());
        let leave_type_res =
            get_leave_period(&pool, business_id, Some(&vec![leave_id]), None, None).await;
        assert!(leave_type_res.is_ok());
        let leave_data_opt = leave_type_res.unwrap();
        assert!(leave_data_opt.first().is_some());
        let leave_period = leave_data_opt.first().unwrap();
        assert!(leave_period.label == "Full Day");

        let leave_period_res =
            get_leave_type(&pool, business_id, None, None, Some("Medical".to_string())).await;
        let del_res = delete_leave_period(&pool, leave_period.id).await;
        assert!(del_res.is_ok());
        assert!(leave_period_res.is_ok());
        let delete_business_account_task = hard_delete_business_account(&pool, business_id);
        let mobile_no = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let delete_user_task = hard_delete_user_account(&pool, &mobile_no);
        let (delete_business_type_res, delete_user_res) =
            join!(delete_business_account_task, delete_user_task);
        assert!(delete_business_type_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    pub async fn create_leave_period_and_get_id(
        pool: &PgPool,
        label: &str,
        value: f32,
        user_id: Uuid,
        business_id: Uuid,
    ) -> Result<Uuid, anyhow::Error> {
        let leave_data = vec![LeavePeriodCreationData {
            id: None,
            label: label.to_string(),
            value: BigDecimal::from_f32(value).unwrap(),
        }];

        save_leave_period(pool, &leave_data, user_id, business_id).await?;

        let leave_period_res = get_leave_period(
            &pool,
            business_id,
            None,
            Some(vec!["Full Day", "Half Day"]),
            None,
        )
        .await;
        let period_id = leave_period_res.unwrap().first().unwrap().id;
        Ok(period_id)
    }

    async fn create_test_leave_group(
        pool: &PgPool,
        business_id: Uuid,
        user_id: Uuid,
        label: String,
        _start_date: DateTime<Utc>,
        _end_date: DateTime<Utc>,
    ) -> Result<Uuid, anyhow::Error> {
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);

        let leave_group_data = LeaveGroupCreationRequest {
            id: None,
            label,
            start_date,
            end_date,
        };

        save_leave_group(pool, &leave_group_data, business_id, user_id).await
    }

    #[tokio::test]
    async fn test_leave_type_create_fetch_and_deletion() {
        let pool = get_test_pool().await;
        let email = "testuser20@example.com";
        let mobile_no = "1234567903";
        let user_res = setup_user(&pool, "testuser20", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();

        let period_id =
            create_leave_period_and_get_id(&pool, "Full Day", 1.0, user_id, business_id)
                .await
                .unwrap();

        let leave_data = vec![
            LeaveTypeCreationData {
                id: None,
                label: "Casual Leave".to_string(),
                period_id_list: vec![period_id],
                allowed_dates: None,
            },
            LeaveTypeCreationData {
                id: None,
                label: "Restricted Leave".to_string(),
                period_id_list: vec![period_id],
                allowed_dates: None,
            },
        ];
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();

        let res =
            save_leave_type(&mut transaction, &leave_data, user_id, user_id, business_id).await;

        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")
            .unwrap();
        assert!(res.is_ok());

        let leave_type_res = get_leave_type(
            &pool,
            business_id,
            None,
            Some(vec!["Casual Leave", "Restricted Leave"]),
            None,
        )
        .await;
        assert!(leave_type_res.is_ok());
        let leave_data = leave_type_res.unwrap();
        assert!(leave_data.first().is_some());
        let leave_id = leave_data.first().unwrap().id;

        let update_leave_data = vec![LeaveTypeCreationData {
            id: Some(leave_id),
            label: "Medical Leave".to_string(),
            period_id_list: vec![period_id],
            allowed_dates: None,
        }];
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();
        let res = save_leave_type(
            &mut transaction,
            &update_leave_data,
            user_id,
            user_id,
            business_id,
        )
        .await;
        print!("aaaa{:?}", res);
        assert!(res.is_ok());

        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to save leave type.")
            .unwrap();

        let leave_type_res =
            get_leave_type(&pool, business_id, Some(vec![leave_id]), None, None).await;
        assert!(leave_type_res.is_ok());
        let leave_data_opt = leave_type_res.unwrap();
        assert!(leave_data_opt.first().is_some());
        let leave_type = leave_data_opt.first().unwrap();
        assert!(leave_type.label == "Medical Leave");

        let leave_type_res =
            get_leave_type(&pool, business_id, None, None, Some("Medical".to_string())).await;
        let del_res = delete_leave_type(&pool, leave_type.id).await;
        assert!(del_res.is_ok());
        assert!(leave_type_res.is_ok());
        let delete_business_account_task = hard_delete_business_account(&pool, business_id);
        let mobile_no = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let delete_user_task = hard_delete_user_account(&pool, &mobile_no);
        let (delete_business_type_res, delete_user_res) =
            join!(delete_business_account_task, delete_user_task);
        assert!(delete_business_type_res.is_ok());
        assert!(delete_user_res.is_ok());
    }

    #[tokio::test]
    async fn test_user_leave_create_fetch_and_delete() {
        let pool = get_test_pool().await;
        let email = "testuser38@example.com";
        let mobile_no = "1234575905";
        let user_res = setup_user(&pool, "testuser38", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();

        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);
        let period_id =
            create_leave_period_and_get_id(&pool, "Full Day", 1.0, user_id, business_id)
                .await
                .unwrap();
        let leave_group_data = LeaveGroupCreationRequest {
            id: None,
            label: "2025".to_string(),
            start_date,
            end_date,
        };
        let leave_type_data = vec![LeaveTypeCreationData {
            id: None,
            label: "Casual Leave".to_string(),
            period_id_list: vec![period_id],
            allowed_dates: None,
        }];
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();

        let (save_group_res, save_type_res) = tokio::join!(
            save_leave_group(&pool, &leave_group_data, business_id, user_id),
            save_leave_type(
                &mut transaction,
                &leave_type_data,
                user_id,
                user_id,
                business_id
            )
        );
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to save leave type..")
            .unwrap();
        assert!(save_group_res.is_ok());
        assert!(save_type_res.is_ok());
        let leave_group_id = save_group_res.unwrap();

        let leave_type_res = get_leave_type(
            &pool,
            business_id,
            None,
            Some(vec!["Casual Leave", "Restricted Leave"]),
            None,
        )
        .await;
        assert!(leave_type_res.is_ok());
        let leave_type_list = leave_type_res.unwrap();
        assert!(leave_type_list.first().is_some());
        let leave_type_id = leave_type_list.first().unwrap().id;

        let user_leave_data = vec![UserLeaveCreationData {
            type_id: leave_type_id,
            count: BigDecimal::from_i32(23).unwrap(),
            status: Status::Active,
        }];
        let res = save_user_leave(&pool, &user_leave_data, user_id, leave_group_id, user_id).await;
        assert!(res.is_ok());

        let user_leave_res =
            fetch_user_leaves(&pool, business_id, user_id, Some(leave_group_id), None).await;
        assert!(user_leave_res.is_ok());
        let user_leave_opt = user_leave_res.unwrap();

        assert!(user_leave_opt.first().is_some());

        let del_res =
            delete_user_leave(&pool, business_id, user_leave_opt.first().unwrap().id).await;
        assert!(del_res.is_ok());
        let user_leave_res =
            fetch_user_leaves(&pool, business_id, user_id, Some(leave_group_id), None).await;
        assert!(user_leave_res.is_ok());

        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }

    #[tokio::test]
    async fn test_leave_request_creation_validation() {
        let pool = get_test_pool().await;
        let email = "testuser17@example.com";
        let mobile_no = "1234567900";
        let user_res = setup_user(&pool, "testuser17", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let period_id =
            create_leave_period_and_get_id(&pool, "Full Day", 1.0, user_id, business_id)
                .await
                .unwrap();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);
        let leave_type_data = vec![LeaveTypeCreationData {
            id: None,
            label: "Casual Leave".to_string(),
            period_id_list: vec![period_id],
            allowed_dates: None,
        }];

        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();

        let (save_group_res, save_type_res) = tokio::join!(
            create_test_leave_group(
                &pool,
                business_id,
                user_id,
                "2025".to_string(),
                start_date,
                end_date
            ),
            save_leave_type(
                &mut transaction,
                &leave_type_data,
                user_id,
                user_id,
                business_id
            )
        );
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to save leave type..")
            .unwrap();
        assert!(save_group_res.is_ok());
        assert!(save_type_res.is_ok());
        let leave_group_id = save_group_res.unwrap();

        let leave_type_res = get_leave_type(
            &pool,
            business_id,
            None,
            Some(vec!["Casual Leave", "Restricted Leave"]),
            None,
        )
        .await;
        assert!(leave_type_res.is_ok());
        let leave_type_list = leave_type_res.unwrap();
        assert!(leave_type_list.first().is_some());
        let leave_type_id = leave_type_list.first().unwrap().id;

        let user_leave_data = vec![UserLeaveCreationData {
            type_id: leave_type_id,
            count: BigDecimal::from_i32(1).unwrap(),
            status: Status::Active,
        }];
        let res = save_user_leave(&pool, &user_leave_data, user_id, leave_group_id, user_id).await;

        assert!(res.is_ok());

        let user_leave_res =
            fetch_user_leaves(&pool, business_id, user_id, Some(leave_group_id), None).await;
        assert!(user_leave_res.is_ok());
        let user_leave_opt = user_leave_res.unwrap();

        assert!(user_leave_opt.first().is_some());
        let user_leave = user_leave_opt.first().unwrap();

        let leave_data = vec![CreateLeaveData {
            date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            period_id,
        }];
        let mut leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            user_id: Some(user_id),
            leave_data,
            user_leave_id: user_leave.id,
            send_mail: false,
        };

        let leave_request_validation = validate_leave_request_creation(&leave_request, user_leave);
        print!("Leave request validation: {:?}", leave_request_validation);
        assert!(leave_request_validation.is_ok());

        let leave_data = vec![
            CreateLeaveData {
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                period_id,
            },
            CreateLeaveData {
                date: NaiveDate::from_ymd_opt(2025, 7, 4).expect("invalid date"),
                period_id,
            },
        ];
        leave_request.leave_data = leave_data;
        let leave_request_validation = validate_leave_request_creation(&leave_request, &user_leave);
        assert!(leave_request_validation.is_err());

        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }

    #[tokio::test]
    async fn test_leave_request_creation_and_deletion() {
        let pool = get_test_pool().await;
        let email = "testuser37@example.com";
        let mobile_no = "2234567900";
        let user_res = setup_user(&pool, "testuser37", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let period_id =
            create_leave_period_and_get_id(&pool, "Full Day", 1.0, user_id, business_id)
                .await
                .unwrap();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);
        let leave_type_data = vec![LeaveTypeCreationData {
            id: None,
            label: "Casual Leave".to_string(),
            period_id_list: vec![period_id],
            allowed_dates: None,
        }];
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();
        let (save_group_res, save_type_res) = tokio::join!(
            create_test_leave_group(
                &pool,
                business_id,
                user_id,
                "2025".to_string(),
                start_date,
                end_date
            ),
            save_leave_type(
                &mut transaction,
                &leave_type_data,
                user_id,
                user_id,
                business_id
            )
        );
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to save leave type..")
            .unwrap();
        assert!(save_group_res.is_ok());
        assert!(save_type_res.is_ok());
        let leave_group_id = save_group_res.unwrap();

        let leave_type_res = get_leave_type(
            &pool,
            business_id,
            None,
            Some(vec!["Casual Leave", "Restricted Leave"]),
            None,
        )
        .await;
        assert!(leave_type_res.is_ok());
        let leave_type_list = leave_type_res.unwrap();
        assert!(leave_type_list.first().is_some());
        let leave_type_id = leave_type_list.first().unwrap().id;

        let user_leave_data = vec![UserLeaveCreationData {
            type_id: leave_type_id,
            count: BigDecimal::from_i32(5).unwrap(),
            status: Status::Active,
        }];
        let res = save_user_leave(&pool, &user_leave_data, user_id, leave_group_id, user_id).await;

        assert!(res.is_ok());

        let user_leave_res =
            fetch_user_leaves(&pool, business_id, user_id, Some(leave_group_id), None).await;
        assert!(user_leave_res.is_ok());
        let user_leave_opt = user_leave_res.unwrap();

        assert!(user_leave_opt.first().is_some());
        let user_leave = user_leave_opt.first().unwrap();

        let leave_data = vec![CreateLeaveData {
            period_id,
            date: NaiveDate::from_ymd_opt(2025, 7, 4).expect("invalid date"),
        }];
        let leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            user_id: Some(user_id),
            leave_data,
            user_leave_id: user_leave.id,
            send_mail: false,
        };
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();

        let res = save_leave_request(
            &mut transaction,
            &leave_request,
            user_leave.id,
            user_id,
            Uuid::new_v4(),
            Some("abc@gmail.com"),
        )
        .await;
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to create leave type.")
            .unwrap();
        assert!(res.is_ok());
        let query: FetchLeaveQuery<'_> = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
        let leaves = get_leaves(&pool, &query).await;
        assert!(leaves.is_ok());
        let leave_vec = leaves.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_some());
        let deleted = delete_leave(&pool, leave_opt.unwrap().id, user_id).await;
        assert!(deleted.is_ok());
        let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
        let leaves = get_leaves(&pool, &query).await;
        assert!(leaves.is_ok());
        let leave_vec = leaves.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_none());

        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }

    #[tokio::test]
    async fn test_leave_request_fetch() {
        let pool = get_test_pool().await;
        let email = "testuser21@example.com";
        let mobile_no = "1234567904";
        let user_res = setup_user(&pool, "testuser21", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();

        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);
        let period_id =
            create_leave_period_and_get_id(&pool, "Full Day", 1.0, user_id, business_id)
                .await
                .unwrap();
        let leave_type_data = vec![LeaveTypeCreationData {
            id: None,
            label: "Casual Leave".to_string(),
            period_id_list: vec![period_id],
            allowed_dates: None,
        }];
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();

        let (save_group_res, save_type_res) = tokio::join!(
            create_test_leave_group(
                &pool,
                business_id,
                user_id,
                "2025".to_string(),
                start_date,
                end_date
            ),
            save_leave_type(
                &mut transaction,
                &leave_type_data,
                user_id,
                user_id,
                business_id
            )
        );
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to create leave type.")
            .unwrap();
        assert!(save_group_res.is_ok());
        assert!(save_type_res.is_ok());
        let leave_group_id = save_group_res.unwrap();

        let leave_type_res = get_leave_type(
            &pool,
            business_id,
            None,
            Some(vec!["Casual Leave", "Restricted Leave"]),
            None,
        )
        .await;
        assert!(leave_type_res.is_ok());
        let leave_type_list = leave_type_res.unwrap();
        assert!(leave_type_list.first().is_some());
        let leave_type_id = leave_type_list.first().unwrap().id;

        let user_leave_data = vec![UserLeaveCreationData {
            type_id: leave_type_id,
            count: BigDecimal::from_i32(5).unwrap(),
            status: Status::Active,
        }];
        let res = save_user_leave(&pool, &user_leave_data, user_id, leave_group_id, user_id).await;

        assert!(res.is_ok());

        let user_leave_res =
            fetch_user_leaves(&pool, business_id, user_id, Some(leave_group_id), None).await;
        assert!(user_leave_res.is_ok());
        let user_leave_opt = user_leave_res.unwrap();

        assert!(user_leave_opt.first().is_some());
        let user_leave = user_leave_opt.first().unwrap();

        let leave_data = vec![
            CreateLeaveData {
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                period_id,
            },
            CreateLeaveData {
                period_id,
                date: NaiveDate::from_ymd_opt(2025, 7, 4).expect("invalid date"),
            },
        ];
        let leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            user_id: Some(user_id),
            leave_data,
            user_leave_id: user_leave.id,
            send_mail: false,
        };
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();
        let receiver_id = Uuid::new_v4();
        let res = save_leave_request(
            &mut transaction,
            &leave_request,
            user_leave.id,
            user_id,
            receiver_id,
            Some("abc@gmail.com"),
        )
        .await;
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")
            .unwrap();
        assert!(res.is_ok());
        let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
        let leave_with_sender = get_leaves(&pool, &query).await;
        assert!(leave_with_sender.is_ok());
        let leave_vec = leave_with_sender.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_some());
        let tz: Tz = DUMMY_TIMEZONE.parse().unwrap();
        let start_utc: DateTime<Utc> = Utc::now();
        let end_utc = start_utc - Duration::minutes(3);
        let start_in_tz = start_utc.with_timezone(&tz);
        let end_in_tz = end_utc.with_timezone(&tz);

        let start_naive = start_in_tz.naive_local();
        let end_naive = end_in_tz.naive_local();
        let query = FetchLeaveQuery::builder()
            .with_limit(Some(1))
            .with_offset(Some(0))
            .with_start_date(Some(&start_naive))
            .with_end_date(Some(&end_naive));
        let leave_with_start_and_end_date = get_leaves(&pool, &query).await;
        assert!(leave_with_start_and_end_date.is_ok());
        let leave_vec = leave_with_start_and_end_date.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_some());
        let query = FetchLeaveQuery::builder().with_leave_id(Some(leave_opt.unwrap().id));
        let leave_with_id = get_leaves(&pool, &query).await;
        assert!(leave_with_id.is_ok());
        let leave_vec = leave_with_id.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_some());
        let query = FetchLeaveQuery::builder().with_recevier_id(Some(receiver_id));
        let leave_with_receiver_id = get_leaves(&pool, &query).await;
        assert!(leave_with_receiver_id.is_ok());
        let leave_vec = leave_with_receiver_id.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_some());

        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }

    fn get_dummy_user_leave_data(
        user_id: Uuid,
        used_count: BigDecimal,
        allocated_count: BigDecimal,
    ) -> UserLeave {
        UserLeave {
            id: Uuid::new_v4(),
            allocated_count,
            used_count,
            business_id: user_id,
            user_id,
            leave_type: UserLeaveType {
                id: Uuid::new_v4(),
                label: "Casual Leave".to_owned(),
            },
            leave_group: UserLeaveGroup {
                id: Uuid::new_v4(),
                label: "2025".to_owned(),
            },
            periods: vec![LeavePeriodData {
                id: Uuid::new_v4(),
                label: "Full Day".to_string(),
                value: BigDecimal::from_i32(1).unwrap(),
            }],
            allowed_dates: None,
        }
    }

    #[tokio::test]
    async fn test_leave_request_status_validation() {
        let mut dummy_user_leave = get_dummy_user_leave_data(
            Uuid::new_v4(),
            BigDecimal::from_i32(2).unwrap(),
            BigDecimal::from_i32(5).unwrap(),
        );
        let period = LeavePeriodData {
            id: Uuid::new_v4(),
            label: "Full Day".to_string(),
            value: BigDecimal::from_f32(0.5).unwrap(),
        };
        let val_res = validate_leave_status_update(
            &LeaveStatus::Approved,
            &LeaveStatus::Rejected,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
            &dummy_user_leave,
            &period,
        );
        assert!(val_res.is_err());

        let val_res = validate_leave_status_update(
            &LeaveStatus::Approved,
            &LeaveStatus::Cancelled,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
            &dummy_user_leave,
            &period,
        );
        assert!(val_res.is_err());

        let val_res = validate_leave_status_update(
            &LeaveStatus::Approved,
            &LeaveStatus::Approved,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
            &dummy_user_leave,
            &period,
        );
        assert!(val_res.is_err());

        let val_res = validate_leave_status_update(
            &LeaveStatus::Rejected,
            &LeaveStatus::Approved,
            &AllowedPermission {
                permission_list: vec![],
            },
            &dummy_user_leave,
            &period,
        );
        assert!(val_res.is_err());
        let val_res = validate_leave_status_update(
            &LeaveStatus::Cancelled,
            &LeaveStatus::Approved,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
            &dummy_user_leave,
            &period,
        );
        assert!(val_res.is_ok());
        dummy_user_leave.used_count = BigDecimal::from_i32(5).unwrap();
        let val_res = validate_leave_status_update(
            &LeaveStatus::Cancelled,
            &LeaveStatus::Approved,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
            &dummy_user_leave,
            &period,
        );
        assert!(val_res.is_err());

        dummy_user_leave.used_count = BigDecimal::from_i32(5).unwrap();
        let val_res = validate_leave_status_update(
            &LeaveStatus::Cancelled,
            &LeaveStatus::Approved,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
            &dummy_user_leave,
            &period,
        );
        assert!(val_res.is_err());
    }

    #[tokio::test]
    async fn test_leave_request_status_updation() {
        let pool = get_test_pool().await;
        let email = "testuser19@example.com";
        let mobile_no = "1234567902";
        let user_res = setup_user(&pool, "testuser19", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        let business_id = business_res.unwrap();
        let period_id =
            create_leave_period_and_get_id(&pool, "Full Day", 1.0, user_id, business_id)
                .await
                .unwrap();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);
        let leave_type_data = vec![LeaveTypeCreationData {
            id: None,
            label: "Casual Leave".to_string(),
            period_id_list: vec![period_id],
            allowed_dates: None,
        }];
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();
        let (save_group_res, save_type_res) = tokio::join!(
            create_test_leave_group(
                &pool,
                business_id,
                user_id,
                "2025".to_string(),
                start_date,
                end_date
            ),
            save_leave_type(
                &mut transaction,
                &leave_type_data,
                user_id,
                user_id,
                business_id
            )
        );
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")
            .unwrap();
        assert!(save_group_res.is_ok());
        assert!(save_type_res.is_ok());
        let leave_group_id = save_group_res.unwrap();

        let leave_type_res = get_leave_type(
            &pool,
            business_id,
            None,
            Some(vec!["Casual Leave", "Restricted Leave"]),
            None,
        )
        .await;
        assert!(leave_type_res.is_ok());
        let leave_type_list = leave_type_res.unwrap();
        assert!(leave_type_list.first().is_some());
        let leave_type_id = leave_type_list.first().unwrap().id;

        let user_leave_data = vec![UserLeaveCreationData {
            type_id: leave_type_id,
            count: BigDecimal::from_i32(5).unwrap(),
            status: Status::Active,
        }];
        let res = save_user_leave(&pool, &user_leave_data, user_id, leave_group_id, user_id).await;

        assert!(res.is_ok());

        let user_leave_res =
            fetch_user_leaves(&pool, business_id, user_id, Some(leave_group_id), None).await;
        assert!(user_leave_res.is_ok());
        let user_leave_opt = user_leave_res.unwrap();

        assert!(user_leave_opt.first().is_some());
        let user_leave = user_leave_opt.first().unwrap();
        let leave_data = vec![CreateLeaveData {
            period_id,
            date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
        }];
        let leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            user_id: Some(user_id),
            leave_data,
            user_leave_id: user_leave.id,
            send_mail: false,
        };
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();
        let res = save_leave_request(
            &mut transaction,
            &leave_request,
            user_leave.id,
            user_id,
            Uuid::new_v4(),
            Some("abc@gmail.com"),
        )
        .await;
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")
            .unwrap();
        assert!(res.is_ok());
        let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
        let leaves = get_leaves(&pool, &query).await;
        assert!(leaves.is_ok());
        let leave_vec = leaves.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_some());
        let leave = leave_opt.unwrap();
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")
            .unwrap();
        let res = update_leave_request_status(
            &mut transaction,
            leave.id,
            &LeaveStatus::Approved,
            user_id,
        )
        .await;
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")
            .unwrap();

        assert!(res.is_ok());
        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }
}
