use std::time;

#[derive(Debug)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32
}

fn get_days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 => {
            if has_extra_day(year) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0, 
    }
}

fn has_extra_day(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn get_current_time() -> Date {
    let time = time::SystemTime::now();
    let secs = time.duration_since(time::UNIX_EPOCH).expect("Share your time machine bruh").as_secs() + 2*60*60;

    let seconds_in_a_day = 86400;

    let mut days = secs / seconds_in_a_day;
    let remaining_secs = secs % seconds_in_a_day;
    let mut year = 1970;
    let mut month = 1;
    let mut day = 1;

    while days >= 365 {
        if has_extra_day(year) {
            if days >= 366 {
                days -= 366;
                year += 1;
            } else {
                break;
            }
        } else {
            days -= 365;
            year += 1;
        }
    }

    while days > get_days_in_month(year, month) as u64 {
        days -= get_days_in_month(year, month) as u64;
        month += 1;
    }

    day += days;

    let hour = remaining_secs/3600;
    let minute = (remaining_secs%3600)/60;
    let second = (remaining_secs%3600)%60;

    Date{
        year,
        month,
        day: day as u32,
        hour: hour as u32,
        minute: minute as u32,
        second: second as u32
    }
}