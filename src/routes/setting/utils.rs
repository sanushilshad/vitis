use std::collections::HashMap;

use super::{
    models::{BulkSettingCreateModel, SettingEnumModel, SettingModel, SettingValueModel},
    schemas::{
        CreateBusinessSettingRequest, CreateSettingData, Setting, SettingEnumData, SettingType,
        Settings,
    },
};
use chrono::DateTime;
use chrono::Utc;
use sqlx::{Execute, Executor, PgPool, QueryBuilder};
use uuid::Uuid;

pub async fn fetch_setting(
    pool: &PgPool,
    key_list: &Vec<String>,
    r#type: SettingType,
) -> Result<Vec<SettingModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT id, key, is_editable, enum_id, cluster_id
        FROM setting
        WHERE is_deleted = false
        "#,
    );

    query_builder.push(" AND key = ANY(");
    query_builder.push_bind(key_list);
    query_builder.push(")");

    match r#type {
        SettingType::Business => {
            query_builder.push(" AND is_business = true");
        }
        SettingType::User => {
            query_builder.push(" AND is_user = true");
        }
        SettingType::Global => {
            query_builder.push(" AND is_global = true");
        }
        SettingType::UserBusiness => {
            query_builder.push(" AND is_user_business = true");
        }
    }

    let query = query_builder.build_query_as::<SettingModel>();

    let rows = query.fetch_all(pool).await?;
    Ok(rows)
}

fn get_setting_bulk_insert_data(
    settings: &[CreateSettingData],
    user_id: Option<Uuid>,
    created_by: Uuid,
    business_account_id: Option<Uuid>,
    setting_map: &HashMap<String, &SettingModel>,
) -> BulkSettingCreateModel {
    let mut id_list = vec![];
    let mut user_id_list = vec![];
    let mut business_id_list = vec![];
    let mut setting_id_list = vec![];
    let mut value_list = vec![];
    let mut created_on_list = vec![];
    let mut created_by_list = vec![];
    let created_on = Utc::now();
    for setting in settings.iter() {
        if let Some(setting_obj) = setting_map.get(&setting.key) {
            setting_id_list.push(setting_obj.id);
        } else {
            continue;
        }

        id_list.push(Uuid::new_v4());
        user_id_list.push(user_id);
        business_id_list.push(business_account_id);
        value_list.push(setting.value.to_owned());
        created_on_list.push(created_on);
        created_by_list.push(created_by);
    }
    BulkSettingCreateModel {
        id_list,
        user_id_list,
        business_id_list,
        setting_id_list,
        value_list,
        created_on_list,
        created_by_list,
    }
}

async fn create_setting(
    pool: &PgPool,
    bulk_data: BulkSettingCreateModel,
) -> Result<(), anyhow::Error> {
    // (setting_id, user_id, business_id) DO UPDATE
    // SET value = EXCLUDED.value;
    let query = sqlx::query!(
        r#"
        INSERT INTO setting_value(id, user_id, business_id, setting_id, value, created_by, created_on)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[], $4::uuid[], $5::text[], $6::uuid[], $7::timestamp[])
            ON CONFLICT (setting_id, user_id, business_id) DO UPDATE
            SET value = EXCLUDED.value,
            updated_by = EXCLUDED.created_by,
            updated_on =EXCLUDED.created_on

        "#,
        &bulk_data.id_list[..] as &[Uuid],
        &bulk_data.user_id_list as &[Option<Uuid>],
        &bulk_data.business_id_list as &[Option<Uuid>],
        &bulk_data.setting_id_list as &[Uuid],
        &bulk_data.value_list as &[String],
        &bulk_data.created_by_list as &[Uuid],
        &bulk_data.created_on_list as &[DateTime<Utc>],
    );
    pool.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e)
            .context("A database failure occurred while saving settings to database")
    })?;
    Ok(())
}

// pub async fn create_business_setting(
//     pool: &PgPool,
//     setting_req: &CreateBusinessSettingRequest,
//     created_by: Uuid,
//     business_account_id: Uuid,
//     setting_map: &HashMap<String, &SettingModel>,
// ) -> Result<(), anyhow::Error> {
//     let bulk_data = get_setting_bulk_insert_data(
//         &setting_req.settings,
//         None,
//         created_by,
//         Some(business_account_id),
//         setting_map,
//     );
//     create_setting(pool, bulk_data).await?;

//     Ok(())
// }

// pub async fn create_user_setting(
//     pool: &PgPool,
//     settings: &[CreateSettingData],
//     user_id: Uuid,
//     created_by: Uuid,
//     setting_map: &HashMap<String, &SettingModel>,
// ) -> Result<(), anyhow::Error> {
//     let bulk_data =
//         get_setting_bulk_insert_data(settings, Some(user_id), created_by, None, setting_map);
//     create_setting(pool, bulk_data).await?;

//     Ok(())
// }

// pub async fn create_user_business_setting(
//     pool: &PgPool,
//     setting_req: &CreateBusinessSettingRequest,
//     user_id: Uuid,
//     created_by: Uuid,
//     business_account_id: Uuid,
//     setting_map: &HashMap<String, &SettingModel>,
// ) -> Result<(), anyhow::Error> {
//     let bulk_data = get_setting_bulk_insert_data(
//         &setting_req.settings,
//         Some(user_id),
//         created_by,
//         Some(business_account_id),
//         setting_map,
//     );
//     create_setting(pool, bulk_data).await?;

//     Ok(())
// }

// pub async fn create_global_setting(
//     pool: &PgPool,
//     settings: &[CreateSettingData],
//     created_by: Uuid,
//     setting_map: &HashMap<String, &SettingModel>,
// ) -> Result<(), anyhow::Error> {
//     let bulk_data = get_setting_bulk_insert_data(settings, None, created_by, None, setting_map);
//     create_setting(pool, bulk_data).await?;

//     Ok(())
// }

pub async fn create_setting_with_scope(
    pool: &PgPool,
    settings: &[CreateSettingData],
    user_id: Option<Uuid>,
    business_id: Option<Uuid>,
    created_by: Uuid,
    setting_map: &HashMap<String, &SettingModel>,
) -> Result<(), anyhow::Error> {
    let bulk_data =
        get_setting_bulk_insert_data(settings, user_id, created_by, business_id, setting_map);
    create_setting(pool, bulk_data).await?;
    Ok(())
}

async fn fetch_setting_value_model(
    pool: &PgPool,
    key_list: &Vec<String>,
    business_id: Option<Uuid>,
    user_id: Option<Uuid>,
    fetch_multi_level: bool,
) -> Result<Vec<SettingValueModel>, anyhow::Error> {
    let mut query = QueryBuilder::new(
        r#"
        SELECT 
            sv.id AS id,
            s.cluster_id as cluster_id,
            s.key AS key,
            sv.value AS value,
            s.label AS label,
            s.enum_id AS enum_id,
            sv.user_id AS user_id,
            sv.business_id AS business_id,
            s.is_editable AS is_editable
        FROM 
            setting AS s
            LEFT JOIN setting_value AS sv ON sv.setting_id = s.id AND sv.is_deleted = false
        "#,
    );

    // JOIN on setting_value
    // query.push("LEFT JOIN setting_value AS sv ON sv.setting_id = s.id AND sv.is_deleted = false");

    // Scope filters
    match (user_id, business_id) {
        (Some(user_id), Some(business_id)) => {
            query.push(" AND (");
            query.push("(sv.user_id = ");
            query.push_bind(user_id);
            query.push(" AND sv.business_id = ");
            query.push_bind(business_id);
            query.push(") OR (sv.user_id IS NULL AND sv.business_id = ");
            query.push_bind(business_id);
            query.push(") OR (sv.user_id IS NULL AND sv.business_id IS NULL))");
        }
        (Some(user_id), None) => {
            query.push(" AND (");
            query.push("(sv.user_id = ");
            query.push_bind(user_id);
            query.push(" AND sv.business_id IS NULL)");
            if fetch_multi_level {
                query.push(" OR (sv.user_id IS NULL AND sv.business_id IS NULL)");
            }
            query.push(")");
        }
        (None, Some(business_id)) => {
            query.push(" AND ((sv.user_id IS NULL AND sv.business_id = ");
            query.push_bind(business_id);
            query.push(") OR (sv.user_id IS NULL AND sv.business_id IS NULL))");
        }
        (None, None) => {
            query.push(" AND sv.user_id IS NULL AND sv.business_id IS NULL");
        }
    }
    query.push(" WHERE s.is_deleted = false");
    match (user_id.is_some(), business_id.is_some(), fetch_multi_level) {
        (true, false, true) => {
            query.push(" AND (s.is_user = true OR s.is_global = true)");
        }
        (true, false, false) => {
            query.push(" AND s.is_user = true");
        }

        (true, true, true) => {
            // query.push(" AND s.is_user=true or  s.is_business = true");
            query.push(" AND (s.is_user = true OR s.is_global = true OR s.is_business = true OR s.is_user_business = true)");
        }
        (true, true, false) => {
            // query.push(" AND s.is_user=true or  s.is_business = true");
            query.push(" AND (s.is_user_business = true)");
        }
        (false, true, false) => {
            // query.push(" AND s.is_user=true or  s.is_business = true");
            query.push(" AND s.is_business = true");
        }
        (false, false, _) => {
            query.push(" AND (s.is_global = true)");
        }
        _ => {}
    }

    // Filter by keys if present
    if !key_list.is_empty() {
        query.push(" AND s.key = ANY(");
        query.push_bind(key_list);
        query.push(")");
    }

    let final_query = query.build_query_as::<SettingValueModel>();
    println!("Generated SQL query for: {}", final_query.sql());

    let rows = final_query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("Failed to fetch settings from database")
    })?;

    Ok(rows)
}

pub fn get_setting_value_from_model(
    data_models: Vec<SettingValueModel>,
    user_id: Option<Uuid>,
    business_id: Option<Uuid>,
) -> Vec<Settings> {
    let mut settings_map: HashMap<String, Settings> = HashMap::new();

    for model in data_models.into_iter() {
        let entry = settings_map
            .entry(model.key.clone())
            .or_insert_with(|| Settings {
                key: model.key,
                label: model.label,
                enum_id: model.enum_id,
                is_editable: model.is_editable,
                global_level: vec![],
                user_level: vec![],
                business_level: vec![],
                user_business_level: vec![],
            });

        let setting = Setting {
            id: model.id,
            value: model.value,
        };
        match (model.user_id, model.business_id) {
            (Some(_), Some(_)) => entry.user_business_level.push(setting),
            (Some(_), None) => entry.user_level.push(setting),
            (None, Some(_)) => entry.business_level.push(setting),
            (None, None) => {
                // Use the fallback context
                match (user_id.is_some(), business_id.is_some()) {
                    (true, true) => entry.user_business_level.push(setting),
                    (true, false) => entry.user_level.push(setting),
                    (false, true) => entry.business_level.push(setting),
                    (false, false) => entry.global_level.push(setting),
                }
            }
        }
    }

    settings_map.into_values().collect()
}

pub async fn get_setting_value(
    pool: &PgPool,
    key_list: &Vec<String>,
    business_id: Option<Uuid>,
    user_id: Option<Uuid>,
    fetch_multi_level: bool,
) -> Result<Vec<Settings>, anyhow::Error> {
    let data_models =
        fetch_setting_value_model(pool, key_list, business_id, user_id, fetch_multi_level).await?;
    let data = get_setting_value_from_model(data_models, user_id, business_id);
    Ok(data)
}
#[allow(dead_code)]
pub async fn delete_global_setting(pool: &PgPool) -> Result<(), anyhow::Error> {
    let _ = sqlx::query("DELETE FROM setting_value WHERE user_id IS NULL AND business_id IS NULL")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow::Error::new(e)
                .context("A database failure occurred while deleting global setting from database")
        })?;
    Ok(())
}

#[tracing::instrument(name = "Get setting enum", skip(pool))]
pub async fn fetch_setting_enum_models(
    pool: &PgPool,
    value_list: &Vec<Uuid>,
) -> Result<Vec<SettingEnumModel>, anyhow::Error> {
    let row: Vec<SettingEnumModel> = sqlx::query_as!(
        SettingEnumModel,
        r#"SELECT id, values  as "values:sqlx::types::Json<Vec<String>>" FROM setting_enum
        WHERE id = ANY($1)
        "#,
        &value_list
    )
    .fetch_all(pool)
    .await?;

    Ok(row)
}

#[tracing::instrument(name = "Get setting enum", skip(pool))]
pub async fn fetch_setting_enums(
    pool: &PgPool,
    ids: &Vec<Uuid>,
) -> Result<Vec<SettingEnumData>, anyhow::Error> {
    let models = fetch_setting_enum_models(pool, ids).await?;
    let rows = models.into_iter().map(|e| e.into_schema()).collect();
    Ok(rows)
}
