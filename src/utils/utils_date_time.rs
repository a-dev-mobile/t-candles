use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};

/// Принимает время в секундах (Unix timestamp) и возвращает
/// время 23:59:59 того же дня в секундах (Unix timestamp)
///
/// # Arguments
/// * `timestamp_seconds` - Unix timestamp в секундах
///
/// # Returns
/// Unix timestamp в секундах для конца дня (23:59:59)
pub fn get_end_of_day(timestamp_seconds: i64) -> i64 {
    // Преобразуем Unix timestamp в DateTime
    let datetime = DateTime::from_timestamp(timestamp_seconds, 0).unwrap_or_else(|| Utc::now());

    // Получаем год, месяц и день из входного времени
    let year = datetime.year();
    let month = datetime.month();
    let day = datetime.day();

    // Создаем новую дату с временем 23:59:59
    let end_of_day = Utc
        .with_ymd_and_hms(year, month, day, 23, 59, 59)
        .single()
        .unwrap_or_default();

    // Возвращаем новое время в формате Unix timestamp (секунды)
    end_of_day.timestamp()
}

/// Принимает время в секундах (Unix timestamp) и возвращает
/// два значения в секундах (Unix timestamp):
/// 1. Начало следующего дня (00:00:00)
/// 2. Конец следующего дня (23:59:59)
///
/// # Arguments
/// * `timestamp_seconds` - Unix timestamp в секундах
///
/// # Returns
/// Кортеж из двух Unix timestamp в секундах (начало и конец следующего дня)
pub fn get_next_day_range(timestamp_seconds: i64) -> (i64, i64) {
    // Преобразуем Unix timestamp в DateTime
    let datetime = DateTime::from_timestamp(timestamp_seconds, 0).unwrap_or_else(|| Utc::now());

    // Добавляем 1 день
    let next_day = datetime + Duration::days(1);

    // Получаем год, месяц и день следующего дня
    let year = next_day.year();
    let month = next_day.month();
    let day = next_day.day();

    // Создаем временную метку на начало следующего дня (00:00:00)
    let start_of_next_day = Utc
        .with_ymd_and_hms(year, month, day, 0, 0, 0)
        .single()
        .unwrap_or_default();

    // Создаем временную метку на конец следующего дня (23:59:59)
    let end_of_next_day = Utc
        .with_ymd_and_hms(year, month, day, 23, 59, 59)
        .single()
        .unwrap_or_default();

    // Возвращаем обе временные метки в формате Unix timestamp (секунды)
    (start_of_next_day.timestamp(), end_of_next_day.timestamp())
}

/// Принимает время в секундах (Unix timestamp) и возвращает
/// два значения в секундах (Unix timestamp):
/// 1. Начало предыдущего дня (00:00:00)
/// 2. Конец предыдущего дня (23:59:59)
///
/// # Arguments
/// * `timestamp_seconds` - Unix timestamp в секундах, если None, используется текущее время
///
/// # Returns
/// Кортеж из двух Unix timestamp в секундах (начало и конец предыдущего дня)
 pub fn get_yesterday_range(timestamp_seconds: Option<i64>) -> (i64, i64) {
    // Используем переданное время или текущее, если время не передано
    let datetime = match timestamp_seconds {
        Some(ts) => DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()),
        None => Utc::now(),
    };
    
    // Вычитаем 1 день, чтобы получить вчерашний день
    let yesterday = datetime - Duration::days(1);
    
    // Получаем год, месяц и день вчерашнего дня
    let year = yesterday.year();
    let month = yesterday.month();
    let day = yesterday.day();
    
    // Создаем временную метку на начало вчерашнего дня (00:00:00)
    let start_of_yesterday = Utc
        .with_ymd_and_hms(year, month, day, 0, 0, 0)
        .single()
        .unwrap_or_default();
    
    // Создаем временную метку на конец вчерашнего дня (23:59:59)
    let end_of_yesterday = Utc
        .with_ymd_and_hms(year, month, day, 23, 59, 59)
        .single()
        .unwrap_or_default();
    
    // Возвращаем обе временные метки в формате Unix timestamp (секунды)
    (start_of_yesterday.timestamp(), end_of_yesterday.timestamp())
}