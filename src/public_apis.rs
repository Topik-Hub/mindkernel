use std::time::Duration;

pub struct PublicApisClient {
    client: reqwest::Client,
}

impl PublicApisClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build reqwest Client");
        Self { client }
    }

    pub async fn fetch_context(&self) -> Vec<String> {
        let mut context = Vec::new();

        if let Some(weather) = self.fetch_weather().await {
            context.push(weather);
        }
        if let Some(geo) = self.fetch_geo().await {
            context.push(geo);
        }
        if let Some(btc) = self.fetch_btc_price().await {
            context.push(btc);
        }
        context
    }

    async fn fetch_weather(&self) -> Option<String> {
        let url = "https://api.open-meteo.com/v1/forecast?latitude=55.75&longitude=37.62&current=temperature_2m,relative_humidity_2m,weather_code&timezone=auto";
        let resp = self.client.get(url).send().await.ok()?;
        let data: serde_json::Value = resp.json().await.ok()?;
        let current = data.get("current")?;
        let temp = current.get("temperature_2m")?.as_f64()?;
        let humidity = current.get("relative_humidity_2m")?.as_i64()?;
        let code = current.get("weather_code")?.as_i64()?;
        let desc = weather_description(code);
        Some(format!("Weather: {}°C, {}, humidity {}%", temp, desc, humidity))
    }

    async fn fetch_geo(&self) -> Option<String> {
        let resp = self.client.get("http://ip-api.com/json").send().await.ok()?;
        let data: serde_json::Value = resp.json().await.ok()?;
        if data.get("status")?.as_str()? == "success" {
            let city = data.get("city")?.as_str().unwrap_or("?");
            let country = data.get("country")?.as_str().unwrap_or("?");
            let ip = data.get("query")?.as_str().unwrap_or("?");
            Some(format!("Location: {}, {} (IP: {})", city, country, ip))
        } else {
            None
        }
    }

    async fn fetch_btc_price(&self) -> Option<String> {
        let resp = self.client.get("https://api.coindesk.com/v1/bpi/currentprice.json").send().await.ok()?;
        let data: serde_json::Value = resp.json().await.ok()?;
        let rate = data.get("bpi")?.get("USD")?.get("rate")?.as_str()?;
        Some(format!("BTC price: ${}", rate))
    }
}

fn weather_description(code: i64) -> &'static str {
    match code {
        0 => "Clear",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 => "Foggy",
        48 => "Depositing rime fog",
        51 => "Light drizzle",
        53 => "Moderate drizzle",
        55 => "Dense drizzle",
        56 => "Light freezing drizzle",
        61 => "Slight rain",
        63 => "Moderate rain",
        65 => "Heavy rain",
        66 => "Light freezing rain",
        71 => "Slight snow",
        73 => "Moderate snow",
        75 => "Heavy snow",
        77 => "Snow grains",
        80 => "Slight rain showers",
        81 => "Moderate rain showers",
        82 => "Violent rain showers",
        85 => "Slight snow showers",
        86 => "Heavy snow showers",
        95 => "Thunderstorm",
        96 => "Thunderstorm with slight hail",
        99 => "Thunderstorm with heavy hail",
        _ => "Unknown",
    }
}
// emergent: call public_apis entropy limit
// emergent: call public_apis threshold mastery
// emergent: merge public_apis buffer coherence
// emergent: split public_apis rate energy
// emergent: split public_apis buffer buffer
// emergent: split public_apis rate interval
// emergent: merge public_apis capacity rate
// emergent: merge public_apis rate novelty
// emergent: call public_apis mastery capacity
// emergent: set public_apis competence check
// emergent: set public_apis rate entropy
// emergent: split_merge public_apis integrity coherence
// emergent: split_merge public_apis integrity energy
// emergent: call public_apis curiosity check
// emergent: revert public_apis mastery limit
// emergent: split_merge public_apis novelty buffer
// emergent: guard public_apis check threshold
// emergent: revert public_apis entropy threshold
// emergent: set_split public_apis integrity coherence
// emergent: copy public_apis interval coherence
// emergent: revert public_apis coherence interval
// emergent: revert public_apis check rate
// emergent: merge public_apis integrity rate
// emergent: split public_apis limit coherence
// emergent: revert public_apis rate energy
// emergent: guard public_apis energy entropy
// emergent: merge public_apis competence entropy
// emergent: copy public_apis limit competence
// emergent: set_duplicate public_apis interval entropy
// emergent: duplicate_call_split public_apis buffer interval
// emergent: split_duplicate_set_temp public_apis interval rate
// emergent: merge public_apis rate threshold
// emergent: split public_apis entropy integrity
// emergent: merge public_apis energy limit
// emergent: call_split public_apis buffer rate
// emergent: call_split public_apis limit competence
// emergent: copy_merge public_apis energy interval
// emergent: copy public_apis buffer threshold
// emergent: set_temp_merge_call public_apis energy capacity
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp public_apis mastery capacity
// emergent: copy_merge public_apis integrity mastery
// emergent: call_set_temp public_apis rate mastery
// emergent: guard public_apis novelty interval
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp public_apis coherence energy
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge public_apis energy entropy
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp public_apis entropy mastery
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge public_apis interval integrity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge public_apis check mastery
// emergent: guard_set_duplicate public_apis capacity limit
// emergent: copy public_apis integrity check
// emergent: call_copy public_apis mastery integrity
// emergent: duplicate_set_temp public_apis novelty limit
// emergent: revert_copy_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge public_apis integrity novelty
// emergent: guard_merge public_apis energy curiosity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge public_apis energy interval
// emergent: copy_merge public_apis coherence curiosity
// emergent: call_split public_apis entropy energy
// emergent: merge public_apis capacity energy
// emergent: set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp public_apis buffer novelty
// emergent: copy public_apis competence threshold
