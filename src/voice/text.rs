use chrono::{FixedOffset, TimeZone, Utc};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use serenity::{
    all::MessageFlags, cache::Cache, model::channel::Message, utils::{ContentSafeOptions, content_safe}
};

static RE_MASKED_LINK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([^\]]+)\]\((https?://[^)\s]+)\)").unwrap());

static RE_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://\S+").unwrap());

static RE_CODE_BLOCK: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)```.*?```").unwrap());

static RE_INLINE_CODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`[^`\n]+`").unwrap());

static RE_SPOILER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\|\|(.+?)\|\|").unwrap());

static RE_SLASH_COMMAND: Lazy<Regex> = Lazy::new(|| Regex::new(r"</([^:>]+):\d+>").unwrap());

static RE_CUSTOM_EMOJI: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<a?:([A-Za-z0-9_]+):\d+>").unwrap());

static RE_TIMESTAMP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<t:(\d+)(?::([tTdDfFsSR]))?>").unwrap());

static RE_GUILD_NAV: Lazy<Regex> = Lazy::new(|| Regex::new(r"<id:[^>]+>").unwrap());

static RE_HEADER: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^\s{0,3}#{1,3}\s+").unwrap());

static RE_SUBTEXT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^\s*-#(?:\s+.*)?$").unwrap());

static RE_BLOCKQUOTE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^\s*>>?>\s?").unwrap());

static RE_LIST: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^\s*(?:[-*]|\d+\.)\s+").unwrap());

static RE_WS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

const TTS_SEGMENT_SOFT_LIMIT: usize = 36;
const TTS_SEGMENT_HARD_LIMIT: usize = 60;
const TTS_MAX_TOTAL_CHARS: usize = 200;

pub fn apply_tts_dictionary(input: &str, entries: &[(String, String)]) -> String {
    if input.is_empty() || entries.is_empty() {
        return input.to_string();
    }

    // 長いキーを先に適用して、短いキーによる過剰置換を減らす。
    let mut sorted = entries
        .iter()
        .filter(|(source, target)| !source.is_empty() && !target.is_empty())
        .collect::<Vec<_>>();
    sorted.sort_by(|a, b| b.0.chars().count().cmp(&a.0.chars().count()));

    let mut out = input.to_string();
    for (source, target) in sorted {
        out = out.replace(source, target);
    }

    out
}

pub fn split_tts_segments(input: &str) -> Vec<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for ch in trimmed.chars() {
        if matches!(ch, '\n' | '\r') {
            let chunk = current.trim();
            if !chunk.is_empty() {
                segments.push(chunk.to_string());
            }
            current.clear();
            current_len = 0;
            continue;
        }

        current.push(ch);
        current_len += 1;

        let at_sentence_boundary = matches!(
            ch,
            '。' | '！'
                | '？'
                | '!'
                | '?'
                | '、'
                | ','
                | '，'
                | '．'
                | '.'
                | '；'
                | ';'
                | '：'
                | ':'
        );
        let at_soft_boundary = matches!(ch, ' ' | '　') && current_len >= TTS_SEGMENT_SOFT_LIMIT;
        let over_hard_limit = current_len >= TTS_SEGMENT_HARD_LIMIT;

        if at_sentence_boundary || at_soft_boundary || over_hard_limit {
            let chunk = current.trim();
            if !chunk.is_empty() {
                segments.push(chunk.to_string());
            }
            current.clear();
            current_len = 0;
        }
    }

    let rest = current.trim();
    if !rest.is_empty() {
        segments.push(rest.to_string());
    }

    segments
}

pub fn build_tts_text_from_message(cache: &Cache, msg: &Message) -> Option<String> {
    // 先頭 ! / / はコマンド扱いで無視
    if msg.content.starts_with('!') || msg.content.starts_with('/') {
        return None;
    }

    if msg.flags.map(|flags| flags.contains(MessageFlags::SUPPRESS_NOTIFICATIONS)).unwrap_or(false) {
        return None;
    }

    // serenity 側で mention 系をまず人間向け文字列へ
    let safe = if let Some(guild_id) = msg.guild_id {
        let options = ContentSafeOptions::default().display_as_member_from(guild_id);
        content_safe(cache, &msg.content, &options, &msg.mentions)
    } else {
        msg.content_safe(cache)
    };

    let content = normalize_tts_text(&safe);

    let content = if content.is_empty() {
        if msg.attachments.is_empty() {
            return None;
        }
        "ファイルが送信されました".to_string()
    } else {
        content
    };

    Some(content)
}

pub fn normalize_tts_text(input: &str) -> String {
    // Discord の見た目に近い順で潰す
    // code block を spoiler より先に処理すると挙動が自然
    let mut out = input.to_string();

    // [text](url) -> text
    out = RE_MASKED_LINK.replace_all(&out, "$1").into_owned();

    // ```...``` -> コードブロック
    out = RE_CODE_BLOCK
        .replace_all(&out, "コードブロック")
        .into_owned();

    // `...` -> コード
    out = RE_INLINE_CODE.replace_all(&out, "コード").into_owned();

    // ||...|| -> ネタバレあり
    out = RE_SPOILER
        .replace_all(&out, |caps: &Captures| {
            let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if inner.is_empty() {
                "ネタバレ".to_string()
            } else {
                format!("ネタバレあり {}", inner)
            }
        })
        .into_owned();

    // </foo:123> / </foo bar:123> -> スラッシュコマンド foo / foo bar
    out = RE_SLASH_COMMAND
        .replace_all(&out, |caps: &Captures| {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if name.is_empty() {
                "スラッシュコマンド".to_string()
            } else {
                format!("スラッシュコマンド {}", name)
            }
        })
        .into_owned();

    // <:name:id> / <a:name:id> -> 絵文字 name
    out = RE_CUSTOM_EMOJI
        .replace_all(&out, |caps: &Captures| {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            if name.is_empty() {
                "絵文字".to_string()
            } else {
                format!("絵文字 {}", name)
            }
        })
        .into_owned();

    // <t:1618953630:R> -> 相対時刻 / 日時
    out = RE_TIMESTAMP
        .replace_all(&out, |caps: &Captures| {
            let ts = caps.get(1).and_then(|m| m.as_str().parse::<i64>().ok());

            let style = caps.get(2).map(|m| m.as_str());

            match ts {
                Some(ts) => format_timestamp_for_tts(ts, style),
                None => "日時".to_string(),
            }
        })
        .into_owned();

    // <id:guide> など
    out = RE_GUILD_NAV
        .replace_all(&out, "サーバー内リンク")
        .into_owned();

    // URL
    out = RE_URL.replace_all(&out, "URL").into_owned();

    // Markdown 記号のうち読み上げ不要なもの
    out = RE_HEADER.replace_all(&out, "").into_owned();
    out = RE_SUBTEXT.replace_all(&out, "").into_owned();
    out = RE_BLOCKQUOTE.replace_all(&out, "").into_owned();
    out = RE_LIST.replace_all(&out, "").into_owned();

    // 改行・連続空白を畳む
    out = RE_WS.replace_all(&out, " ").into_owned();
    out = out.trim().to_string();

    if out.chars().count() > TTS_MAX_TOTAL_CHARS {
        let truncated = out.chars().take(TTS_MAX_TOTAL_CHARS).collect::<String>();
        return format!("{}、以下省略", truncated);
    }

    out
}

fn format_timestamp_for_tts(ts: i64, style: Option<&str>) -> String {
    // Discord 自体は各ユーザーの timezone / locale で表示するが、
    // 読み上げは固定表現の方が扱いやすいので JST に寄せる
    let jst = FixedOffset::east_opt(9 * 60 * 60).unwrap();

    let Some(dt_utc) = Utc.timestamp_opt(ts, 0).single() else {
        return "日時".to_string();
    };
    let dt = dt_utc.with_timezone(&jst);

    match style.unwrap_or("f") {
        "t" => dt.format("%H時%M分").to_string(),
        "T" => dt.format("%H時%M分%S秒").to_string(),
        "d" => dt.format("%Y年%m月%d日").to_string(),
        "D" => dt.format("%Y年%m月%d日").to_string(),
        "f" => dt.format("%Y年%m月%d日 %H時%M分").to_string(),
        "F" => dt.format("%Y年%m月%d日 %H時%M分").to_string(),
        "s" => dt.format("%Y年%m月%d日 %H時%M分").to_string(),
        "S" => dt.format("%Y年%m月%d日 %H時%M分%S秒").to_string(),
        "R" => relative_time_jp(ts),
        _ => dt.format("%Y年%m月%d日 %H時%M分").to_string(),
    }
}

fn relative_time_jp(ts: i64) -> String {
    let now = Utc::now().timestamp();
    let diff = ts - now;
    let abs = diff.abs();

    let (value, unit) = if abs < 60 {
        (abs, "秒")
    } else if abs < 60 * 60 {
        (abs / 60, "分")
    } else if abs < 60 * 60 * 24 {
        (abs / (60 * 60), "時間")
    } else if abs < 60 * 60 * 24 * 30 {
        (abs / (60 * 60 * 24), "日")
    } else if abs < 60 * 60 * 24 * 365 {
        (abs / (60 * 60 * 24 * 30), "か月")
    } else {
        (abs / (60 * 60 * 24 * 365), "年")
    };

    if diff >= 0 {
        format!("{}{}後", value, unit)
    } else {
        format!("{}{}前", value, unit)
    }
}
