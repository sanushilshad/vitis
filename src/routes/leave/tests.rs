#[cfg(test)]
pub mod tests {
    use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
    use chrono_tz::Tz;
    use uuid::Uuid;

    use crate::{
        constants::{DUMMY_INTERNATIONAL_DIALING_CODE, DUMMY_TIMEZONE},
        email::EmailObject,
        routes::{
            leave::{
                schemas::{
                    CreateLeaveData, CreateLeaveRequest, FetchLeaveQuery, LeavePeriod, LeaveStatus,
                    LeaveType,
                },
                utils::{
                    delete_leave, get_leaves, save_leave_request, update_leave_status,
                    validate_leave_request, validate_leave_status_update,
                },
            },
            user::{tests::tests::setup_user, utils::hard_delete_user_account},
        },
        schemas::{AllowedPermission, PermissionType},
        tests::tests::get_test_pool,
    };

    #[tokio::test]
    async fn test_leave_request_creation_validation() {
        let pool = get_test_pool().await;
        let email = "testuser17@example.com";
        let mobile_no = "1234567900";
        let user_res = setup_user(&pool, "testuser17", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
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
            r#type: LeaveType::Casual,
            user_id: Some(user_id),
            leave_data,
        };

        let financial_year_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let leave_request_validation =
            validate_leave_request(&pool, financial_year_start, &leave_request, user_id, 9).await;
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
            validate_leave_request(&pool, financial_year_start, &leave_request, user_id, 9).await;
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
            validate_leave_request(&pool, financial_year_start, &leave_request, user_id, 9).await;
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
            validate_leave_request(&pool, financial_year_start, &leave_request, user_id, 1).await;
        assert!(leave_request_validation.is_err());

        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_save_leave_request() {
        let pool = get_test_pool().await;
        let email = "testuser18@example.com";
        let mobile_no = "1234567901";
        let user_res = setup_user(&pool, "testuser18", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();

        let leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            r#type: LeaveType::Casual,
            user_id: Some(user_id),
            leave_data: vec![
                CreateLeaveData {
                    period: LeavePeriod::HalfDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
                CreateLeaveData {
                    period: LeavePeriod::FullDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
            ],
        };

        let res = save_leave_request(
            &pool,
            &leave_request,
            user_id,
            Uuid::new_v4(),
            "abc@gmail.com",
        )
        .await;
        assert!(res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_leave_request_status_validation() {
        let val_res = validate_leave_status_update(
            &LeaveStatus::Approved,
            &LeaveStatus::Rejected,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
        );
        assert!(val_res.is_err());

        let val_res = validate_leave_status_update(
            &LeaveStatus::Approved,
            &LeaveStatus::Cancelled,
            &AllowedPermission {
                permission_list: vec![PermissionType::ApproveLeaveRequest.to_string()],
            },
        );
        assert!(val_res.is_err());

        let val_res = validate_leave_status_update(
            &LeaveStatus::Approved,
            &LeaveStatus::Approved,
            &AllowedPermission {
                permission_list: vec![],
            },
        );
        assert!(val_res.is_err());

        let val_res = validate_leave_status_update(
            &LeaveStatus::Rejected,
            &LeaveStatus::Approved,
            &AllowedPermission {
                permission_list: vec![],
            },
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
        let leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            r#type: LeaveType::Casual,
            user_id: Some(user_id),
            leave_data: vec![
                CreateLeaveData {
                    period: LeavePeriod::HalfDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
                CreateLeaveData {
                    period: LeavePeriod::FullDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
            ],
        };

        let res = save_leave_request(
            &pool,
            &leave_request,
            user_id,
            Uuid::new_v4(),
            "abc@gmail.com",
        )
        .await;
        assert!(res.is_ok());
        let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
        let leaves = get_leaves(&pool, &query).await;
        eprint!("aaaa{:?}", leaves);
        assert!(leaves.is_ok());
        let leave_vec = leaves.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_some());
        let leave = leave_opt.unwrap();
        let res = update_leave_status(&pool, leave.id, &LeaveStatus::Approved, user_id).await;
        assert!(res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_leave_request_deletion() {
        let pool = get_test_pool().await;
        let email = "testuser22@example.com";
        let mobile_no = "1234567903";
        let user_res = setup_user(&pool, "testuser22", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            r#type: LeaveType::Casual,
            user_id: Some(user_id),
            leave_data: vec![
                CreateLeaveData {
                    period: LeavePeriod::HalfDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
                CreateLeaveData {
                    period: LeavePeriod::HalfDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
            ],
        };

        let res = save_leave_request(
            &pool,
            &leave_request,
            user_id,
            Uuid::new_v4(),
            "abc@gmail.com",
        )
        .await;
        assert!(res.is_ok());
        let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
        let leaves = get_leaves(&pool, &query).await;
        assert!(leaves.is_ok());
        eprint!("aaaa{:?}", leaves);
        let leave_vec = leaves.unwrap();
        let leave_opt = leave_vec.first();
        eprint!("aaaa{:?}", leave_opt);
        assert!(leave_opt.is_some());

        let deleted = delete_leave(&pool, leave_opt.unwrap().id, user_id).await;
        assert!(deleted.is_ok());
        let query = FetchLeaveQuery::builder().with_sender_id(Some(user_id));
        let leaves = get_leaves(&pool, &query).await;
        assert!(leaves.is_ok());
        let leave_vec = leaves.unwrap();
        let leave_opt = leave_vec.first();
        assert!(leave_opt.is_none());

        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_leave_request_fetch() {
        let pool = get_test_pool().await;
        let email = "testuser21@example.com";
        let mobile_no = "1234567904";
        let user_res = setup_user(&pool, "testuser21", email, mobile_no, "testuser@123").await;
        assert!(user_res.is_ok());
        let user_id = user_res.unwrap();
        let leave_request = CreateLeaveRequest {
            to: EmailObject::new(email.to_string()),
            cc: None,
            reason: None,
            r#type: LeaveType::Casual,
            user_id: Some(user_id),
            leave_data: vec![
                CreateLeaveData {
                    period: LeavePeriod::HalfDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
                CreateLeaveData {
                    period: LeavePeriod::HalfDay,
                    date: NaiveDate::from_ymd_opt(2025, 7, 3).expect("invalid date"),
                },
            ],
        };
        let receiver_id = Uuid::new_v4();
        let res =
            save_leave_request(&pool, &leave_request, user_id, receiver_id, "abc@gmail.com").await;
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
        let query = FetchLeaveQuery::builder().with_period(Some(&LeavePeriod::HalfDay));
        let leave_with_type = get_leaves(&pool, &query).await;
        assert!(leave_with_type.is_ok());
        let leave_vec = leave_with_type.unwrap();
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

        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", DUMMY_INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }
}
