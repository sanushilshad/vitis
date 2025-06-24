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
                    LeaveGroupCreationRequest, LeavePeriod, LeaveStatus, LeaveTypeCreationData,
                    UserLeaveCreationData,
                },
                utils::{
                    delete_leave_group,
                    delete_leave_type,
                    delete_user_leave,
                    fetch_user_leaves,
                    get_leave_group,
                    get_leave_type,
                    save_leave_group,
                    save_leave_type,
                    save_user_leave,
                    validate_leave_request_creation, //  delete_leave, get_leaves,
                                                     // save_leave_request, update_leave_status,
                                                     // validate_leave_request, validate_leave_status_update,
                },
            },
            user::{
                self,
                tests::tests::setup_user,
                utils::{hard_delete_business_account, hard_delete_user_account},
            },
        },
        schemas::{AllowedPermission, PermissionType, Status},
        tests::tests::get_test_pool,
    };
    use anyhow::Context;
    use bigdecimal::{BigDecimal, FromPrimitive};
    use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
    use chrono_tz::Tz;
    use tokio::join;
    use uuid::Uuid;

    // #[tokio::test]
    // async fn test_save_leave_request() {
    //     let pool = get_test_pool().await;
    //     let email = "testuser18@example.com";
    //     let mobile_no = "1234567901";
    //     let user_res = setup_user(&pool, "testuser18", email, mobile_no, "testuser@123").await;
    //     assert!(user_res.is_ok());
    //     let user_id = user_res.unwrap();

    //     let leave_request = CreateLeaveRequest {
    //         to: EmailObject::new(email.to_string()),
    //         cc: None,
    //         reason: None,
    //         r#type: LeaveType::Casual,
    //         user_id: Some(user_id),
    //         leave_data: vec![
    //             CreateLeaveData {
    //                 period: LeavePeriod::HalfDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //             CreateLeaveData {
    //                 period: LeavePeriod::FullDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //         ],
    //     };
    //     let mut transaction = pool
    //         .begin()
    //         .await
    //         .context("Failed to acquire a Postgres connection from the pool")
    //         .unwrap();
    //     let res = save_leave_request(
    //         &mut transaction,
    //         &leave_request,
    //         user_id,
    //         Uuid::new_v4(),
    //         "abc@gmail.com",
    //     )
    //     .await;
    //     transaction
    //         .commit()
    //         .await
    //         .context("Failed to commit SQL transaction to store a new user account.")
    //         .unwrap();
    //     assert!(res.is_ok());
    //     let delete_res = hard_delete_user_account(
    //         &pool,
    //         &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
    //     )
    //     .await;
    //     assert!(delete_res.is_ok());
    // }

    // #[tokio::test]
    // async fn test_leave_request_status_validation() {
    //     let val_res = validate_leave_status_update(
    //         &LeaveStatus::Approved,
    //         &LeaveStatus::Rejected,
    //         &AllowedPermission {
    //             permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
    //         },
    //     );
    //     assert!(val_res.is_err());

    //     let val_res = validate_leave_status_update(
    //         &LeaveStatus::Approved,
    //         &LeaveStatus::Cancelled,
    //         &AllowedPermission {
    //             permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
    //         },
    //     );
    //     assert!(val_res.is_err());

    //     let val_res = validate_leave_status_update(
    //         &LeaveStatus::Approved,
    //         &LeaveStatus::Approved,
    //         &AllowedPermission {
    //             permission_list: vec![],
    //         },
    //     );
    //     assert!(val_res.is_err());

    //     let val_res = validate_leave_status_update(
    //         &LeaveStatus::Rejected,
    //         &LeaveStatus::Approved,
    //         &AllowedPermission {
    //             permission_list: vec![],
    //         },
    //     );
    //     assert!(val_res.is_err());
    // }

    // #[tokio::test]
    // async fn test_leave_request_status_updation() {
    //     let pool = get_test_pool().await;
    //     let email = "testuser19@example.com";
    //     let mobile_no = "1234567902";
    //     let user_res = setup_user(&pool, "testuser19", email, mobile_no, "testuser@123").await;
    //     assert!(user_res.is_ok());
    //     let user_id = user_res.unwrap();
    //     let leave_request = CreateLeaveRequest {
    //         to: EmailObject::new(email.to_string()),
    //         cc: None,
    //         reason: None,
    //         r#type: LeaveType::Casual,
    //         user_id: Some(user_id),
    //         leave_data: vec![
    //             CreateLeaveData {
    //                 period: LeavePeriod::HalfDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //             CreateLeaveData {
    //                 period: LeavePeriod::FullDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //         ],
    //     };
    //     let mut transaction = pool
    //         .begin()
    //         .await
    //         .context("Failed to acquire a Postgres connection from the pool")
    //         .unwrap();
    //     let res = save_leave_request(
    //         &mut transaction,
    //         &leave_request,
    //         user_id,
    //         Uuid::new_v4(),
    //         "abc@gmail.com",
    //     )
    //     .await;
    //     transaction
    //         .commit()
    //         .await
    //         .context("Failed to commit SQL transaction to store a new user account.")
    //         .unwrap();
    //     assert!(res.is_ok());
    //     let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
    //     let leaves = get_leaves(&pool, &query).await;
    //     eprint!("aaaa{:?}", leaves);
    //     assert!(leaves.is_ok());
    //     let leave_vec = leaves.unwrap();
    //     let leave_opt = leave_vec.first();
    //     assert!(leave_opt.is_some());
    //     let leave = leave_opt.unwrap();
    //     let mut transaction = pool
    //         .begin()
    //         .await
    //         .context("Failed to acquire a Postgres connection from the pool")
    //         .unwrap();
    //     let res =
    //         update_leave_status(&mut transaction, leave.id, &LeaveStatus::Approved, user_id).await;
    //     transaction
    //         .commit()
    //         .await
    //         .context("Failed to commit SQL transaction to store a new user account.")
    //         .unwrap();

    //     assert!(res.is_ok());
    //     let delete_res = hard_delete_user_account(
    //         &pool,
    //         &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
    //     )
    //     .await;
    //     assert!(delete_res.is_ok());
    // }

    // #[tokio::test]
    // async fn test_leave_request_deletion() {
    //     let pool = get_test_pool().await;
    //     let email = "testuser22@example.com";
    //     let mobile_no = "1234567903";
    //     let user_res = setup_user(&pool, "testuser22", email, mobile_no, "testuser@123").await;
    //     assert!(user_res.is_ok());
    //     let user_id = user_res.unwrap();
    //     let leave_request = CreateLeaveRequest {
    //         to: EmailObject::new(email.to_string()),
    //         cc: None,
    //         reason: None,
    //         r#type: LeaveType::Casual,
    //         user_id: Some(user_id),
    //         leave_data: vec![
    //             CreateLeaveData {
    //                 period: LeavePeriod::HalfDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //             CreateLeaveData {
    //                 period: LeavePeriod::HalfDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //         ],
    //     };
    //     let mut transaction = pool
    //         .begin()
    //         .await
    //         .context("Failed to acquire a Postgres connection from the pool")
    //         .unwrap();
    //     let res = save_leave_request(
    //         &mut transaction,
    //         &leave_request,
    //         user_id,
    //         Uuid::new_v4(),
    //         "abc@gmail.com",
    //     )
    //     .await;
    //     transaction
    //         .commit()
    //         .await
    //         .context("Failed to commit SQL transaction to store a new user account.")
    //         .unwrap();
    //     assert!(res.is_ok());
    //     let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
    //     let leaves = get_leaves(&pool, &query).await;
    //     assert!(leaves.is_ok());
    //     eprint!("aaaa{:?}", leaves);
    //     let leave_vec = leaves.unwrap();
    //     let leave_opt = leave_vec.first();
    //     eprint!("aaaa{:?}", leave_opt);
    //     assert!(leave_opt.is_some());

    //     let deleted = delete_leave(&pool, leave_opt.unwrap().id, user_id).await;
    //     assert!(deleted.is_ok());
    //     let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
    //     let leaves = get_leaves(&pool, &query).await;
    //     assert!(leaves.is_ok());
    //     let leave_vec = leaves.unwrap();
    //     let leave_opt = leave_vec.first();
    //     assert!(leave_opt.is_none());

    //     let delete_res = hard_delete_user_account(
    //         &pool,
    //         &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
    //     )
    //     .await;
    //     assert!(delete_res.is_ok());
    // }

    // #[tokio::test]
    // async fn test_leave_request_fetch() {
    //     let pool = get_test_pool().await;
    //     let email = "testuser21@example.com";
    //     let mobile_no = "1234567904";
    //     let user_res = setup_user(&pool, "testuser21", email, mobile_no, "testuser@123").await;
    //     assert!(user_res.is_ok());
    //     let user_id = user_res.unwrap();
    //     let leave_request = CreateLeaveRequest {
    //         to: EmailObject::new(email.to_string()),
    //         cc: None,
    //         reason: None,
    //         r#type: LeaveType::Casual,
    //         user_id: Some(user_id),
    //         leave_data: vec![
    //             CreateLeaveData {
    //                 period: LeavePeriod::HalfDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //             CreateLeaveData {
    //                 period: LeavePeriod::HalfDay,
    //                 date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
    //             },
    //         ],
    //     };
    //     let receiver_id = Uuid::new_v4();
    //     let mut transaction = pool
    //         .begin()
    //         .await
    //         .context("Failed to acquire a Postgres connection from the pool")
    //         .unwrap();
    //     let res = save_leave_request(
    //         &mut transaction,
    //         &leave_request,
    //         user_id,
    //         receiver_id,
    //         "abc@gmail.com",
    //     )
    //     .await;
    //     transaction
    //         .commit()
    //         .await
    //         .context("Failed to commit SQL transaction to store a new user account.")
    //         .unwrap();
    //     assert!(res.is_ok());
    //     let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
    //     let leave_with_sender = get_leaves(&pool, &query).await;
    //     assert!(leave_with_sender.is_ok());
    //     let leave_vec = leave_with_sender.unwrap();
    //     let leave_opt = leave_vec.first();
    //     assert!(leave_opt.is_some());
    //     let tz: Tz = DUMMY_TIMEZONE.parse().unwrap();
    //     let start_utc: DateTime<Utc> = Utc::now();
    //     let end_utc = start_utc - Duration::minutes(3);
    //     let start_in_tz = start_utc.with_timezone(&tz);
    //     let end_in_tz = end_utc.with_timezone(&tz);

    //     let start_naive = start_in_tz.naive_local();
    //     let end_naive = end_in_tz.naive_local();
    //     let query = FetchLeaveQuery::builder()
    //         .with_limit(Some(1))
    //         .with_offset(Some(0))
    //         .with_start_date(Some(&start_naive))
    //         .with_end_date(Some(&end_naive));
    //     let leave_with_start_and_end_date = get_leaves(&pool, &query).await;
    //     assert!(leave_with_start_and_end_date.is_ok());
    //     let leave_vec = leave_with_start_and_end_date.unwrap();
    //     let leave_opt = leave_vec.first();
    //     assert!(leave_opt.is_some());
    //     let query = FetchLeaveQuery::builder().with_period(Some(&LeavePeriod::HalfDay));
    //     let leave_with_type = get_leaves(&pool, &query).await;
    //     assert!(leave_with_type.is_ok());
    //     let leave_vec = leave_with_type.unwrap();
    //     let leave_opt = leave_vec.first();
    //     assert!(leave_opt.is_some());
    //     let query = FetchLeaveQuery::builder().with_leave_id(Some(leave_opt.unwrap().id));
    //     let leave_with_id = get_leaves(&pool, &query).await;
    //     assert!(leave_with_id.is_ok());
    //     let leave_vec = leave_with_id.unwrap();
    //     let leave_opt = leave_vec.first();
    //     assert!(leave_opt.is_some());
    //     let query = FetchLeaveQuery::builder().with_recevier_id(Some(receiver_id));
    //     let leave_with_receiver_id = get_leaves(&pool, &query).await;
    //     assert!(leave_with_receiver_id.is_ok());
    //     let leave_vec = leave_with_receiver_id.unwrap();
    //     let leave_opt = leave_vec.first();
    //     assert!(leave_opt.is_some());

    //     let delete_res = hard_delete_user_account(
    //         &pool,
    //         &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
    //     )
    //     .await;
    //     assert!(delete_res.is_ok());
    // }

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
        let leave_data = vec![
            LeaveTypeCreationData {
                id: None,
                label: "Casual Leave".to_string(),
            },
            LeaveTypeCreationData {
                id: None,
                label: "Restricted Leave".to_string(),
            },
        ];

        let res = save_leave_type(&pool, &leave_data, user_id, business_id).await;
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
        }];

        let res = save_leave_type(&pool, &update_leave_data, user_id, business_id).await;
        assert!(res.is_ok());
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
        let leave_group_data = LeaveGroupCreationRequest {
            id: None,
            label: "2025".to_string(),
            start_date,
            end_date,
        };
        let leave_type_data = vec![LeaveTypeCreationData {
            id: None,
            label: "Casual Leave".to_string(),
        }];

        let (save_group_res, save_type_res) = tokio::join!(
            save_leave_group(&pool, &leave_group_data, business_id, user_id),
            save_leave_type(&pool, &leave_type_data, user_id, business_id)
        );
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
        let res = save_user_leave(&pool, &user_leave_data, user_id, leave_group_id).await;
        eprint!("aaaa{:?}", res);
        assert!(res.is_ok());

        let user_leave_res = fetch_user_leaves(&pool, business_id, user_id, leave_group_id).await;
        assert!(user_leave_res.is_ok());
        let user_leave_opt = user_leave_res.unwrap();

        assert!(user_leave_opt.first().is_some());

        let del_res =
            delete_user_leave(&pool, business_id, user_leave_opt.first().unwrap().id).await;
        assert!(del_res.is_ok());
        let user_leave_res = fetch_user_leaves(&pool, business_id, user_id, leave_group_id).await;
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

        let start_date = Utc::now();
        let end_date = start_date + Duration::days(2);
        let leave_group_data = LeaveGroupCreationRequest {
            id: None,
            label: "2025".to_string(),
            start_date,
            end_date,
        };
        let leave_type_data = vec![LeaveTypeCreationData {
            id: None,
            label: "Casual Leave".to_string(),
        }];

        let (save_group_res, save_type_res) = tokio::join!(
            save_leave_group(&pool, &leave_group_data, business_id, user_id),
            save_leave_type(&pool, &leave_type_data, user_id, business_id)
        );
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
        let res = save_user_leave(&pool, &user_leave_data, user_id, leave_group_id).await;

        assert!(res.is_ok());

        let user_leave_res = fetch_user_leaves(&pool, business_id, user_id, leave_group_id).await;
        assert!(user_leave_res.is_ok());
        let user_leave_opt = user_leave_res.unwrap();

        assert!(user_leave_opt.first().is_some());
        let user_leave = user_leave_opt.first().unwrap();

        let leave_data = vec![
            CreateLeaveData {
                period: LeavePeriod::FullDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
            CreateLeaveData {
                period: LeavePeriod::FullDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
        ];
        let mut leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            user_id: Some(user_id),
            leave_data,
            type_id: leave_type_id,
            group_id: leave_group_id,
            send_mail: false,
        };

        let leave_request_validation =
            validate_leave_request_creation(&pool, &leave_request, user_leave).await;
        assert!(leave_request_validation.is_err());

        let leave_data = vec![
            CreateLeaveData {
                period: LeavePeriod::HalfDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
            CreateLeaveData {
                period: LeavePeriod::FullDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
        ];
        leave_request.leave_data = leave_data;
        let leave_request_validation =
            validate_leave_request_creation(&pool, &leave_request, &user_leave).await;
        assert!(leave_request_validation.is_err());

        let leave_data = vec![
            CreateLeaveData {
                period: LeavePeriod::HalfDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
            CreateLeaveData {
                period: LeavePeriod::HalfDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
        ];
        leave_request.leave_data = leave_data;
        let leave_request_validation =
            validate_leave_request_creation(&pool, &leave_request, &user_leave).await;

        assert!(leave_request_validation.is_ok());

        let leave_data = vec![
            CreateLeaveData {
                period: LeavePeriod::FullDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
            CreateLeaveData {
                period: LeavePeriod::FullDay,
                date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
            },
        ];
        leave_request.leave_data = leave_data;
        let leave_request_validation =
            validate_leave_request_creation(&pool, &leave_request, &user_leave).await;
        assert!(leave_request_validation.is_err());

        let delete_mobile = format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no);
        let (delete_business_account_res, delete_user_account_res) = tokio::join!(
            hard_delete_business_account(&pool, business_id),
            hard_delete_user_account(&pool, &delete_mobile)
        );
        assert!(delete_business_account_res.is_ok());
        assert!(delete_user_account_res.is_ok());
    }
}
