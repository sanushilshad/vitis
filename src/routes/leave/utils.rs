use std::collections::{HashMap, HashSet};

use anyhow::{Context, anyhow};
use bigdecimal::BigDecimal;
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Execute, Executor, PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

use crate::{
    errors::GenericError,
    routes::{
        leave::models::{
            LeaveGroupModel, LeavePeriodWithTypeModel, LeaveTypeModel, UserLeaveModel,
        },
        user::{schemas::MinimalUserAccount, utils::get_minimal_user_list},
    },
    schemas::{AlertStatus, AllowedPermission, PermissionType},
    slack_client::{SlackBlockType, SlackClient, SlackNotificationPayload, SlackTextType},
    utils::snake_to_title_case,
};

use super::{
    models::{LeaveDataModel, LeavePeriodModel, MinimalLeaveModel},
    schemas::{
        BulkLeavePeriodInsert, BulkLeaveRequestInsert, BulkLeaveTypeInsert,
        BulkLeaveTypePeriodInsert, BulkUserLeaveInsert, CreateLeaveRequest, FetchLeaveQuery,
        LeaveGroup, LeaveGroupCreationRequest, LeavePeriodCreationData, LeavePeriodData,
        LeaveRequestData, LeaveStatus, LeaveTypeCreationData, LeaveTypeCreationRequest,
        LeaveTypeData, UserLeave, UserLeaveCreationData,
    },
};
use serde_json::Value;
#[tracing::instrument(name = "prepare bulk leave request data", skip(created_by))]
pub async fn prepare_bulk_leave_request_data<'a>(
    leave_request_data: &'a CreateLeaveRequest,
    user_leave_id: Uuid,
    created_by: Uuid,
    received_by: Uuid,
    email_message_id: Option<&'a str>,
) -> Result<Option<BulkLeaveRequestInsert<'a>>, anyhow::Error> {
    let current_utc = Utc::now();
    let mut created_by_list = vec![];
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut sender_id_list = vec![];
    let mut user_leave_id_list = vec![];
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
        user_leave_id_list.push(user_leave_id);
        leave_period_list.push(&leave_request.period_id);

        date_list.push(Utc.from_utc_datetime(&leave_request.date.and_hms_opt(0, 0, 0).unwrap()));
        status_list.push(LeaveStatus::Requested); // Assuming default status is Requested
        email_message_id_list.push(email_message_id);
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
        // sender_id: sender_id_list,
        receiver_id: receiver_id_list,
        created_on: created_on_list,
        created_by: created_by_list,
        date: date_list,
        status: status_list,
        reason: reason_list,
        email_message_id: email_message_id_list,
        cc: cc_list,
        user_leave_id: user_leave_id_list,
        leave_period_id: leave_period_list,
    }))
}

#[tracing::instrument(name = "save leave request to database", skip(transaction, data))]
pub async fn save_leave_to_database<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    data: BulkLeaveRequestInsert<'a>,
) -> Result<bool, anyhow::Error> {
    let query = sqlx::query!(
        r#"
    INSERT INTO leave_request (id, created_by, created_on, leave_period_id, date, status, reason, email_message_id, cc, receiver_id, user_leave_id)
    SELECT * FROM UNNEST(
        $1::uuid[], 
        $2::uuid[], 
        $3::timestamptz[], 
        $4::uuid[], 
        $5::timestamptz[], 
        $6::leave_status[], 
        $7::text[], 
        $8::text[], 
        $9::jsonb[], 
        $10::uuid[],
        $11::uuid[]
    ) ON CONFLICT DO NOTHING
    "#,
        &data.id[..] as &[Uuid],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
        &data.leave_period_id[..] as &[&Uuid],
        &data.date[..] as &[DateTime<Utc>],
        &data.status[..] as &[LeaveStatus],
        &data.reason[..] as &[Option<&str>],
        &data.email_message_id[..] as &[Option<&str>],
        &data.cc[..] as &[Option<Value>],
        &data.receiver_id[..] as &[Uuid],
        &data.user_leave_id[..] as &[Uuid],
    );
    let query_string = query.sql();
    println!("Generated SQL query for: {}", query_string);
    let result = transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving leave request")
    })?;

    Ok(result.rows_affected() > 0)
}

#[tracing::instrument(name = "save leave request", skip(transaction))]
pub async fn save_leave_request(
    transaction: &mut Transaction<'_, Postgres>,
    leave_request_data: &CreateLeaveRequest,
    user_leave_id: Uuid,
    created_by: Uuid,
    received_by: Uuid,
    email_message_id: Option<&str>,
) -> Result<bool, anyhow::Error> {
    let bulk_data = prepare_bulk_leave_request_data(
        leave_request_data,
        user_leave_id,
        created_by,
        received_by,
        email_message_id,
    )
    .await?;
    if let Some(data) = bulk_data {
        return save_leave_to_database(transaction, data).await;
    }
    Ok(false)
}
#[tracing::instrument(name = "Fetch leave models", skip(pool))]
pub async fn fetch_leave_models<'a>(
    pool: &PgPool,
    query: &'a FetchLeaveQuery<'a>,
) -> Result<Vec<LeaveDataModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT 
            l_r.id, 
            l_r.leave_period_id, 
            l_r.date, 
            l_r.reason, 
            l_r.status,  
            l_r.email_message_id, 
            l_r.cc, 
            l_r.created_on,
            l_r.user_leave_id,
            lt.label AS leave_type,
            ulr.user_id,
            lp.id AS period_id,
            lp.label AS period_label,
            lp.value AS period_value
        FROM 
            leave_request AS l_r
        LEFT JOIN 
            user_leave_relationship AS ulr 
            ON l_r.user_leave_id = ulr.id
        LEFT JOIN 
            leave_type AS lt 
            ON ulr.leave_type_id = lt.id
        LEFT JOIN 
            leave_period AS lp 
            ON l_r.leave_period_id = lp.id
        WHERE 
            l_r.is_deleted = false"#,
    );
    if let Some(user_id) = query.sender_id {
        query_builder.push(" AND ulr.user_id =");
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
    // if let Some(period) = query.period {
    //     query_builder.push(" AND period =");
    //     query_builder.push_bind(period);
    // }

    if let Some(leave_id) = query.leave_id {
        query_builder.push(" AND l_r.id =");
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

            query_builder.push(" AND l_r.created_on >= ");
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
            query_builder.push(" AND l_r.created_on <= ");
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
) -> Result<Vec<LeaveRequestData>, anyhow::Error> {
    let models = fetch_leave_models(pool, query).await?;
    let final_data = models
        .into_iter()
        .map(|a| a.into_schema(query.tz))
        .collect();

    // let period_models =
    //     fetch_leave_periods_by_association(&pool, None, Some(period_id_list)).await?;
    // let period_map: HashMap<Uuid, LeavePeriodWithTypeModel> =
    //     period_models.into_iter().map(|p| (p.id, p)).collect();
    // let mut final_data = vec![];
    // for model in models {
    //     if let Some(period) = period_map.get(&model.leave_period_id) {
    //         let period_schema = period.clone().into_schema(); // clone the individual period
    //         final_data.push(model.into_schema(query.tz, period_schema));
    //     }
    // }
    Ok(final_data)
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

// pub async fn get_leave_count(
//     pool: &PgPool,
//     start_date: DateTime<Utc>,
//     end_date: DateTime<Utc>,
//     r#type: &LeaveType,
//     user_id: Uuid,
// ) -> Result<BigDecimal, anyhow::Error> {
//     let count: Option<BigDecimal> = sqlx::query_scalar!(
//         r#"
//         SELECT
//             SUM(
//                 CASE period
//                     WHEN 'half_day' THEN 0.5
//                     WHEN 'full_day' THEN 1.0
//                     ELSE 0.0
//                 END
//             ) as count
//         FROM leave_request
//         WHERE sender_id = $1
//           AND date >= $2
//           AND date <= $3
//           AND type = $4
//           AND status != $5 AND status != $6
//           AND is_deleted = false
//           AND status !='rejected'
//           AND status !='cancelled'
//         "#,
//         user_id,
//         start_date,
//         end_date,
//         r#type as &LeaveType,
//         &LeaveStatus::Rejected as &LeaveStatus,
//         &LeaveStatus::Cancelled as &LeaveStatus,
//     )
//     .fetch_one(pool)
//     .await
//     .map_err(|e| {
//         tracing::error!("Failed to execute query: {:?}", e);
//         anyhow!(e).context("A database failure occurred while fetching leave count")
//     })?;

//     Ok(count.unwrap_or_default())
// }

pub fn validate_leave_request_creation(
    body: &CreateLeaveRequest,
    user_leave: &UserLeave,
) -> Result<(), anyhow::Error> {
    let period_map: HashMap<Uuid, &LeavePeriodData> =
        user_leave.periods.iter().map(|p| (p.id, p)).collect();

    // Sum the requested leave values
    let mut new_leave_count = BigDecimal::from(0);

    for item in &body.leave_data {
        match period_map.get(&item.period_id) {
            Some(period_data) => {
                new_leave_count += &period_data.value;
            }
            None => {
                return Err(anyhow!("Invalid leave period with id: {}", item.period_id));
            }
        }
    }

    if (&user_leave.used_count + &new_leave_count) > user_leave.allocated_count {
        return Err(anyhow!(
            "You have exceeded the allowed leave count of {} for the group.",
            user_leave.allocated_count
        ));
    } else if new_leave_count > user_leave.allocated_count {
        return Err(anyhow!(
            "You are applying for more than {} leaves.",
            user_leave.allocated_count
        ));
    }
    Ok(())
}

#[tracing::instrument(name = "reactivate user account", skip(transaction))]
pub async fn update_leave_request_status(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    status: &LeaveStatus,
    updated_by: Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE leave_request
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
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while updating leave status")
    })?;
    Ok(())
}

pub fn validate_leave_status_update(
    incoming_status: &LeaveStatus,
    current_status: &LeaveStatus,
    permissions: &AllowedPermission,
    user_leave: &UserLeave,
    leave_period: &LeavePeriodData,
) -> Result<(), GenericError> {
    let has_approval_permission = permissions
        .permission_list
        .iter()
        .any(|p| p == &PermissionType::ApproveLeaveRequest.to_string());

    // Status transition validation
    if !has_approval_permission {
        return Err(GenericError::InsufficientPrevilegeError(
            "You don't have sufficient privilege to update leave requests.".to_string(),
        ));
    }
    if current_status == &LeaveStatus::Rejected {
        return Err(GenericError::ValidationError(
            "Leave request is already rejected.".to_string(),
        ));
    }

    if current_status == &LeaveStatus::Cancelled {
        return Err(GenericError::ValidationError(
            "Leave request is already cancelled.".to_string(),
        ));
    }

    if incoming_status == &LeaveStatus::Approved && current_status == &LeaveStatus::Approved {
        return Err(GenericError::InsufficientPrevilegeError(
            "Leave request is already approved.".to_string(),
        ));
    }

    if incoming_status == &LeaveStatus::Cancelled && current_status != &LeaveStatus::Approved {
        return Err(GenericError::ValidationError(
            "Only approved leaves can be cancelled.".to_string(),
        ));
    }

    if incoming_status == &LeaveStatus::Rejected && current_status != &LeaveStatus::Approved {
        return Err(GenericError::InsufficientPrevilegeError(
            "You don't have sufficient privilege to approve leave requests.".to_string(),
        ));
    }

    if (&leave_period.value + &user_leave.used_count) > user_leave.allocated_count {
        return Err(GenericError::ValidationError(format!(
            "You have exceeded the allowed leave count of {} for the group.",
            user_leave.allocated_count
        )));
    }

    Ok(())
}

// pub async fn send_personal_html() -> Result<(), anyhow::Error>
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
        UPDATE leave_request
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

async fn get_approved_leaves_by_lock(
    transaction: &mut Transaction<'_, Postgres>,
    leave_date: DateTime<Utc>,
) -> Result<Vec<MinimalLeaveModel>, anyhow::Error> {
    let rows = sqlx::query_as!(
        MinimalLeaveModel,
        r#"
        SELECT 
            l_r.id, 
            lp.label as period, 
            ulr.user_id,
            lt.label as type
        FROM 
            leave_request AS l_r
        INNER JOIN 
            user_leave_relationship AS ulr 
            ON l_r.user_leave_id = ulr.id
        INNER JOIN 
            leave_type AS lt 
            ON ulr.leave_type_id = lt.id
        INNER JOIN 
            leave_type_period_relationship AS ltpr
            ON ltpr.leave_type_id = lt.id
        INNER JOIN 
            leave_period AS lp
            ON lp.id = ltpr.leave_period_id
        WHERE 
            l_r.is_deleted = false
            AND l_r.date = $1
        FOR UPDATE
        "#,
        leave_date,
    )
    .fetch_all(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching leave with lock")
    })?;

    Ok(rows)
}

#[tracing::instrument(name = "update leave alert status", skip(transaction))]
pub async fn update_leave_alert_status(
    id_list: &Vec<Uuid>,
    transaction: &mut Transaction<'_, Postgres>,
    alert_status: &AlertStatus,
) -> Result<(), anyhow::Error> {
    // let val_list: Vec<String> = id_list.iter().map(|&s| s.to_string()).collect();
    let query = sqlx::query!(
        r#"
        UPDATE leave_request
        SET
        alert_status = $1
        Where id = ANY($2)
        "#,
        alert_status as &AlertStatus,
        id_list
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while updating leave alert status")
    })?;
    Ok(())
}

pub async fn send_slack_notification_for_approved_leave(
    pool: &PgPool,
    slack_client: &SlackClient,
    leave_date: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let leave_data = get_approved_leaves_by_lock(&mut transaction, leave_date).await?;
    if !leave_data.is_empty() {
        let mut leave_id_list = Vec::new();
        let mut sender_id_list = Vec::new();
        let mut grouped: HashMap<(&str, &str), Vec<&MinimalLeaveModel>> = HashMap::new();
        for leave in &leave_data {
            leave_id_list.push(leave.id);
            sender_id_list.push(leave.user_id);
            grouped
                .entry((&leave.r#type, &leave.period))
                .or_default()
                .push(leave);
        }

        let users = get_minimal_user_list(pool, None, 1000, 0, Some(&sender_id_list)).await?;
        let user_map: HashMap<Uuid, MinimalUserAccount> =
            users.into_iter().map(|x| (x.id, x)).collect();

        let mut notification = SlackNotificationPayload::new("Leave Notification".to_string())
            .add_section(
                // format!("🗓️ *Leave* on {}", leave_date.format("%Y-%m-%d")),
                format!("🗓️ Leave on {}", leave_date.format("%Y-%m-%d")),
                SlackBlockType::Header,
                SlackTextType::PlainText,
            );
        for ((leave_type, period), leaves) in grouped {
            let mut user_string = String::new();

            for leave in &leaves {
                if let Some(user_obj) = user_map.get(&leave.user_id) {
                    user_string.push_str(&format!("{}, ", user_obj.display_name));
                }
            }
            if !user_string.is_empty() {
                user_string.truncate(user_string.len().saturating_sub(2));
            }

            let section_text = format!(
                "*{}* — {}: {}",
                snake_to_title_case(leave_type),
                snake_to_title_case(period),
                user_string
            );

            notification = notification.add_section(
                section_text,
                SlackBlockType::Section,
                SlackTextType::Mrkdwn,
            );
        }
        let result = slack_client
            .send_notification(notification.build(), &slack_client.channel.leave)
            .await;

        let status = if result.is_ok() {
            AlertStatus::Success
        } else {
            AlertStatus::Failed
        };

        update_leave_alert_status(&leave_id_list, &mut transaction, &status).await?;
    }

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new user account.")?;
    Ok(())
}

// pub fn create_leave

#[tracing::instrument(name = "prepare bulk leave type data", skip(created_by))]
pub async fn prepare_bulk_leave_type_data<'a>(
    leave_type_data: &'a Vec<LeaveTypeCreationData>,
    business_id: Uuid,
    created_by: Uuid,
) -> Option<BulkLeaveTypeInsert<'a>> {
    let current_utc = Utc::now();
    let mut label_list = vec![];
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut business_id_list = vec![];
    let mut created_by_list = vec![];
    if leave_type_data.is_empty() {
        return None;
    }
    for leave_data in leave_type_data.iter() {
        created_on_list.push(current_utc);
        created_by_list.push(created_by);
        if let Some(id) = leave_data.id {
            id_list.push(id);
        } else {
            id_list.push(Uuid::new_v4());
        }
        label_list.push(leave_data.label.as_ref());
        business_id_list.push(business_id);
    }
    Some(BulkLeaveTypeInsert {
        id: id_list,
        label: label_list,
        created_on: created_on_list,
        created_by: created_by_list,
        business_id: business_id_list,
    })
}

// test case not needed
#[tracing::instrument(name = "save leave type to database", skip(transaction, data))]
pub async fn save_leave_type_to_database<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    data: BulkLeaveTypeInsert<'a>,
) -> Result<HashMap<String, Uuid>, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO leave_type (id, created_by, created_on, label, business_id)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::TIMESTAMP[],  $4::TEXT[], $5::uuid[]) 
        ON CONFLICT (id) DO UPDATE
        SET label = EXCLUDED.label,
        updated_by = EXCLUDED.created_by,
        updated_on = EXCLUDED.created_on
        RETURNING id, label
        "#,
        &data.id[..] as &[Uuid],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
        &data.label[..] as &[&str],
        &data.business_id[..] as &[Uuid]
    );
    let query_string = query.sql();
    println!("Generated SQL query for: {}", query_string);
    let rows = query
        .fetch_all(&mut **transaction)
        .await
        .map_err(|e: sqlx::Error| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow!(e).context("A database failure occurred while saving leave type request")
        })?;

    let label_id_map = rows
        .into_iter()
        .map(|row| (row.label, row.id))
        .collect::<HashMap<String, Uuid>>();

    Ok(label_id_map)
}

#[tracing::instrument(name = "save leave type", skip(transaction))]
pub async fn save_leave_type(
    transaction: &mut Transaction<'_, Postgres>,
    leave_type_data: &Vec<LeaveTypeCreationData>,
    user_id: Uuid,
    created_by: Uuid,
    business_id: Uuid,
) -> Result<(), anyhow::Error> {
    let bulk_data = prepare_bulk_leave_type_data(leave_type_data, business_id, created_by).await;
    if let Some(data) = bulk_data {
        let map = save_leave_type_to_database(transaction, data).await?;
        save_leave_type_period_relationship(
            transaction,
            leave_type_data,
            user_id,
            business_id,
            map,
        )
        .await?;
    }
    Ok(())
}

#[tracing::instrument(name = "Fetch leave type models", skip(pool))]
async fn fetch_leave_type_models<'a>(
    pool: &PgPool,
    business_id: Uuid,
    id_list: Option<Vec<Uuid>>,
    label_list: Option<Vec<&str>>,
    query: Option<String>,
) -> Result<Vec<LeaveTypeModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT id, label FROM leave_type WHERE business_id="#,
    );
    query_builder.push_bind(business_id);
    if let Some(id_list) = id_list {
        if !id_list.is_empty() {
            query_builder.push(" AND id = ANY(");
            query_builder.push_bind(id_list);
            query_builder.push(")");
        }
    }
    if let Some(label_list) = label_list {
        if !label_list.is_empty() {
            query_builder.push(" AND label = ANY(");
            query_builder.push_bind(label_list);
            query_builder.push(")");
        }
    }
    if let Some(query) = query {
        let like_pattern = format!("%{}%", query);
        query_builder.push(" AND label ILIKE ");
        query_builder.push_bind(like_pattern);
    }

    let query = query_builder.build_query_as::<LeaveTypeModel>();
    println!("Generated SQL query for: {}", query.sql());
    let leave_type = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while fetching leave type")
    })?;

    Ok(leave_type)
}

pub async fn get_leave_type(
    pool: &PgPool,
    business_id: Uuid,
    id_list: Option<Vec<Uuid>>,
    label_list: Option<Vec<&str>>,
    query: Option<String>,
) -> Result<Vec<LeaveTypeData>, anyhow::Error> {
    let data_models =
        fetch_leave_type_models(pool, business_id, id_list, label_list, query).await?;
    let leave_type_id_list = data_models.iter().map(|a| a.id).collect();
    let period_models =
        fetch_leave_periods_by_association(pool, Some(leave_type_id_list), None).await?;
    let mut period_map: HashMap<Uuid, Vec<LeavePeriodData>> = HashMap::new();
    for period in period_models {
        period_map
            .entry(period.type_id)
            .or_default()
            .push(period.into_schema());
    }
    let mut final_data = Vec::with_capacity(data_models.len());

    for data_model in data_models {
        let periods = period_map.remove(&data_model.id).unwrap_or_default();
        final_data.push(data_model.into_schema(periods));
    }

    Ok(final_data)
}

#[tracing::instrument(name = "delete payment", skip(pool))]
pub async fn delete_leave_type(pool: &PgPool, id: Uuid) -> Result<(), anyhow::Error> {
    sqlx::query("DELETE FROM leave_type WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to delete leave type by ID")?;

    Ok(())
}
pub async fn leave_type_create_validation(
    pool: &PgPool,
    req: &LeaveTypeCreationRequest,
    business_id: Uuid,
) -> Result<(), GenericError> {
    if req.data.is_empty() {
        return Err(GenericError::ValidationError(
            "Incoming Data is empty".to_string(),
        ));
    }

    let mut label_list = vec![];
    let mut period_id_list: Vec<Uuid> = vec![];
    for item in &req.data {
        if item.id.is_none() {
            label_list.push(item.label.as_str());
        }
        if item.period_id_list.is_empty() {
            return Err(GenericError::ValidationError(format!(
                "Please select a period for type  {}",
                item.label
            )));
        }
        period_id_list.extend(&item.period_id_list);
    }
    if !label_list.is_empty() {
        let leave_types = get_leave_type(pool, business_id, None, Some(label_list), None)
            .await
            .map_err(|e| {
                GenericError::DatabaseError(
                    "Something went wrong while fetching leave type".to_string(),
                    e,
                )
            })?;
        if !leave_types.is_empty() {
            let invalid_keys_str = leave_types
                .iter()
                .map(|s| s.label.as_str())
                .collect::<Vec<&str>>()
                .join(", ");
            return Err(GenericError::ValidationError(format!(
                "Leave Type/s already exists: {}",
                invalid_keys_str
            )));
        }
    }

    let leave_period = get_leave_period(pool, business_id, Some(&period_id_list), None, None)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching leave period".to_string(),
                e,
            )
        })?;
    let existing_ids: HashSet<Uuid> = leave_period.iter().map(|p| p.id).collect();

    let missing_ids: Vec<&Uuid> = period_id_list
        .iter()
        .filter(|id| !existing_ids.contains(id))
        .collect();

    if !missing_ids.is_empty() {
        return Err(GenericError::ValidationError(format!(
            "Invalid period IDs: {:?}",
            missing_ids
        )));
    }
    Ok(())
}

#[tracing::instrument(name = "Fetch leave models", skip(pool))]
async fn fetch_leave_group_models<'a>(
    pool: &PgPool,
    business_id: Uuid,
    id_list: Option<&[uuid::Uuid]>,
    query: Option<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Vec<LeaveGroupModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT id, label, start_date, end_date FROM leave_group WHERE business_id="#,
    );
    query_builder.push_bind(business_id);
    if let Some(id_list) = id_list {
        if !id_list.is_empty() {
            query_builder.push(" AND id = ANY(");
            query_builder.push_bind(id_list);
            query_builder.push(")");
        }
    }
    if let Some(query) = query {
        let like_pattern = format!("%{}%", query);
        query_builder.push(" AND label ILIKE ");
        query_builder.push_bind(like_pattern);
    }
    if let Some(start_date) = start_date {
        query_builder.push(" AND start_date >= ");
        query_builder.push_bind(start_date);
    }
    if let Some(end_date) = end_date {
        query_builder.push(" AND end_date <= ");
        query_builder.push_bind(end_date);
    }

    let query = query_builder.build_query_as::<LeaveGroupModel>();
    println!("Generated SQL query for: {}", query.sql());
    let leave_group = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        print!("{}", e);
        anyhow!(e).context("A database failure occurred while fetching leave group")
    })?;

    Ok(leave_group)
}

pub async fn get_leave_group(
    pool: &PgPool,
    business_id: Uuid,
    id_list: Option<&[uuid::Uuid]>,
    query: Option<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Vec<LeaveGroup>, anyhow::Error> {
    let data_models =
        fetch_leave_group_models(pool, business_id, id_list, query, start_date, end_date).await?;
    let data = data_models.into_iter().map(|a| a.into_schema()).collect();
    Ok(data)
}

pub async fn leave_group_create_validation(
    pool: &PgPool,
    req: &LeaveGroupCreationRequest,
    business_id: Uuid,
    // start_date: DateTime<Utc>,
    // end_date: DateTime<Utc>,
) -> Result<(), GenericError> {
    let id_list = req.id.map(|id| vec![id]);
    let group_data_list = get_leave_group(
        pool,
        business_id,
        id_list.as_deref(),
        None,
        Some(req.start_date),
        Some(req.end_date),
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching leave group".to_string(),
            e,
        )
    })?;
    if let Some(group_data) = group_data_list.first() {
        if id_list.is_some() {
            return Err(GenericError::ValidationError(
                "Leave Group is not found for given id".to_string(),
            ));
        }

        if group_data.label == req.label {
            return Err(GenericError::ValidationError("Duplicate Label".to_string()));
        }

        return Err(GenericError::ValidationError(format!(
            "Leave Group already exists for start date {} end date {}",
            group_data.start_date, group_data.end_date
        )));
    }

    Ok(())
}

#[tracing::instrument(name = "save leave group to database", skip(pool, req))]
pub async fn save_leave_group<'a>(
    pool: &PgPool,
    req: &LeaveGroupCreationRequest,
    business_id: Uuid,
    created_by: Uuid,
) -> Result<Uuid, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO leave_group (id, label, business_id, start_date, end_date,  created_by, created_on)
        VALUES($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE
        SET label = EXCLUDED.label,
        updated_by = EXCLUDED.created_by,
        updated_on = EXCLUDED.created_on
        RETURNING id
        "#,
        &req.id.unwrap_or(Uuid::new_v4()),
        &req.label,
        business_id,
        &req.start_date,
        &req.end_date,
        &created_by,
        &Utc::now()
    );
    let result = query.fetch_one(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving leave group")
    })?;
    Ok(result.id)
}

#[tracing::instrument(name = "delete leave group", skip(pool))]
pub async fn delete_leave_group(pool: &PgPool, id: Uuid) -> Result<(), anyhow::Error> {
    sqlx::query("DELETE FROM leave_group WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to delete leave group by ID")
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow!(e).context("A database failure occurred while deleting leave group")
        })?;

    Ok(())
}

#[tracing::instrument(name = "Fetch user leave models", skip(pool))]
async fn fetch_user_leave_models(
    pool: &PgPool,
    business_id: Uuid,
    user_id: Uuid,
    group_id: Option<Uuid>,
    user_leave_id: Option<Uuid>,
) -> Result<Vec<UserLeaveModel>, anyhow::Error> {
    let mut builder = QueryBuilder::new(
        r#"
        SELECT 
            u_l.id,
            u_l.leave_type_id,
            u_l.leave_group_id,
            u_l.allocated_count,
            u_l.used_count,
            u_l.user_id,
            l_g.business_id,
            lt.label AS leave_type_label,
            l_g.label AS leave_group_label
        FROM user_leave_relationship AS u_l
        INNER JOIN leave_group AS l_g ON u_l.leave_group_id = l_g.id
        INNER JOIN leave_type AS lt ON u_l.leave_type_id = lt.id
        WHERE
        "#,
    );

    builder.push(" l_g.business_id = ").push_bind(business_id);
    builder.push(" AND u_l.user_id = ").push_bind(user_id);

    if let Some(gid) = group_id {
        builder.push(" AND u_l.leave_group_id = ").push_bind(gid);
    }

    if let Some(uid) = user_leave_id {
        builder.push(" AND u_l.id = ").push_bind(uid);
    }

    let query = builder.build_query_as::<UserLeaveModel>();

    let result = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to fetch user leave models: {:?}", e);
        anyhow!(e).context("A database failure occurred while fetching user leave")
    })?;

    Ok(result)
}

pub async fn fetch_user_leaves(
    pool: &PgPool,
    business_id: Uuid,
    user_id: Uuid,
    group_id: Option<Uuid>,
    user_leave_id: Option<Uuid>,
) -> Result<Vec<UserLeave>, anyhow::Error> {
    let data_models =
        fetch_user_leave_models(pool, business_id, user_id, group_id, user_leave_id).await?;
    let leave_type_id_list: Vec<Uuid> = data_models.iter().map(|a| a.leave_type_id).collect();
    let period_models =
        fetch_leave_periods_by_association(pool, Some(leave_type_id_list), None).await?;
    let mut period_map: HashMap<Uuid, Vec<LeavePeriodData>> = HashMap::new();
    for period in period_models {
        period_map
            .entry(period.type_id)
            .or_default()
            .push(period.into_schema());
    }
    let mut final_data = vec![];
    for data_model in data_models.into_iter() {
        let periods = period_map
            .remove(&data_model.leave_type_id)
            .unwrap_or_default();
        final_data.push(data_model.into_schema(periods))
    }

    // let mut final_data = Vec::with_capacity(data_models.len());

    // for data_model in data_models {
    //     let periods = period_map.remove(&data_model.id).unwrap_or_default();
    //     final_data.push(data_model.into_schema(periods));
    // }

    // let data = data_models.into_iter().map(|a| a.into_schema()).collect();
    Ok(final_data)
}

// let data_models =
//     fetch_leave_type_models(pool, business_id, id_list, label_list, query).await?;
// let leave_type_id_list = data_models.iter().map(|a| a.id).collect();
// let period_models = fetch_leave_periods_by_type(&pool, leave_type_id_list).await?;
// let mut period_map: HashMap<Uuid, Vec<LeavePeriodWithTypeModel>> = HashMap::new();
// for period in period_models {
//     period_map
//         .entry(period.type_id)
//         .or_insert_with(Vec::new)
//         .push(period.in);
// }
// let mut final_data = vec![];
// for data in data_models.iter() {
//     finaldata.push(data.into_schema(periods))
// }
// // let data = data_models.into_iter().map(|a| a.into_schema()).collect();
// Ok(data)

#[tracing::instrument(name = "delete payment", skip(pool))]
pub async fn delete_user_leave(
    pool: &PgPool,
    business_id: Uuid,
    user_leave_id: Uuid,
) -> Result<(), anyhow::Error> {
    sqlx::query("DELETE FROM leave_type WHERE id = $1 AND business_id = $2")
        .bind(user_leave_id)
        .bind(business_id)
        .execute(pool)
        .await
        .context("Failed to delete leave type by ID")?;

    Ok(())
}

#[tracing::instrument(name = "prepare bulk leave type data", skip())]
pub fn prepare_bulk_user_leave_data<'a>(
    leave_type_data: &'a Vec<UserLeaveCreationData>,
    user_id: Uuid,
    group_id: Uuid,
) -> BulkUserLeaveInsert<'a> {
    let current_utc = Utc::now();
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut created_by_list = vec![];
    let mut group_id_list = vec![];
    let mut type_id_list = vec![];
    let mut allocated_count_list = vec![];

    for leave_data in leave_type_data.iter() {
        created_on_list.push(current_utc);
        created_by_list.push(user_id);
        id_list.push(Uuid::new_v4());
        group_id_list.push(group_id);
        type_id_list.push(leave_data.type_id);
        allocated_count_list.push(&leave_data.count);
    }
    BulkUserLeaveInsert {
        id: id_list,
        created_on: created_on_list,
        created_by: created_by_list,
        group_id: group_id_list,
        type_id: type_id_list,
        allocated_count: allocated_count_list,
    }
}

pub async fn save_user_leave(
    pool: &PgPool,
    data: &Vec<UserLeaveCreationData>,
    user_id: Uuid,
    group_id: Uuid,
) -> Result<(), anyhow::Error> {
    if data.is_empty() {
        return Ok(());
    }
    let data = prepare_bulk_user_leave_data(data, user_id, group_id);
    let query = sqlx::query!(
        r#"
        INSERT INTO user_leave_relationship ( id, leave_type_id, leave_group_id, allocated_count, user_id, created_by, created_on)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::decimal[],  $5::uuid[], $5::uuid[], $6::TIMESTAMP[])
        ON CONFLICT (user_id, leave_group_id, leave_type_id) DO UPDATE
        SET allocated_count = EXCLUDED.allocated_count,
        updated_by = EXCLUDED.created_by,
        updated_on = EXCLUDED.created_on
        "#,
        &data.id[..] as &[Uuid],
        &data.type_id[..] as &[Uuid],
        &data.group_id[..] as &[Uuid],
        &data.allocated_count[..] as &[&BigDecimal],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
    );
    pool.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving user leave")
    })?;
    Ok(())
}

#[tracing::instrument(name = "reactivate user account", skip(transaction))]
pub async fn update_user_leave_count(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    count: &BigDecimal,
    updated_by: Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE user_leave_relationship
        SET
        used_count = used_count + $1,
        updated_on = $2,
        updated_by = $3
        WHERE id = $4
        "#,
        count,
        Utc::now(),
        updated_by,
        id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while updating leave status")
    })?;
    Ok(())
}

#[tracing::instrument(name = "prepare bulk leave period data", skip(created_by))]
pub async fn prepare_bulk_leave_period_data<'a>(
    leave_type_data: &'a Vec<LeavePeriodCreationData>,
    business_id: Uuid,
    created_by: Uuid,
) -> Result<Option<BulkLeavePeriodInsert<'a>>, anyhow::Error> {
    let current_utc = Utc::now();
    let mut label_list = vec![];
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut business_id_list = vec![];
    let mut created_by_list = vec![];
    let mut value_list = vec![];
    if leave_type_data.is_empty() {
        return Ok(None);
    }
    for leave_data in leave_type_data.iter() {
        created_on_list.push(current_utc);
        created_by_list.push(created_by);
        if let Some(id) = leave_data.id {
            id_list.push(id);
        } else {
            id_list.push(Uuid::new_v4());
        }
        label_list.push(leave_data.label.as_ref());
        business_id_list.push(business_id);
        value_list.push(&leave_data.value);
    }
    Ok(Some(BulkLeavePeriodInsert {
        id: id_list,
        label: label_list,
        value: value_list,
        created_on: created_on_list,
        created_by: created_by_list,
        business_id: business_id_list,
    }))
}

#[tracing::instrument(name = "save leave period to database", skip(pool, data))]
pub async fn save_leave_period_to_database<'a>(
    pool: &PgPool,
    data: BulkLeavePeriodInsert<'a>,
) -> Result<bool, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO leave_period (id, created_by, created_on, label, business_id, value)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::TIMESTAMP[],  $4::TEXT[], $5::uuid[], $6::decimal[]) 
        ON CONFLICT (id) DO UPDATE
        SET label = EXCLUDED.label,
        updated_by = EXCLUDED.created_by,
        updated_on = EXCLUDED.created_on
        "#,
        &data.id[..] as &[Uuid],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
        &data.label[..] as &[&str],
        &data.business_id[..] as &[Uuid],
        &data.value[..] as &[&BigDecimal]
    );
    let query_string = query.sql();
    println!("Generated SQL query for: {}", query_string);
    let result = pool.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving leave type request")
    })?;

    Ok(result.rows_affected() > 0)
}

#[tracing::instrument(name = "save leave period", skip(pool))]
pub async fn save_leave_period(
    pool: &PgPool,
    leave_period_data: &Vec<LeavePeriodCreationData>,
    created_by: Uuid,
    business_id: Uuid,
) -> Result<bool, anyhow::Error> {
    let bulk_data =
        prepare_bulk_leave_period_data(leave_period_data, business_id, created_by).await?;
    if let Some(data) = bulk_data {
        return save_leave_period_to_database(pool, data).await;
    }
    Ok(false)
}

#[tracing::instrument(name = "Fetch leave period models", skip(pool))]
async fn fetch_leave_period_models<'a>(
    pool: &PgPool,
    business_id: Uuid,
    id_list: Option<&Vec<Uuid>>,
    label_list: Option<Vec<&str>>,
    query: Option<String>,
) -> Result<Vec<LeavePeriodModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT id, value,  label FROM leave_period WHERE business_id="#,
    );
    query_builder.push_bind(business_id);
    if let Some(id_list) = id_list {
        if !id_list.is_empty() {
            query_builder.push(" AND id = ANY(");
            query_builder.push_bind(id_list);
            query_builder.push(")");
        }
    }
    if let Some(label_list) = label_list {
        if !label_list.is_empty() {
            query_builder.push(" AND label = ANY(");
            query_builder.push_bind(label_list);
            query_builder.push(")");
        }
    }
    if let Some(query) = query {
        let like_pattern = format!("%{}%", query);
        query_builder.push(" AND label ILIKE ");
        query_builder.push_bind(like_pattern);
    }

    let query = query_builder.build_query_as::<LeavePeriodModel>();
    let leave_type = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while fetching leave period")
    })?;

    Ok(leave_type)
}

#[tracing::instrument(name = "get leave period", skip(pool))]
pub async fn get_leave_period(
    pool: &PgPool,
    business_id: Uuid,
    id_list: Option<&Vec<Uuid>>,
    label_list: Option<Vec<&str>>,
    query: Option<String>,
) -> Result<Vec<LeavePeriodData>, anyhow::Error> {
    let data_models =
        fetch_leave_period_models(pool, business_id, id_list, label_list, query).await?;
    let data = data_models.into_iter().map(|a| a.into_schema()).collect();
    Ok(data)
}

#[tracing::instrument(name = "delete leave period", skip(pool))]
pub async fn delete_leave_period(pool: &PgPool, id: Uuid) -> Result<(), anyhow::Error> {
    sqlx::query("DELETE FROM leave_period WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to delete leave period by ID")
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow!(e).context("A database failure occurred while deleting leave period")
        })?;

    Ok(())
}

#[tracing::instrument(name = "prepare bulk leave type data", skip(created_by))]
pub async fn prepare_bulk_leave_type_period_relationship_data(
    leave_type_data: &Vec<LeaveTypeCreationData>,
    business_id: Uuid,
    created_by: Uuid,
    type_map: HashMap<String, Uuid>,
) -> Result<Option<BulkLeaveTypePeriodInsert>, anyhow::Error> {
    let current_utc = Utc::now();
    let mut period_id_list = vec![];
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut business_id_list = vec![];
    let mut created_by_list = vec![];
    let mut type_id_list = vec![];
    if leave_type_data.is_empty() {
        return Ok(None);
    }
    for leave_data in leave_type_data.iter() {
        if let Some(type_id) = type_map.get(&leave_data.label) {
            for id in leave_data.period_id_list.iter() {
                type_id_list.push(type_id.to_owned());
                created_on_list.push(current_utc);
                created_by_list.push(created_by);
                if let Some(id) = leave_data.id {
                    id_list.push(id);
                } else {
                    id_list.push(Uuid::new_v4());
                }

                business_id_list.push(business_id);
                period_id_list.push(id);
            }
        }
    }
    Ok(Some(BulkLeaveTypePeriodInsert {
        id: id_list,
        type_id: type_id_list,
        period_id: period_id_list,
        created_on: created_on_list,
        created_by: created_by_list,
        // business_id: business_id_list,
    }))
}

// test case not needed
#[tracing::instrument(
    name = "save leave type period relationship to database",
    skip(transaction, data)
)]
pub async fn save_leave_type_period_relationship_to_database<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    data: BulkLeaveTypePeriodInsert<'a>,
) -> Result<bool, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO leave_type_period_relationship(id, created_by, created_on, leave_type_id, leave_period_id)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::TIMESTAMP[],  $4::uuid[], $5::uuid[]) 
        ON CONFLICT (leave_type_id, leave_period_id) DO NOTHING
        "#,
        &data.id[..] as &[Uuid],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
        &data.type_id[..] as &[Uuid],
        &data.period_id[..] as &[&Uuid]
    );
    let query_string = query.sql();
    println!("Generated SQL query for: {}", query_string);
    let result = transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving leave type request")
    })?;

    Ok(result.rows_affected() > 0)
}

#[tracing::instrument(name = "save leave period relationship", skip(transaction))]
pub async fn save_leave_type_period_relationship(
    transaction: &mut Transaction<'_, Postgres>,
    leave_type_data: &Vec<LeaveTypeCreationData>,
    created_by: Uuid,
    business_id: Uuid,
    type_map: HashMap<String, Uuid>,
) -> Result<bool, anyhow::Error> {
    let bulk_data = prepare_bulk_leave_type_period_relationship_data(
        leave_type_data,
        business_id,
        created_by,
        type_map,
    )
    .await?;
    if let Some(data) = bulk_data {
        print!("aaaa{:?}", &data);
        return save_leave_type_period_relationship_to_database(transaction, data).await;
    }
    Ok(false)
}

#[tracing::instrument(name = "Fetch leave periods by type", skip(pool))]
pub async fn fetch_leave_periods_by_association(
    pool: &PgPool,
    leave_type_id: Option<Vec<Uuid>>,
    id_list: Option<Vec<Uuid>>,
) -> Result<Vec<LeavePeriodWithTypeModel>, anyhow::Error> {
    let mut query_builder = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            lp.id, 
            lp.label, 
            lp.value, 
            ltpr.leave_type_id AS type_id
        FROM leave_period AS lp
        INNER JOIN leave_type_period_relationship AS ltpr 
            ON lp.id = ltpr.leave_period_id
        WHERE 1 = 1
        "#,
    );

    if let Some(ref ids) = leave_type_id {
        if !ids.is_empty() {
            query_builder
                .push(" AND ltpr.leave_type_id = ANY(")
                .push_bind(ids)
                .push(")");
        }
    }

    if let Some(ref ids) = id_list {
        if !ids.is_empty() {
            query_builder
                .push(" AND lp.id = ANY(")
                .push_bind(ids)
                .push(")");
        }
    }

    let query = query_builder.build_query_as::<LeavePeriodWithTypeModel>();

    let rows = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to fetch leave periods with filters: {:?}", e);
        anyhow::anyhow!("Database error").context(e)
    })?;

    Ok(rows)
}
