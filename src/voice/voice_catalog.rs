use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct VoiceStyleEntry {
    pub vvm_file: String,
    pub speaker_name: String,
    pub style_name: String,
    pub style_id: u32,
}

const VOICE_STYLE_TABLE: &str = r#"
| VVMファイル名 | 話者名 | スタイル名 | スタイルID |
|---|---|---|---|
| 0.vvm | 四国めたん | ノーマル | 2 |
| 0.vvm | 四国めたん | あまあま | 0 |
| 0.vvm | 四国めたん | ツンツン | 6 |
| 0.vvm | 四国めたん | セクシー | 4 |
| 0.vvm | ずんだもん | ノーマル | 3 |
| 0.vvm | ずんだもん | あまあま | 1 |
| 0.vvm | ずんだもん | ツンツン | 7 |
| 0.vvm | ずんだもん | セクシー | 5 |
| 0.vvm | 春日部つむぎ | ノーマル | 8 |
| 0.vvm | 雨晴はう | ノーマル | 10 |
| 1.vvm | 冥鳴ひまり | ノーマル | 14 |
| 2.vvm | 九州そら | ノーマル | 16 |
| 2.vvm | 九州そら | あまあま | 15 |
| 2.vvm | 九州そら | ツンツン | 18 |
| 2.vvm | 九州そら | セクシー | 17 |
| 3.vvm | 波音リツ | ノーマル | 9 |
| 3.vvm | 波音リツ | クイーン | 65 |
| 3.vvm | 中国うさぎ | ノーマル | 61 |
| 3.vvm | 中国うさぎ | おどろき | 62 |
| 3.vvm | 中国うさぎ | こわがり | 63 |
| 3.vvm | 中国うさぎ | へろへろ | 64 |
| 4.vvm | 玄野武宏 | ノーマル | 11 |
| 4.vvm | 剣崎雌雄 | ノーマル | 21 |
| 5.vvm | 四国めたん | ささやき | 36 |
| 5.vvm | 四国めたん | ヒソヒソ | 37 |
| 5.vvm | ずんだもん | ささやき | 22 |
| 5.vvm | ずんだもん | ヒソヒソ | 38 |
| 5.vvm | 九州そら | ささやき | 19 |
| 6.vvm | No.7 | ノーマル | 29 |
| 6.vvm | No.7 | アナウンス | 30 |
| 6.vvm | No.7 | 読み聞かせ | 31 |
| 7.vvm | 後鬼 | 人間ver. | 27 |
| 7.vvm | 後鬼 | ぬいぐるみver. | 28 |
| 8.vvm | WhiteCUL | ノーマル | 23 |
| 8.vvm | WhiteCUL | たのしい | 24 |
| 8.vvm | WhiteCUL | かなしい | 25 |
| 8.vvm | WhiteCUL | びえーん | 26 |
| 9.vvm | 白上虎太郎 | ふつう | 12 |
| 9.vvm | 白上虎太郎 | わーい | 32 |
| 9.vvm | 白上虎太郎 | びくびく | 33 |
| 9.vvm | 白上虎太郎 | おこ | 34 |
| 9.vvm | 白上虎太郎 | びえーん | 35 |
| 10.vvm | 玄野武宏 | 喜び | 39 |
| 10.vvm | 玄野武宏 | ツンギレ | 40 |
| 10.vvm | 玄野武宏 | 悲しみ | 41 |
| 10.vvm | ちび式じい | ノーマル | 42 |
| 11.vvm | 櫻歌ミコ | ノーマル | 43 |
| 11.vvm | 櫻歌ミコ | 第二形態 | 44 |
| 11.vvm | 櫻歌ミコ | ロリ | 45 |
| 11.vvm | ナースロボ＿タイプＴ | ノーマル | 47 |
| 11.vvm | ナースロボ＿タイプＴ | 楽々 | 48 |
| 11.vvm | ナースロボ＿タイプＴ | 恐怖 | 49 |
| 11.vvm | ナースロボ＿タイプＴ | 内緒話 | 50 |
| 12.vvm | †聖騎士 紅桜† | ノーマル | 51 |
| 12.vvm | 雀松朱司 | ノーマル | 52 |
| 12.vvm | 麒ヶ島宗麟 | ノーマル | 53 |
| 13.vvm | 春歌ナナ | ノーマル | 54 |
| 13.vvm | 猫使アル | ノーマル | 55 |
| 13.vvm | 猫使アル | おちつき | 56 |
| 13.vvm | 猫使アル | うきうき | 57 |
| 13.vvm | 猫使ビィ | ノーマル | 58 |
| 13.vvm | 猫使ビィ | おちつき | 59 |
| 13.vvm | 猫使ビィ | 人見知り | 60 |
| 14.vvm | 栗田まろん | ノーマル | 67 |
| 14.vvm | あいえるたん | ノーマル | 68 |
| 14.vvm | 満別花丸 | ノーマル | 69 |
| 14.vvm | 満別花丸 | 元気 | 70 |
| 14.vvm | 満別花丸 | ささやき | 71 |
| 14.vvm | 満別花丸 | ぶりっ子 | 72 |
| 14.vvm | 満別花丸 | ボーイ | 73 |
| 14.vvm | 琴詠ニア | ノーマル | 74 |
| 15.vvm | ずんだもん | ヘロヘロ | 75 |
| 15.vvm | ずんだもん | なみだめ | 76 |
| 15.vvm | 青山龍星 | ノーマル | 13 |
| 15.vvm | 青山龍星 | 熱血 | 81 |
| 15.vvm | 青山龍星 | 不機嫌 | 82 |
| 15.vvm | 青山龍星 | 喜び | 83 |
| 15.vvm | 青山龍星 | しっとり | 84 |
| 15.vvm | 青山龍星 | かなしみ | 85 |
| 15.vvm | 青山龍星 | 囁き | 86 |
| 15.vvm | もち子さん | ノーマル | 20 |
| 15.vvm | もち子さん | セクシー／あん子 | 66 |
| 15.vvm | もち子さん | 泣き | 77 |
| 15.vvm | もち子さん | 怒り | 78 |
| 15.vvm | もち子さん | 喜び | 79 |
| 15.vvm | もち子さん | のんびり | 80 |
| 15.vvm | 小夜/SAYO | ノーマル | 46 |
| 16.vvm | 後鬼 | 人間（怒り）ver. | 87 |
| 16.vvm | 後鬼 | 鬼ver. | 88 |
| 17.vvm | Voidoll | ノーマル | 89 |
| 18.vvm | ぞん子 | ノーマル | 90 |
| 18.vvm | ぞん子 | 低血圧 | 91 |
| 18.vvm | ぞん子 | 覚醒 | 92 |
| 18.vvm | ぞん子 | 実況風 | 93 |
| 18.vvm | 中部つるぎ | ノーマル | 94 |
| 18.vvm | 中部つるぎ | 怒り | 95 |
| 18.vvm | 中部つるぎ | ヒソヒソ | 96 |
| 18.vvm | 中部つるぎ | おどおど | 97 |
| 18.vvm | 中部つるぎ | 絶望と敗北 | 98 |
| 19.vvm | 離途 | ノーマル | 99 |
| 19.vvm | 離途 | シリアス | 101 |
| 19.vvm | 黒沢冴白 | ノーマル | 100 |
| 20.vvm | ユーレイちゃん | ノーマル | 102 |
| 20.vvm | ユーレイちゃん | 甘々 | 103 |
| 20.vvm | ユーレイちゃん | 哀しみ | 104 |
| 20.vvm | ユーレイちゃん | ささやき | 105 |
| 20.vvm | ユーレイちゃん | ツクモちゃん | 106 |
| 21.vvm | 猫使アル | つよつよ | 110 |
| 21.vvm | 猫使アル | へろへろ | 111 |
| 21.vvm | 猫使ビィ | つよつよ | 112 |
| 21.vvm | 東北ずん子 | ノーマル | 107 |
| 21.vvm | 東北きりたん | ノーマル | 108 |
| 21.vvm | 東北イタコ | ノーマル | 109 |
| 22.vvm | あんこもん | ノーマル | 113 |
| 22.vvm | あんこもん | つよつよ | 114 |
| 22.vvm | あんこもん | よわよわ | 115 |
| 22.vvm | あんこもん | けだるげ | 116 |
| 23.vvm | あんこもん | ささやき | 117 |
"#;

static VOICE_STYLE_ENTRIES: OnceLock<Vec<VoiceStyleEntry>> = OnceLock::new();

fn normalize_text(input: &str) -> String {
    input.trim().replace('　', " ").to_lowercase()
}

fn parse_voice_style_line(line: &str) -> Option<VoiceStyleEntry> {
    let line = line.trim();
    if !line.starts_with('|') {
        return None;
    }

    let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
    if columns.len() < 6 {
        return None;
    }

    let vvm_file = columns[1];
    if vvm_file.is_empty() || vvm_file == "---" || vvm_file == "VVMファイル名" {
        return None;
    }

    let speaker_name = columns[2];
    let style_name = columns[3];
    let style_id = columns[4].parse::<u32>().ok()?;

    Some(VoiceStyleEntry {
        vvm_file: vvm_file.to_string(),
        speaker_name: speaker_name.to_string(),
        style_name: style_name.to_string(),
        style_id,
    })
}

fn parse_voice_style_entries() -> Vec<VoiceStyleEntry> {
    VOICE_STYLE_TABLE
        .lines()
        .filter_map(parse_voice_style_line)
        .collect::<Vec<_>>()
}

pub fn entries() -> &'static [VoiceStyleEntry] {
    VOICE_STYLE_ENTRIES
        .get_or_init(parse_voice_style_entries)
        .as_slice()
}

pub fn speaker_names() -> Vec<String> {
    let mut out = Vec::<String>::new();
    for entry in entries() {
        if !out.iter().any(|name| name == &entry.speaker_name) {
            out.push(entry.speaker_name.clone());
        }
    }
    out
}

pub fn styles_for_speaker(speaker: &str) -> Vec<String> {
    let key = normalize_text(speaker);
    let mut out = Vec::<String>::new();

    for entry in entries() {
        if normalize_text(&entry.speaker_name) != key {
            continue;
        }
        if !out.iter().any(|style| style == &entry.style_name) {
            out.push(entry.style_name.clone());
        }
    }

    out
}

pub fn find_style_id(speaker: &str, style: &str) -> Option<u32> {
    let speaker_key = normalize_text(speaker);
    let style_key = normalize_text(style);

    entries()
        .iter()
        .find(|entry| {
            normalize_text(&entry.speaker_name) == speaker_key
                && normalize_text(&entry.style_name) == style_key
        })
        .map(|entry| entry.style_id)
}

pub fn speaker_name_for_id(style_id: u32) -> Option<String> {
    entries()
        .iter()
        .find(|entry| entry.style_id == style_id)
        .map(|entry| entry.speaker_name.clone())
}

pub fn style_name_for_id(style_id: u32) -> Option<String> {
    entries()
        .iter()
        .find(|entry| entry.style_id == style_id)
        .map(|entry| entry.style_name.clone())
}

pub fn suggest_speakers(partial: &str, limit: usize) -> Vec<String> {
    let partial_key = normalize_text(partial);

    speaker_names()
        .into_iter()
        .filter(|speaker| {
            if partial_key.is_empty() {
                return true;
            }
            let key = normalize_text(speaker);
            key.contains(&partial_key)
        })
        .take(limit)
        .collect::<Vec<_>>()
}

pub fn suggest_styles(
    partial: &str,
    speaker_filter: Option<&str>,
    limit: usize,
) -> Vec<VoiceStyleEntry> {
    let partial_key = normalize_text(partial);
    let speaker_filter = speaker_filter.map(normalize_text);

    entries()
        .iter()
        .filter(|entry| {
            if let Some(speaker_key) = &speaker_filter
                && normalize_text(&entry.speaker_name) != *speaker_key
            {
                return false;
            }

            if partial_key.is_empty() {
                return true;
            }

            let style_key = normalize_text(&entry.style_name);
            style_key.contains(&partial_key)
        })
        .take(limit)
        .cloned()
        .collect::<Vec<_>>()
}
