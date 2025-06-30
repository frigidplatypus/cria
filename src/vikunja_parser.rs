use crate::debug::debug_log;
use regex::Regex;
use chrono::{DateTime, Utc, NaiveDate, Local, Duration, Datelike};
use chrono_english::{parse_date_string, Dialect};
use aho_corasick::AhoCorasick;

#[derive(Debug, Clone)]
pub struct ParsedTask {
    pub title: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub project: Option<String>,
    pub priority: Option<u8>,
    pub due_date: Option<DateTime<Utc>>,
    pub repeat_interval: Option<RepeatInterval>,
}

#[derive(Debug, Clone)]
pub struct RepeatInterval {
    #[allow(dead_code)]
    pub amount: u32,
    #[allow(dead_code)]
    pub interval_type: String, // "day", "week", "month", etc.
}

#[derive(Debug, Clone)]
pub struct QuickAddParser {
    label_regex: Regex,
    priority_regex: Regex,
    assignee_regex: Regex,
    project_regex: Regex,
    repeat_regex: Regex,
    // Enhanced date parsing
    time_regex: Regex,
    date_keywords: AhoCorasick,
    weekday_keywords: AhoCorasick,
}

impl QuickAddParser {
    pub fn new() -> Self {
        // Date and time keywords for more sophisticated parsing
        let date_patterns = vec![
            "today", "tomorrow", "yesterday",
            "next week", "this week", "last week",
            "next month", "this month", "last month",
            "next year", "this year", "last year",
            "this weekend", "next weekend",
            "later this week", "later next week",
            "end of month", "end of week", "end of year",
        ];
        
        let weekdays = vec![
            "monday", "tuesday", "wednesday", "thursday", 
            "friday", "saturday", "sunday",
            "mon", "tue", "wed", "thu", "fri", "sat", "sun"
        ];

        Self {
            // Match labels: *label or *"label with spaces"
            label_regex: Regex::new(r#"\*(?:"([^"]+)"|'([^']+)'|(\S+))"#).unwrap(),
            // Match priority: !1 through !5
            priority_regex: Regex::new(r"!([1-5])").unwrap(),
            // Match assignees: @username or @"user name"
            assignee_regex: Regex::new(r#"@(?:"([^"]+)"|'([^']+)'|(\S+))"#).unwrap(),
            // Match projects: +project or +"project with spaces"
            project_regex: Regex::new(r#"\+(?:"([^"]+)"|'([^']+)'|(\S+))"#).unwrap(),
            // Match repeating: every X days/weeks/months
            repeat_regex: Regex::new(r"every\s+(?:(\d+)\s+)?(\w+)").unwrap(),
            // Match time: "at 17:00" or "at 5pm"
            time_regex: Regex::new(r"(?i)\bat\s+(\d{1,2}):?(\d{2})?\s*(am|pm)?").unwrap(),
            // Keyword matchers for faster date detection
            date_keywords: AhoCorasick::new(date_patterns).unwrap(),
            weekday_keywords: AhoCorasick::new(weekdays).unwrap(),
        }
    }

    pub fn parse(&self, text: &str) -> ParsedTask {
        let mut task = ParsedTask {
            title: text.to_string(),
            labels: Vec::new(),
            assignees: Vec::new(),
            project: None,
            priority: None,
            due_date: None,
            repeat_interval: None,
        };

        // Extract labels
        for cap in self.label_regex.captures_iter(text) {
            let label = cap.get(1).or(cap.get(2)).or(cap.get(3)).unwrap().as_str();
            task.labels.push(label.to_string());
        }

        // Extract priority
        if let Some(cap) = self.priority_regex.captures(text) {
            task.priority = cap[1].parse().ok();
        }

        // Extract assignees
        for cap in self.assignee_regex.captures_iter(text) {
            let assignee = cap.get(1).or(cap.get(2)).or(cap.get(3)).unwrap().as_str();
            task.assignees.push(assignee.to_string());
        }

        // Extract project
        if let Some(cap) = self.project_regex.captures(text) {
            task.project = Some(cap.get(1).or(cap.get(2)).or(cap.get(3)).unwrap().as_str().to_string());
        }

        // Extract repeat interval
        if let Some(cap) = self.repeat_regex.captures(text) {
            let amount = cap.get(1).map(|m| m.as_str().parse().unwrap_or(1)).unwrap_or(1);
            let interval_type = cap[2].to_string();
            task.repeat_interval = Some(RepeatInterval { amount, interval_type });
        }

        // Parse dates (simplified - you'd want a more robust date parser)
        task.due_date = self.parse_date(text);

        // Clean the title by removing all magic syntax
        let cleaned_title = self.clean_title(text);
        debug_log(&format!("[MAGIC PARSER] Cleaned title: '{}', from input: '{}'", cleaned_title, text));
        task.title = cleaned_title;

        task
    }

    fn parse_date(&self, text: &str) -> Option<DateTime<Utc>> {
        let text_lower = text.to_lowercase();
        let now = Local::now();

        // First, try using chrono-english for natural language parsing
        if let Ok(parsed_date) = parse_date_string(&text, now.into(), Dialect::Us) {
            return Some(parsed_date);
        }

        // Enhanced date parsing with better keyword matching
        if self.date_keywords.find(&text_lower).is_some() {
            return self.parse_date_keywords(&text_lower, now);
        }

        // Check for weekday mentions
        if self.weekday_keywords.find(&text_lower).is_some() {
            return self.parse_weekday(&text_lower, now);
        }

        // Check for "in X days/weeks/months" pattern
        if let Some(duration_date) = self.parse_duration_date(&text_lower, now) {
            return Some(duration_date);
        }

        // Check for ordinal dates (17th, 23rd, etc.)
        if let Some(ordinal_date) = self.parse_ordinal_date(&text_lower, now) {
            return Some(ordinal_date);
        }

        // Try specific date formats (DD/MM/YYYY, MM/DD/YYYY, etc.)
        self.parse_specific_date(text)
    }

    fn parse_date_keywords(&self, text: &str, now: chrono::DateTime<Local>) -> Option<DateTime<Utc>> {
        // Extract time if present
        let target_time = self.extract_time(text).unwrap_or((23, 59));
        
        if text.contains("today") {
            Some(now.date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("tomorrow") {
            Some((now + Duration::days(1)).date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("yesterday") {
            Some((now - Duration::days(1)).date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("this weekend") {
            // Next Saturday
            let days_until_saturday = (6 - now.weekday().num_days_from_monday()) % 7;
            let saturday = now + Duration::days(days_until_saturday as i64);
            Some(saturday.date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("next weekend") {
            let days_until_next_saturday = 7 + (6 - now.weekday().num_days_from_monday()) % 7;
            let next_saturday = now + Duration::days(days_until_next_saturday as i64);
            Some(next_saturday.date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("next week") {
            Some((now + Duration::weeks(1)).date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("this week") {
            // End of this week (Sunday)
            let days_until_sunday = (7 - now.weekday().num_days_from_monday()) % 7;
            let sunday = now + Duration::days(days_until_sunday as i64);
            Some(sunday.date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("next month") {
            // Approximate - add 30 days
            Some((now + Duration::days(30)).date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else if text.contains("end of month") {
            // Last day of current month
            let mut last_day = now.date_naive();
            last_day = last_day.with_day(1).unwrap();
            last_day = last_day + Duration::days(32); // Move to next month
            last_day = last_day.with_day(1).unwrap();
            last_day = last_day - Duration::days(1); // Go back to last day of current month
            Some(last_day.and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else {
            None
        }
    }

    fn parse_weekday(&self, text: &str, now: chrono::DateTime<Local>) -> Option<DateTime<Utc>> {
        let target_time = self.extract_time(text).unwrap_or((23, 59));
        
        let weekdays = [
            ("monday", 0), ("mon", 0),
            ("tuesday", 1), ("tue", 1),
            ("wednesday", 2), ("wed", 2),
            ("thursday", 3), ("thu", 3),
            ("friday", 4), ("fri", 4),
            ("saturday", 5), ("sat", 5),
            ("sunday", 6), ("sun", 6),
        ];

        for (day_name, target_weekday) in &weekdays {
            if text.contains(day_name) {
                let current_weekday = now.weekday().num_days_from_monday();
                let days_ahead = if *target_weekday >= current_weekday {
                    *target_weekday - current_weekday
                } else {
                    7 - current_weekday + *target_weekday
                };
                
                let target_date = now + Duration::days(days_ahead as i64);
                return Some(target_date.date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc());
            }
        }
        
        None
    }

    fn parse_duration_date(&self, text: &str, now: chrono::DateTime<Local>) -> Option<DateTime<Utc>> {
        let duration_regex = Regex::new(r"in\s+(\d+)\s+(day|week|month|hour)s?").unwrap();
        
        if let Some(cap) = duration_regex.captures(text) {
            let amount: i64 = cap[1].parse().ok()?;
            let unit = &cap[2];
            let target_time = self.extract_time(text).unwrap_or((23, 59));
            
            let target_date = match unit {
                "hour" => now + Duration::hours(amount),
                "day" => now + Duration::days(amount),
                "week" => now + Duration::weeks(amount),
                "month" => now + Duration::days(amount * 30), // Approximate
                _ => return None,
            };
            
            if unit == "hour" {
                Some(target_date.with_timezone(&Utc))
            } else {
                Some(target_date.date_naive().and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
            }
        } else {
            None
        }
    }

    fn parse_ordinal_date(&self, text: &str, now: chrono::DateTime<Local>) -> Option<DateTime<Utc>> {
        let ordinal_regex = Regex::new(r"(\d{1,2})(?:st|nd|rd|th)").unwrap();
        
        if let Some(cap) = ordinal_regex.captures(text) {
            let day: u32 = cap[1].parse().ok()?;
            let target_time = self.extract_time(text).unwrap_or((23, 59));
            
            // Use current month and year
            let target_date = now.date_naive().with_day(day)?;
            Some(target_date.and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else {
            None
        }
    }

    fn extract_time(&self, text: &str) -> Option<(u32, u32)> {
        if let Some(cap) = self.time_regex.captures(text) {
            let hour: u32 = cap[1].parse().ok()?;
            let minute: u32 = cap.get(2).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
            let am_pm = cap.get(3).map(|m| m.as_str().to_lowercase());
            
            let adjusted_hour = match am_pm.as_deref() {
                Some("pm") if hour != 12 => hour + 12,
                Some("am") if hour == 12 => 0,
                _ => hour,
            };
            
            if adjusted_hour < 24 && minute < 60 {
                Some((adjusted_hour, minute))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse_specific_date(&self, text: &str) -> Option<DateTime<Utc>> {
        // Enhanced date format parsing
        let target_time = self.extract_time(text).unwrap_or((23, 59));

        // Try DD/MM/YYYY format
        if let Some(caps) = Regex::new(r"(\d{1,2})/(\d{1,2})/(\d{4})").unwrap().captures(text) {
            let day: u32 = caps[1].parse().ok()?;
            let month: u32 = caps[2].parse().ok()?;
            let year: i32 = caps[3].parse().ok()?;
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(date.and_hms_opt(target_time.0, target_time.1, 59)?.and_utc());
            }
        }

        // Try YYYY-MM-DD (ISO format)
        if let Some(caps) = Regex::new(r"(\d{4})-(\d{1,2})-(\d{1,2})").unwrap().captures(text) {
            let year: i32 = caps[1].parse().ok()?;
            let month: u32 = caps[2].parse().ok()?;
            let day: u32 = caps[3].parse().ok()?;
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(date.and_hms_opt(target_time.0, target_time.1, 59)?.and_utc());
            }
        }

        // Try month name formats (Feb 17, February 17th, etc.)
        self.parse_month_name_date(text)
    }

    fn parse_month_name_date(&self, text: &str) -> Option<DateTime<Utc>> {
        let month_regex = Regex::new(
            r"(?i)(jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s+(\d{1,2})(?:st|nd|rd|th)?"
        ).unwrap();

        if let Some(caps) = month_regex.captures(text) {
            let month_str = caps[1].to_lowercase();
            let day: u32 = caps[2].parse().ok()?;
            let target_time = self.extract_time(text).unwrap_or((23, 59));

            let month_num = match month_str.as_str() {
                "jan" | "january" => 1,
                "feb" | "february" => 2,
                "mar" | "march" => 3,
                "apr" | "april" => 4,
                "may" => 5,
                "jun" | "june" => 6,
                "jul" | "july" => 7,
                "aug" | "august" => 8,
                "sep" | "september" => 9,
                "oct" | "october" => 10,
                "nov" | "november" => 11,
                "dec" | "december" => 12,
                _ => return None,
            };

            let current_year = Local::now().year();
            let date = NaiveDate::from_ymd_opt(current_year, month_num, day)?;
            Some(date.and_hms_opt(target_time.0, target_time.1, 59)?.and_utc())
        } else {
            None
        }
    }

    fn clean_title(&self, text: &str) -> String {
        let mut cleaned = text.to_string();

        // Remove all magic syntax
        cleaned = self.label_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.priority_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.assignee_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.project_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.repeat_regex.replace_all(&cleaned, "").to_string();
        cleaned = self.time_regex.replace_all(&cleaned, "").to_string();

        // Remove date-related text more intelligently
        cleaned = self.remove_date_text(&cleaned);

        // Clean up extra whitespace and normalize
        cleaned.split_whitespace()
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn remove_date_text(&self, text: &str) -> String {
        let mut cleaned = text.to_string();
        
        // Remove date keywords - order matters, longer patterns first
        let date_patterns = [
            r"(?i)\blater\s+(this|next)\s+week\b",
            r"(?i)\bend\s+of\s+(week|month|year)\b",
            r"(?i)\bin\s+\d+\s+(day|week|month|hour)s?\b",
            r"(?i)\bnext\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
            r"(?i)\b(this|next|last)\s+(week|month|year|weekend)\b", 
            r"(?i)\b(today|tomorrow|yesterday)\b",
            r"(?i)\b(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b",
            r"(?i)\b(mon|tue|wed|thu|fri|sat|sun)\b",
            r"(?i)\b(jan|january|feb|february|mar|march|apr|april|may|jun|june|jul|july|aug|august|sep|september|oct|october|nov|november|dec|december)\s+\d{1,2}(?:st|nd|rd|th)?\b",
            r"\b\d{1,2}/\d{1,2}/\d{4}\b",
            r"\b\d{4}-\d{1,2}-\d{1,2}\b",
            r"\b\d{1,2}(?:st|nd|rd|th)\b",
        ];

        for pattern in &date_patterns {
            let regex = Regex::new(pattern).unwrap();
            cleaned = regex.replace_all(&cleaned, "").to_string();
        }

        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_with_magic() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Buy groceries *shopping @john +personal tomorrow !2");

        assert_eq!(task.title, "Buy groceries");
        assert_eq!(task.labels, vec!["shopping"]);
        assert_eq!(task.assignees, vec!["john"]);
        assert_eq!(task.project, Some("personal".to_string()));
        assert_eq!(task.priority, Some(2));
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_parse_labels_with_spaces() {
        let parser = QuickAddParser::new();
        let task = parser.parse(r#"Task with *"label with spaces" and *simple"#);

        assert_eq!(task.labels, vec!["label with spaces", "simple"]);
    }

    #[test]
    fn test_parse_repeat_interval() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Daily standup every 2 days");

        assert!(task.repeat_interval.is_some());
        let repeat = task.repeat_interval.unwrap();
        assert_eq!(repeat.amount, 2);
        assert_eq!(repeat.interval_type, "days");
    }

    #[test]
    fn test_enhanced_date_parsing() {
        let parser = QuickAddParser::new();
        
        // Test time extraction
        let task1 = parser.parse("Meeting tomorrow at 2:30pm");
        assert!(task1.due_date.is_some());
        assert_eq!(task1.title, "Meeting");
        
        // Test weekday parsing
        let task2 = parser.parse("Call mom next friday");
        assert!(task2.due_date.is_some());
        assert_eq!(task2.title, "Call mom");
        
        // Test ordinal dates
        let task3 = parser.parse("Pay rent 15th");
        assert!(task3.due_date.is_some());
        assert_eq!(task3.title, "Pay rent");
        
        // Test duration parsing
        let task4 = parser.parse("Follow up in 3 days");
        assert!(task4.due_date.is_some());
        assert_eq!(task4.title, "Follow up");
    }

    #[test]
    fn test_complex_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse(
            r#"Review proposal *urgent *"high priority" @jane @"john doe" +"Client Work" next monday at 10am !4 every week"#
        );

        assert_eq!(task.title, "Review proposal");
        assert_eq!(task.labels, vec!["urgent", "high priority"]);
        assert_eq!(task.assignees, vec!["jane", "john doe"]);
        assert_eq!(task.project, Some("Client Work".to_string()));
        assert_eq!(task.priority, Some(4));
        assert!(task.due_date.is_some());
        assert!(task.repeat_interval.is_some());
    }

    #[test]
    fn test_month_name_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Submit report Feb 17th at 5pm");
        
        assert_eq!(task.title, "Submit report");
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_weekend_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Clean garage this weekend");
        
        assert_eq!(task.title, "Clean garage");
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_time_only_parsing() {
        let parser = QuickAddParser::new();
        let task = parser.parse("Team meeting at 10:30am *important");
        
        assert_eq!(task.title, "Team meeting");
        assert_eq!(task.labels, vec!["important"]);
        // Note: without a date, time extraction alone may not create a due_date
    }
}
