use std::time::{SystemTime, Duration, UNIX_EPOCH};

/// Parse a string from an ISO 9660 buffer
pub fn parse_iso_string(buffer: &[u8]) -> String {
    let mut end = buffer.len();
    
    // Find the end of the string (null terminator or space padding)
    for i in 0..buffer.len() {
        if buffer[i] == 0 || buffer[i] == b' ' {
            end = i;
            break;
        }
    }
    
    // Convert to string, replacing invalid UTF-8 with replacement character
    String::from_utf8_lossy(&buffer[0..end]).into_owned()
}

/// Write a string to an ISO 9660 buffer, padded with spaces
pub fn write_iso_string(buffer: &mut [u8], s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len().min(buffer.len());
    
    // Clear buffer with spaces
    buffer.fill(b' ');
    
    // Copy string
    buffer[0..len].copy_from_slice(&bytes[0..len]);
}

/// Parse a u16 stored in both little and big endian
pub fn parse_u16_both(buffer: &[u8]) -> u16 {
    if buffer.len() < 4 {
        return 0;
    }
    
    // Use little endian value
    u16::from_le_bytes([buffer[0], buffer[1]])
}

/// Write a u16 in both little and big endian format
pub fn write_u16_both(buffer: &mut [u8], value: u16) {
    if buffer.len() < 4 {
        return;
    }
    
    // Little endian
    buffer[0..2].copy_from_slice(&value.to_le_bytes());
    
    // Big endian
    buffer[2..4].copy_from_slice(&value.to_be_bytes());
}

/// Parse a u32 stored in both little and big endian
pub fn parse_u32_both(buffer: &[u8]) -> u32 {
    if buffer.len() < 8 {
        return 0;
    }
    
    // Use little endian value
    u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]])
}

/// Write a u32 in both little and big endian format
pub fn write_u32_both(buffer: &mut [u8], value: u32) {
    if buffer.len() < 8 {
        return;
    }
    
    // Little endian
    buffer[0..4].copy_from_slice(&value.to_le_bytes());
    
    // Big endian
    buffer[4..8].copy_from_slice(&value.to_be_bytes());
}

/// Parse ISO 9660 date format (7 bytes)
///
/// Format: year (since 1900), month, day, hour, minute, second, timezone offset (in 15-minute intervals from GMT)
pub fn parse_recording_date(buffer: &[u8]) -> Option<SystemTime> {
    if buffer.len() < 7 {
        return None;
    }
    
    let year = 1900 + buffer[0] as u32;
    let month = buffer[1] as u32;
    let day = buffer[2] as u32;
    let hour = buffer[3] as u32;
    let minute = buffer[4] as u32;
    let second = buffer[5] as u32;
    let _tz_offset = buffer[6] as i8; // Timezone offset in 15-minute intervals
    
    // Convert to Unix timestamp
    // This is a simplified implementation
    let days_since_epoch = (year - 1970) * 365 + (year - 1969) / 4 + days_in_month(year, month) + day - 1;
    let seconds_since_epoch = days_since_epoch * 86400 + hour * 3600 + minute * 60 + second;
    
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds_since_epoch as u64))
}

/// Write recording date in ISO 9660 format
pub fn write_recording_date(buffer: &mut [u8], time: SystemTime) {
    if buffer.len() < 7 {
        return;
    }
    
    // Convert to calendar date/time
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_else(|_| Duration::from_secs(0));
    let secs = duration.as_secs();
    
    let second = (secs % 60) as u8;
    let minute = ((secs / 60) % 60) as u8;
    let hour = ((secs / 3600) % 24) as u8;
    
    // Calculate days since epoch (Jan 1, 1970)
    let days = (secs / 86400) as u32;
    
    // Simple estimate of year/month/day
    // This is a simplified calculation
    let year = 1970 + days / 365;
    let remaining_days = days % 365;
    
    // Simple month calculation (not accurate for leap years)
    let (month, day) = estimate_month_day(remaining_days, is_leap_year(year));
    
    buffer[0] = (year - 1900) as u8;
    buffer[1] = month as u8;
    buffer[2] = day as u8;
    buffer[3] = hour;
    buffer[4] = minute;
    buffer[5] = second;
    buffer[6] = 0; // GMT
}

/// Parse ISO 9660 extended date format (17 bytes)
///
/// Format: "YYYYMMDDHHMMSSFF"
pub fn parse_iso_date(buffer: &[u8]) -> Option<SystemTime> {
    if buffer.len() < 16 {
        return None;
    }
    
    let s = String::from_utf8_lossy(buffer);
    
    // Check format
    if s.len() < 16 || !s.is_ascii() {
        return None;
    }
    
    // Parse components
    let year = s[0..4].parse::<u32>().ok()?;
    let month = s[4..6].parse::<u32>().ok()?;
    let day = s[6..8].parse::<u32>().ok()?;
    let hour = s[8..10].parse::<u32>().ok()?;
    let minute = s[10..12].parse::<u32>().ok()?;
    let second = s[12..14].parse::<u32>().ok()?;
    let _hundredths = s[14..16].parse::<u32>().ok()?;
    
    // Convert to Unix timestamp
    let days_since_epoch = (year - 1970) * 365 + (year - 1969) / 4 + days_in_month(year, month) + day - 1;
    let seconds_since_epoch = days_since_epoch * 86400 + hour * 3600 + minute * 60 + second;
    
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds_since_epoch as u64))
}

/// Write date in ISO 9660 format
pub fn write_iso_date(buffer: &mut [u8], time: SystemTime) {
    if buffer.len() < 17 {
        return;
    }
    
    // Use Unix epoch if the time is before it
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_else(|_| Duration::from_secs(0));
    let secs = duration.as_secs();
    
    let second = (secs % 60) as u8;
    let minute = ((secs / 60) % 60) as u8;
    let hour = ((secs / 3600) % 24) as u8;
    
    // Calculate days since epoch (Jan 1, 1970)
    let days = (secs / 86400) as u32;
    
    // Simple estimate of year/month/day
    let year = 1970 + days / 365;
    let remaining_days = days % 365;
    
    // Simple month calculation (not accurate for leap years)
    let (month, day) = estimate_month_day(remaining_days, is_leap_year(year));
    
    // Format as "YYYYMMDDHHMMSSFF"
    let date_str = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}00",
        year, month, day, hour, minute, second
    );
    
    // Copy to buffer
    buffer[0..16].copy_from_slice(date_str.as_bytes());
    
    // Timezone offset (byte 16): 0 = GMT
    buffer[16] = 0;
}

/// Check if a year is a leap year
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Calculate days in month
fn days_in_month(year: u32, month: u32) -> u32 {
    let days_before_month = match month {
        1 => 0,
        2 => 31,
        3 => 59,
        4 => 90,
        5 => 120,
        6 => 151,
        7 => 181,
        8 => 212,
        9 => 243,
        10 => 273,
        11 => 304,
        12 => 334,
        _ => 0,
    };
    
    // Add leap day if needed
    if month > 2 && is_leap_year(year) {
        days_before_month + 1
    } else {
        days_before_month
    }
}

/// Estimate month and day from days since start of year
fn estimate_month_day(days_since_start_of_year: u32, is_leap: bool) -> (u32, u32) {
    let days_in_month = [
        31, if is_leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31
    ];
    
    let mut remaining_days = days_since_start_of_year;
    let mut month = 1;
    
    for &days in &days_in_month {
        if remaining_days < days {
            return (month, remaining_days + 1);
        }
        
        remaining_days -= days;
        month += 1;
    }
    
    // Default to December 31 if something went wrong
    (12, 31)
}