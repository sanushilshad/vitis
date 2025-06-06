#[cfg(test)]
pub mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};
    use uuid::Uuid;

    use crate::{
        constants::INTERNATIONAL_DIALING_CODE,
        email::EmailObject,
        routes::{
            leave::{
                schemas::{
                    CreateLeaveData, CreateLeaveRequest, LeavePeriod, LeaveStatus, LeaveType,
                },
                utils::{
                    get_leaves, save_leave_request, update_leave_status, validate_leave_request,
                    validate_leave_status_update,
                },
            },
            project::schemas::{AllowedPermission, PermissionType},
            user::{tests::tests::setup_user, utils::hard_delete_user_account},
        },
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
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
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
        eprint!("aaaa{:?}", res);
        assert!(res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
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
        eprint!("aaaa{:?}", res);
        assert!(res.is_ok());
        let leaves = get_leaves(&pool, None, None, Some(user_id), None).await;
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
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }
}
