use std::collections::HashMap;

use anyhow::anyhow;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use sqlx::{Execute, PgPool, QueryBuilder};
use uuid::Uuid;

use crate::{
    errors::GenericError,
    routes::{
        leave::schemas::{LeavePeriod, LeaveType},
        project::schemas::{AllowedPermission, PermissionType},
    },
};

use super::{
    models::LeaveDataModel,
    schemas::{
        BulkLeaveRequestInsert, CreateLeaveRequest, FetchLeaveQuery, LeaveData, LeaveStatus,
    },
};
use serde_json::Value;
#[tracing::instrument(name = "prepare bulk leave request data", skip(created_by))]
pub async fn prepare_bulk_leave_request_data<'a>(
    leave_request_data: &'a CreateLeaveRequest,
    created_by: Uuid,
    received_by: Uuid,
    email_message_id: &'a str,
) -> Result<Option<BulkLeaveRequestInsert<'a>>, anyhow::Error> {
    let current_utc = Utc::now();
    let mut created_by_list = vec![];
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut sender_id_list = vec![];
    let mut leave_type_list = vec![];
    let mut leave_period_list = vec![];
    let mut date_list = vec![];
    let mut status_list = vec![];
    let mut reason_list = vec![];
    let mut email_message_id_list = vec![];
    let mut cc_list = vec![];
    let mut receiver_id_list = vec![];
    if leave_request_data.leave_data.is_empty() {
        return Ok(None);
    }
    for leave_request in leave_request_data.leave_data.iter() {
        created_on_list.push(current_utc);
        created_by_list.push(created_by);
        id_list.push(Uuid::new_v4());
        sender_id_list.push(leave_request_data.user_id.unwrap_or(created_by));
        leave_type_list.push(&leave_request_data.r#type);
        leave_period_list.push(&leave_request.period);

        date_list.push(Utc.from_utc_datetime(&leave_request.date.and_hms_opt(0, 0, 0).unwrap()));
        status_list.push(LeaveStatus::Requested); // Assuming default status is Requested
        email_message_id_list.push(Some(email_message_id));
        reason_list.push(leave_request_data.reason.as_deref());
        cc_list.push(
            leave_request_data
                .cc
                .as_ref()
                .map(|cc| serde_json::to_value(cc).unwrap()),
        );
        receiver_id_list.push(received_by);
    }
    Ok(Some(BulkLeaveRequestInsert {
        id: id_list,
        sender_id: sender_id_list,
        receiver_id: receiver_id_list,
        created_on: created_on_list,
        created_by: created_by_list,
        leave_type: leave_type_list,
        leave_period: leave_period_list,
        date: date_list,
        status: status_list,
        reason: reason_list,
        email_message_id: email_message_id_list,
        cc: cc_list,
    }))
}

// test case not needed
#[tracing::instrument(name = "save leave request", skip(pool, data))]
pub async fn save_leave_to_database<'a>(
    pool: &PgPool,
    data: BulkLeaveRequestInsert<'a>,
) -> Result<bool, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO leave (id, sender_id, created_by, created_on, period,type, date, status, reason, email_message_id, cc, receiver_id)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::TIMESTAMP[],  $5::leave_period[], $6::leave_type[], $7::TIMESTAMP[], $8::leave_status[],
        $9::TEXT[], $10::TEXT[], $11::jsonb[], $12::uuid[]) ON CONFLICT DO NOTHING
        "#,
        &data.id[..] as &[Uuid],
        &data.sender_id[..] as &[Uuid],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
        &data.leave_period[..] as &[&LeavePeriod],
        &data.leave_type[..] as &[&LeaveType],
        &data.date[..] as &[DateTime<Utc>],
        &data.status[..] as &[LeaveStatus],
        &data.reason[..] as &[Option<&str>],
        &data.email_message_id[..] as &[Option<&str>],
        &data.cc[..] as &[Option<Value>],
        &data.receiver_id[..] as &[Uuid],
    );
    let query_string = query.sql();
    println!("Generated SQL query for: {}", query_string);
    let result = query.execute(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving leave request")
    })?;

    Ok(result.rows_affected() > 0)
}

pub async fn save_leave_request(
    pool: &PgPool,
    leave_request_data: &CreateLeaveRequest,
    created_by: Uuid,
    received_by: Uuid,
    email_message_id: &str,
) -> Result<bool, anyhow::Error> {
    let bulk_data = prepare_bulk_leave_request_data(
        leave_request_data,
        created_by,
        received_by,
        email_message_id,
    )
    .await?;
    if let Some(data) = bulk_data {
        return save_leave_to_database(pool, data).await;
    }
    Ok(false)
}

pub async fn fetch_leave_models<'a>(
    pool: &PgPool,
    query: &'a FetchLeaveQuery<'a>,
) -> Result<Vec<LeaveDataModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT id, sender_id, created_on, type, period, date, status, email_message_id, cc, reason FROM leave WHERE is_deleted=false "#,
    );
    if let Some(user_id) = query.sender_id {
        query_builder.push(" AND sender_id =");
        query_builder.push_bind(user_id);
    }
    if let Some(receiver_id) = query.receiver_id {
        query_builder.push(" AND receiver_id =");
        query_builder.push_bind(receiver_id);
    }
    if let Some(date) = query.date {
        query_builder.push(" AND date =");
        query_builder.push_bind(date);
    }
    if let Some(period) = query.period {
        query_builder.push(" AND period =");
        query_builder.push_bind(period);
    }

    if let Some(leave_id) = query.leave_id {
        query_builder.push(" AND id =");
        query_builder.push_bind(leave_id);
    }

    if let Some(start) = query.start_date {
        if let Some(tz) = query.tz {
            // Interpret NaiveDateTime as a datetime in timezone tz
            let localized = tz
                .from_local_datetime(start)
                .single()
                .ok_or_else(|| anyhow!("Ambiguous or invalid local datetime for start_date"))?;

            // Convert to UTC
            let utc_start = localized.with_timezone(&Utc);

            query_builder.push(" AND created_on >= ");
            query_builder.push_bind(utc_start);
        };
    }

    if let Some(end) = query.end_date {
        if let Some(tz) = query.tz {
            let localized = tz
                .from_local_datetime(end)
                .single()
                .ok_or_else(|| anyhow!("Ambiguous or invalid local datetime for end_date"))?;
            let utc_end = localized.with_timezone(&Utc);
            query_builder.push(" AND created_on <= ");
            query_builder.push_bind(utc_end);
        };
    }
    if let Some(limit) = query.limit {
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
    }

    if let Some(offset) = query.offset {
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);
    }

    let query = query_builder.build_query_as::<LeaveDataModel>();
    println!("Generated SQL query for: {}", query.sql());
    let leaves = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while fetching leave")
    })?;

    Ok(leaves)
}

pub async fn get_leaves<'a>(
    pool: &PgPool,
    query: &'a FetchLeaveQuery<'a>,
) -> Result<Vec<LeaveData>, anyhow::Error> {
    let models = fetch_leave_models(pool, query).await?;

    let data: Vec<LeaveData> = models
        .into_iter()
        .map(|a| a.into_schema(query.tz))
        .collect();
    Ok(data)
}

// pub async fn _leave_exists(
//     pool: &PgPool,
//     date: DateTime<Utc>,
//     period: LeavePeriod,
//     user_id: Uuid,
// ) -> Result<bool, anyhow::Error> {
//     let exists = sqlx::query_scalar!(
//         r#"
//         SELECT EXISTS (
//             SELECT 1 FROM leave
//             WHERE sender_id = $1
//               AND date = $2
//               AND period = $3
//         )
//         "#,
//         user_id,
//         date,
//         period as LeavePeriod // Explicit cast if needed
//     )
//     .fetch_one(pool)
//     .await?;

//     Ok(exists.unwrap_or(false)) // just in case DB returns NULL (unlikely here)
// }

pub async fn get_leave_count(
    pool: &PgPool,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    r#type: &LeaveType,
    user_id: Uuid,
) -> Result<BigDecimal, anyhow::Error> {
    let count: Option<BigDecimal> = sqlx::query_scalar!(
        r#"
        SELECT 
            SUM(
                CASE period
                    WHEN 'half_day' THEN 0.5
                    WHEN 'full_day' THEN 1.0
                    ELSE 0.0
                END
            ) as count
        FROM leave
        WHERE sender_id = $1
          AND date >= $2
          AND date <= $3
          AND type = $4
          AND status != $5 AND status != $6
          AND is_deleted = false 
          AND status !='rejected'
          AND status !='cancelled'
        "#,
        user_id,
        start_date,
        end_date,
        r#type as &LeaveType,
        &LeaveStatus::Rejected as &LeaveStatus,
        &LeaveStatus::Cancelled as &LeaveStatus,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while fetching leave count")
    })?;

    Ok(count.unwrap_or_default())
}

pub async fn validate_leave_request(
    pool: &PgPool,
    financial_year_start: DateTime<Utc>,
    body: &CreateLeaveRequest,
    user_id: Uuid,
    allowed_leave_count: i32,
) -> Result<(), anyhow::Error> {
    // let mut unique_entries = HashSet::new();
    let mut leave_map: HashMap<NaiveDate, Vec<&LeavePeriod>> = HashMap::new();

    for entry in body.leave_data.iter() {
        leave_map.entry(entry.date).or_default().push(&entry.period);
    }

    for (date, periods) in leave_map {
        let full_days = periods
            .iter()
            .filter(|&&p| p == &LeavePeriod::FullDay)
            .count();
        let half_days = periods
            .iter()
            .filter(|&&p| p == &LeavePeriod::HalfDay)
            .count();

        if full_days > 1 {
            return Err(anyhow!("More than one FullDay leave applied for {}", date));
        }

        if full_days > 0 && half_days > 0 {
            return Err(anyhow!(
                "Cannot mix FullDay and HalfDay leaves on the same date: {}",
                date
            ));
        }

        if half_days > 2 {
            return Err(anyhow!("More than two HalfDay leaves applied for {}", date));
        }
    }

    let end_date = financial_year_start
        .with_year(financial_year_start.year() + 1)
        .unwrap()
        - Duration::seconds(1);
    let current_count =
        get_leave_count(pool, financial_year_start, end_date, &body.r#type, user_id)
            .await
            .unwrap_or(BigDecimal::default());
    let new_leave_count = body
        .leave_data
        .iter()
        .fold(BigDecimal::from(0), |acc, item| {
            acc + match item.period {
                LeavePeriod::FullDay => BigDecimal::from_f64(1.0).unwrap(),
                LeavePeriod::HalfDay => BigDecimal::from_f64(0.5).unwrap(),
            }
        });

    if current_count + &new_leave_count > BigDecimal::from(&allowed_leave_count) {
        return Err(anyhow!(
            "You have exceeded the allowed leave count of {} for the current financial year.",
            allowed_leave_count
        ));
    } else if new_leave_count > allowed_leave_count.into() {
        return Err(anyhow!(
            "You are applying for more than {} leaves.",
            allowed_leave_count
        ));
    }
    Ok(())
}

#[tracing::instrument(name = "reactivate user account", skip(pool))]
pub async fn update_leave_status(
    pool: &PgPool,
    id: Uuid,
    status: &LeaveStatus,
    updated_by: Uuid,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE leave 
        SET
        status = $1,
        updated_on = $2,
        updated_by = $3
        WHERE id = $4
        "#,
        status as &LeaveStatus,
        Utc::now(),
        updated_by,
        id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while updating leave status")
    })?;
    Ok(())
}

pub fn validate_leave_status_update(
    incoming_status: &LeaveStatus,
    current_status: &LeaveStatus,

    permissions: &AllowedPermission,
) -> Result<(), GenericError> {
    let has_approval_permission = permissions
        .permission_list
        .contains(&PermissionType::ApproveLeaveRequest.to_string());

    match (incoming_status, current_status, has_approval_permission) {
        (_, LeaveStatus::Rejected, _) => Err(GenericError::ValidationError(
            "Leave request is already rejected.".to_string(),
        )),
        (_, LeaveStatus::Cancelled, _) => Err(GenericError::ValidationError(
            "Leave request is already cancelled.".to_string(),
        )),
        (LeaveStatus::Approved, LeaveStatus::Approved, false) => {
            Err(GenericError::InsufficientPrevilegeError(
                "Leave request is already approved.".to_string(),
            ))
        }
        (LeaveStatus::Approved, _, false) | (LeaveStatus::Rejected, _, false) => {
            Err(GenericError::InsufficientPrevilegeError(
                "You don't have sufficient privilege for this operation.".to_string(),
            ))
        }
        _ => Ok(()),
    }
}

// pub async fn send_personal_html() -> Result<(), anyhow::Error> {
//     Ok(())
// }

#[tracing::instrument(name = "reactivate user account", skip(pool))]
pub async fn delete_leave(
    pool: &PgPool,
    leave_id: Uuid,
    deleted_by: Uuid,
) -> Result<(), anyhow::Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE leave 
        SET is_deleted = true,
        deleted_on = $2,
        deleted_by = $3
        WHERE id = $1
        "#,
        leave_id,
        Utc::now(),
        deleted_by
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while deleting leave")
    })?;
    Ok(())
}
