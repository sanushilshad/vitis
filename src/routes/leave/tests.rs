#[cfg(test)]
pub mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};

    use crate::{
        constants::INTERNATIONAL_DIALING_CODE,
        email::EmailObject,
        routes::{
            leave::{
                schemas::{CreateLeaveData, CreateLeaveRequest, LeavePeriod, LeaveType},
                utils::{save_leave_request, validate_leave_request},
            },
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

        let res = save_leave_request(&pool, &leave_request, user_id, "abc@gmail.com").await;
        eprint!("aaaa{:?}", res);
        assert!(res.is_ok());
        let delete_res = hard_delete_user_account(
            &pool,
            &format!("{}{}", INTERNATIONAL_DIALING_CODE, mobile_no),
        )
        .await;
        assert!(delete_res.is_ok());
    }
}
