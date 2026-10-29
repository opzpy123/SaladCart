use crate::error::{AppError, AppResult};
use crate::models::{Project, ProviderProfile, TrendItem};
use crate::providers::{self, llm};
use reqwest::Url;
use serde_json::Value;
use std::net::IpAddr;
use std::time::Duration;

fn is_private_host(host: &str) -> bool {
    let host = host.trim_matches(['[', ']']).to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".local") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => {
            ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
        }
        Ok(IpAddr::V6(ip)) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
        }
        Err(_) => false,
    }
}

pub fn validate_source(source: &str) -> AppResult<Url> {
    let url = Url::parse(source.trim())
        .map_err(|error| AppError::Invalid(format!("invalid trend source URL: {error}")))?;
    if url.scheme() != "https" {
        return Err(AppError::Invalid("trend source must use https".into()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::Invalid(
            "trend source URL must not embed credentials".into(),
        ));
    }
    if url
        .host_str()
        .map(is_private_host)
        .unwrap_or(true)
    {
        return Err(AppError::Invalid(
            "trend source must be a public HTTPS host".into(),
        ));
    }
    Ok(url)
}

fn first_str(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    })
}

fn parse_hot(value: &Value) -> Option<i64> {
    if let Some(number) = value.as_i64() {
        return Some(number);
    }
    let raw = value.as_str()?.trim().replace(',', "");
    let (number, multiplier) = if let Some(rest) = raw.strip_suffix('万') {
        (rest, 10_000.0)
    } else if let Some(rest) = raw.strip_suffix('亿') {
        (rest, 100_000_000.0)
    } else {
        (raw.as_str(), 1.0)
    };
    number
        .parse::<f64>()
        .ok()
        .map(|value| (value * multiplier).round() as i64)
}

fn first_hot(value: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(parse_hot))
}

fn collect_arrays<'a>(value: &'a Value, depth: usize, out: &mut Vec<&'a Vec<Value>>) {
    if depth > 4 {
        return;
    }
    if let Some(array) = value.as_array() {
        if array.iter().any(Value::is_object) {
            out.push(array);
        }
        return;
    }
    if let Some(object) = value.as_object() {
        for child in object.values() {
            collect_arrays(child, depth + 1, out);
        }
    }
}

pub fn parse(value: &Value) -> Vec<TrendItem> {
    let mut arrays = Vec::new();
    collect_arrays(value, 0, &mut arrays);
    let Some(array) = arrays.into_iter().max_by_key(|array| array.len()) else {
        return Vec::new();
    };
    array
        .iter()
        .filter_map(|item| {
            let title = first_str(
                item,
                &[
                    "title",
                    "word",
                    "hotword",
                    "hot_word",
                    "name",
                    "keyword",
                    "sentence",
                    "content",
                ],
            )?;
            Some(TrendItem {
                rank: 0,
                title,
                hot: first_hot(
                    item,
                    &[
                        "hot_value",
                        "hot",
                        "hotindex",
                        "heat",
                        "score",
                        "play_count",
                        "view_count",
                        "hot_score",
                    ],
                ),
                url: first_str(item, &["url", "share_url", "link", "schema"]),
                cover: first_str(item, &["cover", "cover_url", "image", "pic"]),
            })
        })
        .take(20)
        .enumerate()
        .map(|(index, mut item)| {
            item.rank = (index + 1) as i64;
            item
        })
        .collect()
}

pub async fn fetch(source: &str, token: Option<&str>) -> AppResult<Vec<TrendItem>> {
    let url = validate_source(source)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let mut request = client.get(url).header("Accept", "application/json");
    if let Some(token) = token.map(str::trim).filter(|token| !token.is_empty()) {
        request = request.bearer_auth(token);
    }
    let response = request.send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let bytes = response.bytes().await?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(AppError::Invalid(
            "trend source response is larger than 2 MB".into(),
        ));
    }
    let value: Value = serde_json::from_slice(&bytes)?;
    let items = parse(&value);
    if items.is_empty() {
        return Err(AppError::Invalid(
            "the trend source returned no recognizable hot items".into(),
        ));
    }
    Ok(items)
}

pub async fn suggest(
    profile: &ProviderProfile,
    secret: Option<&str>,
    project: &Project,
    items: &[TrendItem],
) -> AppResult<String> {
    if profile.id.starts_with("mock-") {
        return Err(AppError::Invalid(
            "热点参考需要真实编剧模型，请先在设置中启用 DeepSeek".into(),
        ));
    }
    if items.is_empty() {
        return Err(AppError::Invalid("select at least one trend".into()));
    }
    let trends = items
        .iter()
        .take(10)
        .map(|item| {
            format!(
                "{}. {}{}",
                item.rank,
                item.title,
                item.hot
                    .map(|hot| format!("（热度 {hot}）"))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let system = "你是中文短剧/漫剧的开发顾问。只把热榜当作受众兴趣信号，不复制、不改编现有视频的具体情节、台词或角色。输出可执行、可拍摄的中文建议，不要空话。";
    let user = format!(
        "项目：{}\n立意：{}\n类型：{}\n气质：{}\n角色：{}\n\n当前抖音热门参考：\n{}\n\n请给出 5 个适合本项目的参考方向。每个方向包含：钩子（前3秒）、30秒结构、角色反转、可用标题、需要避免的照搬风险。最后给出 3 个可以立刻写进剧本的冲突点。用 Markdown 输出。",
        project.name,
        project.premise,
        project.genre,
        project.tone,
        project
            .style_bible
            .visual_style
            .chars()
            .take(80)
            .collect::<String>(),
        trends
    );
    let result = llm::complete(
        profile,
        secret,
        llm::LlmRequest {
            model: profile.default_model.clone(),
            system: system.into(),
            user,
            temperature: 0.7,
            max_tokens: 2_500,
            json: false,
        },
    )
    .await?;
    Ok(result.text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_nested_hot_lists_and_heat_suffixes() {
        let value = json!({
            "data": {"list": [
                {"word": "第一条", "hot_value": "1.2万", "url": "https://v.douyin.com/a"},
                {"title": "第二条", "hot": 3456}
            ]}
        });
        let items = parse(&value);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].rank, 1);
        assert_eq!(items[0].hot, Some(12_000));
        assert_eq!(items[1].title, "第二条");
    }

    #[test]
    fn rejects_insecure_or_private_sources() {
        assert!(validate_source("http://example.com/hot").is_err());
        assert!(validate_source("https://127.0.0.1/hot").is_err());
        assert!(validate_source("https://localhost/hot").is_err());
        assert!(validate_source("https://example.com/hot").is_ok());
    }
}
