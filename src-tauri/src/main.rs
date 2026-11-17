#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::State as AxumState,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dirs::data_dir;
use futures::StreamExt;
use parking_lot::Mutex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sysinfo::System;
use tauri::{Emitter, Manager, State};
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

// ============================================================================
// Types & State
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    model: String,
    #[serde(alias = "context_length")]
    context_length: u32,
    temperature: f32,
    #[serde(alias = "top_p")]
    top_p: f32,
    #[serde(alias = "max_tokens")]
    max_tokens: u32,
    threads: u32,
    #[serde(alias = "gpu_layers")]
    gpu_layers: u32,
    #[serde(alias = "batch_size")]
    batch_size: u32,
    theme: String,
    #[serde(alias = "auto_scroll")]
    auto_scroll: bool,
    #[serde(alias = "show_timing")]
    show_timing: bool,
    port: u16,
    #[serde(alias = "enable_api")]
    enable_api: bool,
    // Thermal/Performance Management
    #[serde(alias = "power_profile")]
    power_profile: String,
    #[serde(alias = "auto_unload_idle_minutes")]
    auto_unload_idle_minutes: u32,
    #[serde(alias = "unload_on_minimize")]
    unload_on_minimize: bool,
    #[serde(alias = "max_cpu_percent")]
    max_cpu_percent: u32,
    #[serde(alias = "thermal_throttle_enabled")]
    thermal_throttle_enabled: bool,
    #[serde(alias = "low_ram_mode")]
    low_ram_mode: bool,
    #[serde(alias = "quantize_kv_cache")]
    quantize_kv_cache: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model: String::new(),
            context_length: 4096,
            temperature: 0.2,
            top_p: 0.95,
            max_tokens: 2048,
            threads: 0,
            gpu_layers: 0,
            batch_size: 512,
            theme: "system".to_string(),
            auto_scroll: true,
            show_timing: true,
            port: 11434,
            enable_api: true,
            // Thermal defaults - laptop-friendly
            power_profile: "balanced".to_string(),
            auto_unload_idle_minutes: 10,
            unload_on_minimize: true,
            max_cpu_percent: 80, // Default cap at 80% to prevent overheating
            thermal_throttle_enabled: true,
            low_ram_mode: false,
            quantize_kv_cache: true, // Enabled by default for RAM savings
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelInfo {
    id: String,
    name: String,
    description: String,
    #[serde(alias = "size_gb")]
    size_gb: f32,
    #[serde(alias = "ram_required_gb")]
    ram_required_gb: f32,
    url: String,
    mirrors: Vec<String>,
    sha256: String,
    filename: String,
    #[serde(alias = "context_length")]
    context_length: u32,
    tags: Vec<String>,
    recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionRequest {
    prompt: String,
    #[serde(alias = "max_tokens")]
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    #[serde(alias = "top_p")]
    top_p: Option<f32>,
    stop: Option<Vec<String>>,
    stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionResponse {
    text: String,
    done: bool,
    tokens_generated: Option<u32>,
    timing: Option<TimingInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimingInfo {
    #[serde(alias = "prompt_ms")]
    prompt_ms: u64,
    #[serde(alias = "generation_ms")]
    generation_ms: u64,
    #[serde(alias = "tokens_per_second")]
    tokens_per_second: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SystemStats {
    #[serde(alias = "ram_used")]
    ram_used: u64,
    #[serde(alias = "ram_total")]
    ram_total: u64,
    #[serde(alias = "cpu_usage")]
    cpu_usage: f32,
    #[serde(alias = "cpu_temperature")]
    cpu_temperature: Option<f32>,
    #[serde(alias = "model_loaded")]
    model_loaded: bool,
    #[serde(alias = "model_name")]
    model_name: Option<String>,
    #[serde(alias = "tokens_per_second")]
    tokens_per_second: Option<f32>,
    #[serde(alias = "power_profile")]
    power_profile: String,
    #[serde(alias = "thermal_state")]
    thermal_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SystemInfo {
    #[serde(alias = "ram_gb")]
    ram_gb: u64,
    #[serde(alias = "cpu_cores")]
    cpu_cores: usize,
    gpu: String,
}

#[derive(Debug)]
struct ModelState {
    current_model: Option<String>,
    model_path: Option<PathBuf>,
    llama_process: Option<tokio::process::Child>,
    sender: Option<mpsc::UnboundedSender<String>>,
}

// Global state
struct AppState {
    settings: RwLock<Settings>,
    model_state: parking_lot::Mutex<ModelState>,
    models_dir: PathBuf,
    download_tx: broadcast::Sender<(String, f32)>, // (model_id, progress)
    api_server: parking_lot::Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    system: parking_lot::Mutex<System>,
    last_tokens_per_second: RwLock<Option<f32>>,
    thermal_monitor: parking_lot::Mutex<Option<tauri::async_runtime::JoinHandle<()>>>, // Thermal monitoring task
    idle_timer: parking_lot::Mutex<Option<tauri::async_runtime::JoinHandle<()>>>, // Auto-unload timer
}

impl AppState {
    fn new(models_dir: PathBuf) -> Self {
        // Bounded channels to prevent memory leaks
        let (download_tx, _) = broadcast::channel(16);

        Self {
            settings: RwLock::new(Settings::default()),
            model_state: Mutex::new(ModelState {
                current_model: None,
                model_path: None,
                llama_process: None,
                sender: None,
            }),
            models_dir,
            download_tx,
            api_server: Mutex::new(None),
            system: parking_lot::Mutex::new(System::new_all()),
            last_tokens_per_second: RwLock::new(None),
            thermal_monitor: Mutex::new(None),
            idle_timer: Mutex::new(None),
        }
    }
}

// ============================================================================
// Model Registry
// ============================================================================

fn get_model_registry() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "codeqwen-1.5b".to_string(),
            name: "CodeQwen 1.5B".to_string(),
            description: "Fast, efficient code model from Alibaba. Great for Python, JS, TS, Java.".to_string(),
            size_gb: 1.1,
            ram_required_gb: 3.0,
            url: "https://huggingface.co/Qwen/CodeQwen1.5-1.5B-GGUF/resolve/main/codeqwen1.5-1.5b-q4_k_m.gguf".to_string(),
            mirrors: vec![
                "https://hf-mirror.com/Qwen/CodeQwen1.5-1.5B-GGUF/resolve/main/codeqwen1.5-1.5b-q4_k_m.gguf".to_string(),
                "https://mirror.ghproxy.com/https://huggingface.co/Qwen/CodeQwen1.5-1.5B-GGUF/resolve/main/codeqwen1.5-1.5b-q4_k_m.gguf".to_string(),
            ],
            sha256: "".to_string(),
            filename: "codeqwen1.5-1.5b-q4_k_m.gguf".to_string(),
            context_length: 8192,
            tags: vec!["code".to_string(), "fast".to_string(), "low-ram".to_string()],
            recommended: true,
        },
        ModelInfo {
            id: "deepseek-coder-1.3b".to_string(),
            name: "DeepSeek Coder 1.3B".to_string(),
            description: "Strong code completion from DeepSeek. Excellent for infilling.".to_string(),
            size_gb: 0.9,
            ram_required_gb: 2.5,
            url: "https://huggingface.co/deepseek-ai/deepseek-coder-1.3b-instruct-GGUF/resolve/main/deepseek-coder-1.3b-instruct-q4_k_m.gguf".to_string(),
            mirrors: vec![
                "https://hf-mirror.com/deepseek-ai/deepseek-coder-1.3b-instruct-GGUF/resolve/main/deepseek-coder-1.3b-instruct-q4_k_m.gguf".to_string(),
                "https://mirror.ghproxy.com/https://huggingface.co/deepseek-ai/deepseek-coder-1.3b-instruct-GGUF/resolve/main/deepseek-coder-1.3b-instruct-q4_k_m.gguf".to_string(),
            ],
            sha256: "".to_string(),
            filename: "deepseek-coder-1.3b-instruct-q4_k_m.gguf".to_string(),
            context_length: 16384,
            tags: vec!["code".to_string(), "infilling".to_string(), "low-ram".to_string()],
            recommended: true,
        },
        ModelInfo {
            id: "phi-3-mini".to_string(),
            name: "Phi-3 Mini 4K".to_string(),
            description: "Microsoft's small but mighty model. Best reasoning in class.".to_string(),
            size_gb: 2.3,
            ram_required_gb: 4.0,
            url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf".to_string(),
            mirrors: vec![
                "https://hf-mirror.com/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf".to_string(),
                "https://mirror.ghproxy.com/https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf".to_string(),
            ],
            sha256: "".to_string(),
            filename: "phi-3-mini-4k-instruct-q4_k_m.gguf".to_string(),
            context_length: 4096,
            tags: vec!["reasoning".to_string(), "chat".to_string(), "general".to_string()],
            recommended: true,
        },
        ModelInfo {
            id: "stable-code-3b".to_string(),
            name: "Stable Code 3B".to_string(),
            description: "Stability AI's code model. Good balance of size and capability.".to_string(),
            size_gb: 1.8,
            ram_required_gb: 3.5,
            url: "https://huggingface.co/stabilityai/stable-code-3b-GGUF/resolve/main/stable-code-3b-q4_k_m.gguf".to_string(),
            mirrors: vec![
                "https://hf-mirror.com/stabilityai/stable-code-3b-GGUF/resolve/main/stable-code-3b-q4_k_m.gguf".to_string(),
                "https://mirror.ghproxy.com/https://huggingface.co/stabilityai/stable-code-3b-GGUF/resolve/main/stable-code-3b-q4_k_m.gguf".to_string(),
            ],
            sha256: "".to_string(),
            filename: "stable-code-3b-q4_k_m.gguf".to_string(),
            context_length: 16384,
            tags: vec!["code".to_string(), "fill-in-middle".to_string()],
            recommended: false,
        },
    ]
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    Ok(state.settings.read().await.clone())
}

#[tauri::command]
async fn save_settings(state: State<'_, Arc<AppState>>, settings: Settings) -> Result<(), String> {
    *state.settings.write().await = settings.clone();

    // Restart API server if port changed
    if settings.enable_api {
        restart_api_server(&state, settings.port).await?;
    } else {
        stop_api_server(&state).await?;
    }

    // Save to disk
    save_settings_to_disk(state.inner(), &settings).await?;

    Ok(())
}

#[tauri::command]
async fn get_system_info(state: State<'_, Arc<AppState>>) -> Result<SystemInfo, String> {
    let mut sys = state.system.lock();
    sys.refresh_memory();
    sys.refresh_cpu_usage();

    let ram_gb = sys.total_memory() / 1024 / 1024 / 1024;
    let cpu_cores = sys.cpus().len();

    // Try to detect GPU (simplified)
    let gpu = detect_gpu().unwrap_or_else(|| "None detected".to_string());

    Ok(SystemInfo {
        ram_gb,
        cpu_cores,
        gpu,
    })
}

#[tauri::command]
async fn get_system_stats(state: State<'_, Arc<AppState>>) -> Result<SystemStats, String> {
    let state_arc = state.inner().clone();

    // Drop all locks before any await to keep the future Send.
    let (ram_used, ram_total, cpu_usage, cpu_temperature) = {
        let mut sys = state_arc.system.lock();
        sys.refresh_memory();
        sys.refresh_cpu_usage();
        (
            sys.used_memory(),
            sys.total_memory(),
            sys.global_cpu_info().cpu_usage(),
            get_cpu_temperature(),
        )
    };

    let (model_loaded, model_name) = {
        let model_state = state_arc.model_state.lock();
        (
            model_state.current_model.is_some(),
            model_state.current_model.clone(),
        )
    };

    let tokens_per_second = *state_arc.last_tokens_per_second.read().await;
    let settings = state_arc.settings.read().await.clone();
    let thermal_state = determine_thermal_state(cpu_temperature.unwrap_or(0.0), cpu_usage);

    Ok(SystemStats {
        ram_used,
        ram_total,
        cpu_usage,
        cpu_temperature,
        model_loaded,
        model_name,
        tokens_per_second,
        power_profile: settings.power_profile,
        thermal_state,
    })
}

#[tauri::command]
async fn check_internet(_state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    match reqwest::get("https://huggingface.co").await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn get_system_theme(_state: State<'_, Arc<AppState>>) -> Result<String, String> {
    // Simplified - in real app, use system theme detection
    Ok("light".to_string())
}

#[tauri::command]
async fn list_downloaded_models(state: State<'_, Arc<AppState>>) -> Result<Vec<String>, String> {
    list_downloaded_models_internal(state.inner()).await
}

async fn list_downloaded_models_internal(state: &Arc<AppState>) -> Result<Vec<String>, String> {
    let mut models = Vec::new();
    if state.models_dir.exists() {
        for entry in std::fs::read_dir(&state.models_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().is_some_and(|ext| ext == "gguf") {
                if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    models.push(name.to_string());
                }
            }
        }
    }
    Ok(models)
}

#[tauri::command]
async fn download_model(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
    model_id: String,
    urls: Vec<String>,
    filename: String,
    expected_sha256: String,
) -> Result<(), String> {
    if model_id.trim().is_empty() || filename.trim().is_empty() || urls.is_empty() {
        return Err("Invalid model download details.".to_string());
    }

    let models_dir = state.models_dir.clone();
    let download_tx = state.download_tx.clone();
    let mut progress_rx = download_tx.subscribe();
    let app_handle = app.clone();
    let started_id = model_id.clone();
    app.emit("model:download_started", serde_json::json!({ "modelId": started_id }))
        .map_err(|e| format!("Could not start download UI: {e}"))?;

    tokio::spawn(async move {
        let download = download_model_internal(
            &models_dir, &model_id, &urls, &filename, &expected_sha256, &download_tx,
        );
        tokio::pin!(download);

        loop {
            tokio::select! {
                result = &mut download => {
                    match result {
                        Ok(()) => {
                            let _ = app_handle.emit("model:download_progress", serde_json::json!({
                                "modelId": model_id, "progress": 100.0
                            }));
                            let _ = app_handle.emit("model:download_complete", serde_json::json!({
                                "modelId": model_id, "success": true
                            }));
                        }
                        Err(err) => {
                            error!("Download failed for {}: {:#}", model_id, err);
                            let _ = app_handle.emit("model:download_error", serde_json::json!({
                                "modelId": model_id, "error": format!("{err:#}")
                            }));
                            let _ = app_handle.emit("model:download_complete", serde_json::json!({
                                "modelId": model_id, "success": false
                            }));
                        }
                    }
                    break;
                }
                event = progress_rx.recv() => {
                    match event {
                        Ok((event_model_id, progress)) if event_model_id == model_id && progress >= 0.0 => {
                            let _ = app_handle.emit("model:download_progress", serde_json::json!({
                                "modelId": event_model_id, "progress": progress
                            }));
                        }
                        Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }
    });

    Ok(())
}

async fn download_model_internal(
    models_dir: &Path,
    model_id: &str,
    urls: &[String],
    filename: &str,
    expected_sha256: &str,
    download_tx: &broadcast::Sender<(String, f32)>,
) -> Result<()> {
    std::fs::create_dir_all(models_dir).context("Create models dir")?;

    let client = Client::new();
    let file_path = models_dir.join(filename);

    // Try each URL (primary + mirrors) until one works
    let mut last_error = None;
    for (idx, url) in urls.iter().enumerate() {
        info!("Trying download from mirror {}: {}", idx + 1, url);
        let _ = download_tx.send((model_id.to_string(), -2.0 - idx as f32)); // Signal which mirror

        match try_download_from_url(
            &client,
            url,
            &file_path,
            expected_sha256,
            model_id,
            download_tx,
        )
        .await
        {
            Ok(()) => {
                info!("Download complete from mirror {}: {}", idx + 1, model_id);
                let _ = download_tx.send((model_id.to_string(), 100.0));
                return Ok(());
            }
            Err(e) => {
                warn!("Mirror {} failed: {}", idx + 1, e);
                last_error = Some(e);
                // Clean up partial file
                let _ = std::fs::remove_file(&file_path);
                // Small delay before trying next mirror
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }

    // All mirrors failed
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All download mirrors failed")))
}

async fn try_download_from_url(
    client: &Client,
    url: &str,
    file_path: &Path,
    expected_sha256: &str,
    model_id: &str,
    download_tx: &broadcast::Sender<(String, f32)>,
) -> Result<()> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Request failed for {url}"))?
        .error_for_status()
        .with_context(|| format!("Model server returned an HTTP error for {url}"))?;
    let total_size = response.content_length().unwrap_or(0) as f32;

    let mut file = tokio::fs::File::create(file_path)
        .await
        .context("Create file")?;
    let mut hasher = Sha256::new();
    let mut downloaded: f32 = 0.0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Read chunk")?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .context("Write chunk")?;
        hasher.update(&chunk);
        downloaded += chunk.len() as f32;

        if total_size > 0.0 {
            let progress = (downloaded / total_size) * 100.0;
            let _ = download_tx.send((model_id.to_string(), progress));
        }
    }

    file.flush().await.context("Flush file")?;

    // Verify checksum (only if provided and not empty)
    if !expected_sha256.is_empty() {
        let actual = hex::encode(hasher.finalize());
        if actual != expected_sha256 {
            std::fs::remove_file(file_path).ok();
            anyhow::bail!(
                "Checksum mismatch: expected {}, got {}",
                expected_sha256,
                actual
            );
        }
    }

    Ok(())
}

#[tauri::command]
async fn delete_model(state: State<'_, Arc<AppState>>, model_id: String) -> Result<(), String> {
    let state_arc = state.inner().clone();

    // Find actual filename
    let registry = get_model_registry();
    let model_info = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or("Model not in registry")?;
    let file_path = state_arc.models_dir.join(&model_info.filename);

    // Unload if currently loaded
    let is_loaded = {
        let ms = state_arc.model_state.lock();
        ms.current_model.as_deref() == Some(&model_id)
    };
    if is_loaded {
        unload_model_internal(state_arc.clone()).await?;
    }

    // Delete file
    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Model file not found on disk".to_string())
    }
}

#[tauri::command]
async fn load_model(state: State<'_, Arc<AppState>>, model_id: String) -> Result<(), String> {
    // Extract Arc early to avoid Send issues
    let state_arc = state.inner().clone();

    // Unload current model first
    unload_model_internal(state_arc.clone()).await?;

    // Find model file - get path without holding lock
    let registry = get_model_registry();
    let model_info = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or("Model not in registry")?;
    let model_path = state_arc.models_dir.join(&model_info.filename);

    if !model_path.exists() {
        return Err("Model file not found. Download it first.".to_string());
    }

    // Get settings - clone to avoid holding lock across await
    let settings = state_arc.settings.read().await.clone();

    // Start llama.cpp server process
    let (tx, _rx) = mpsc::unbounded_channel::<String>();

    let llama_bin =
        find_llama_binary().ok_or("llama-server not found. Build llama.cpp or install Ollama.")?;

    let mut cmd = tokio::process::Command::new(&llama_bin);
    cmd.arg("-m")
        .arg(&model_path)
        .arg("-c")
        .arg(settings.context_length.to_string())
        .arg("--temp")
        .arg(settings.temperature.to_string())
        .arg("--top-p")
        .arg(settings.top_p.to_string())
        .arg("-t")
        .arg(if settings.threads > 0 {
            settings.threads.to_string()
        } else {
            "0".to_string()
        })
        .arg("-ngl")
        .arg(settings.gpu_layers.to_string())
        .arg("-b")
        .arg(settings.batch_size.to_string())
        .arg("--port")
        .arg(settings.port.to_string())
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--embedding") // Enable embeddings endpoint
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start llama-server: {}", e))?;
    let _child_pid = child.id();

    // Monitor stdout/stderr
    if let Some(stdout) = child.stdout.take() {
        let state_arc_stdout = state_arc.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                debug!("llama-server: {}", line);
                if line.contains("tokens per second") || line.contains("tok/s") {
                    if let Some(tps) = parse_tokens_per_second(&line) {
                        *state_arc_stdout.last_tokens_per_second.write().await = Some(tps);
                    }
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let _state_arc_stderr = state_arc.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                debug!("llama-server: {}", line);
            }
        });
    }

    // Wait a bit for server to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test if server is responding
    let client = Client::new();
    let test_url = format!("http://127.0.0.1:{}/health", settings.port);
    let mut healthy = false;
    for _ in 0..10 {
        if client.get(&test_url).send().await.is_ok() {
            healthy = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    if !healthy {
        child.kill().await.ok();
        return Err("llama-server failed to start".to_string());
    }

    // Update state - acquire lock only for state update
    {
        let mut model_state = state_arc.model_state.lock();
        model_state.current_model = Some(model_id.clone());
        model_state.model_path = Some(model_path);
        model_state.llama_process = Some(child);
        model_state.sender = Some(tx);
    }

    info!("Model loaded: {}", model_id);
    Ok(())
}

#[tauri::command]
async fn unload_model(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    unload_model_internal(state.inner().clone()).await
}

async fn unload_model_internal(state_arc: Arc<AppState>) -> Result<(), String> {
    // Kill the llama process first (if any), releasing the lock before the await.
    let process = {
        let mut model_state = state_arc.model_state.lock();
        model_state.llama_process.take()
    };
    if let Some(mut process) = process {
        process.kill().await.ok();
    }

    {
        let mut model_state = state_arc.model_state.lock();
        model_state.current_model = None;
        model_state.model_path = None;
        model_state.sender = None;
    }
    *state_arc.last_tokens_per_second.write().await = None;

    info!("Model unloaded");
    Ok(())
}

#[tauri::command]
async fn generate_completion(
    state: State<'_, Arc<AppState>>,
    request: CompletionRequest,
) -> Result<CompletionResponse, String> {
    let settings = state.settings.read().await.clone();
    let port = settings.port;

    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/completion", port);
    let stream = request.stream.unwrap_or(true);

    let payload = serde_json::json!({
        "prompt": request.prompt,
        "n_predict": request.max_tokens.unwrap_or(settings.max_tokens),
        "temperature": request.temperature.unwrap_or(settings.temperature),
        "top_p": request.top_p.unwrap_or(settings.top_p),
        "stop": request.stop.unwrap_or_default(),
        "stream": stream,
    });

    let start = Instant::now();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}. Is a model loaded?", e))?;

    if !response.status().is_success() {
        return Err(format!("Server error: {}", response.status()));
    }

    let elapsed = start.elapsed().as_millis() as u64;

    if !stream {
        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let text = json["content"].as_str().unwrap_or("").to_string();
        return Ok(CompletionResponse {
            text,
            done: true,
            tokens_generated: json["tokens_predicted"].as_u64().map(|v| v as u32),
            timing: Some(TimingInfo {
                prompt_ms: 0,
                generation_ms: elapsed,
                tokens_per_second: 0.0,
            }),
        });
    }

    // Stream: collect the full response from llama-server (SSE) and return it.
    // The frontend receives the complete text via the invoke result.
    use futures::StreamExt;
    let mut body_stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut full_text = String::new();
    let mut tokens_predicted: Option<u32> = None;

    while let Some(chunk) = body_stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim_end_matches('\r').to_string();
            buffer = buffer[pos + 1..].to_string();
            if !line.starts_with("data:") {
                continue;
            }
            let data = line[5..].trim();
            if data == "[DONE]" {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(content) = val["content"].as_str() {
                    full_text.push_str(content);
                }
                if val["stop"].as_bool() == Some(true) {
                    tokens_predicted = val["tokens_predicted"].as_u64().map(|v| v as u32);
                }
            }
        }
    }

    let generation_ms = start.elapsed().as_millis() as u64;
    let tps = if generation_ms > 0 {
        (full_text.split_whitespace().count() as f32) / (generation_ms as f32 / 1000.0)
    } else {
        0.0
    };
    *state.last_tokens_per_second.write().await = Some(tps);

    Ok(CompletionResponse {
        text: full_text,
        done: true,
        tokens_generated: tokens_predicted,
        timing: Some(TimingInfo {
            prompt_ms: 0,
            generation_ms,
            tokens_per_second: tps,
        }),
    })
}

#[tauri::command]
async fn stop_generation(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    let port = settings.port;

    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/stop", port);
    client.post(&url).send().await.ok();

    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

fn find_llama_binary() -> Option<PathBuf> {
    // Check common locations
    let candidates = vec![
        "llama-server",
        "llama-server.exe",
        "./llama.cpp/build/bin/llama-server",
        "./llama.cpp/build/bin/Release/llama-server.exe",
        "C:/Program Files/llama.cpp/llama-server.exe",
        "/usr/local/bin/llama-server",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
        // Also check in PATH
        if which::which(candidate).is_ok() {
            return Some(path);
        }
    }

    None
}

fn parse_tokens_per_second(line: &str) -> Option<f32> {
    // Parse lines like: "llama_perf_context_print:        eval time =  1234.56 ms / 512 tokens =  415.23 tok/s"
    if let Some(idx) = line.find("tok/s") {
        let before = &line[..idx];
        if let Some(last_space) = before.rfind(' ') {
            let num_str = &before[last_space + 1..];
            return num_str.parse().ok();
        }
    }
    None
}

fn detect_gpu() -> Option<String> {
    // Simplified GPU detection
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("wmic")
            .args(["path", "win32_VideoController", "get", "name"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let line = line.trim();
                if !line.is_empty() && !line.contains("Microsoft") {
                    return Some(line.to_string());
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("lspci").args(["-nn"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.to_lowercase().contains("vga")
                    || line.to_lowercase().contains("3d")
                    || line.to_lowercase().contains("nvidia")
                    || line.to_lowercase().contains("amd")
                {
                    return Some(line.trim().to_string());
                }
            }
        }
    }
    None
}

async fn save_settings_to_disk(_state: &Arc<AppState>, settings: &Settings) -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("No config dir")?.join("aimitra");
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let settings_path = config_dir.join("settings.json");
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    tokio::fs::write(settings_path, json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn load_settings_from_disk(state: &Arc<AppState>) -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("No config dir")?.join("aimitra");
    let settings_path = config_dir.join("settings.json");
    if settings_path.exists() {
        let json = tokio::fs::read_to_string(settings_path)
            .await
            .map_err(|e| e.to_string())?;
        let settings: Settings = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        *state.settings.write().await = settings;
    }
    Ok(())
}

async fn restart_api_server(state: &State<'_, Arc<AppState>>, port: u16) -> Result<(), String> {
    stop_api_server(state).await?;

    if !state.settings.read().await.enable_api {
        return Ok(());
    }

    let state_clone = state.inner().clone();
    let handle = tauri::async_runtime::spawn(async move {
        run_api_server(state_clone, port).await;
    });

    *state.api_server.lock() = Some(handle);
    Ok(())
}

async fn stop_api_server(state: &Arc<AppState>) -> Result<(), String> {
    let mut guard = state.api_server.lock();
    if let Some(handle) = guard.take() {
        handle.abort();
    }
    Ok(())
}

async fn run_api_server(state: Arc<AppState>, port: u16) {
    let app = Router::new()
        .route("/health", get(health_check))
        .route(
            "/v1/completions",
            post(completions_handler).options(cors_preflight),
        )
        .route("/v1/models", get(models_handler).options(cors_preflight))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    info!("Starting API server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind API server: {}", e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("API server error: {}", e);
    }
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn completions_handler(
    AxumState(state): AxumState<Arc<AppState>>,
    Json(req): Json<CompletionRequest>,
) -> impl IntoResponse {
    let settings = state.settings.read().await.clone();
    let port = settings.port;

    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/completion", port);

    let payload = serde_json::json!({
        "prompt": req.prompt,
        "n_predict": req.max_tokens.unwrap_or(settings.max_tokens),
        "temperature": req.temperature.unwrap_or(settings.temperature),
        "top_p": req.top_p.unwrap_or(settings.top_p),
        "stop": req.stop.unwrap_or_default(),
        "stream": req.stream.unwrap_or(false),
    });

    match client.post(&url).json(&payload).send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();

            // Check if streaming response
            let is_streaming = headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|ct| ct.contains("text/event-stream"))
                .unwrap_or(false);

            if is_streaming {
                // Stream the response body
                let stream = resp.bytes_stream();
                let mut response_builder = Response::builder().status(status);

                // Copy relevant headers
                for (key, value) in headers.iter() {
                    if key == "content-type" || key == "cache-control" || key == "connection" {
                        response_builder = response_builder.header(key, value);
                    }
                }
                // Ensure CORS headers
                response_builder = response_builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                    .header(
                        "Access-Control-Allow-Headers",
                        "Content-Type, Authorization",
                    );

                response_builder
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to create streaming response",
                        )
                            .into_response()
                    })
            } else {
                // Non-streaming: buffer and return
                let body = resp.bytes().await.unwrap_or_default();
                let mut response_builder = Response::builder().status(status);
                response_builder = response_builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                    .header(
                        "Access-Control-Allow-Headers",
                        "Content-Type, Authorization",
                    );
                response_builder.body(Body::from(body)).unwrap_or_else(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to create response",
                    )
                        .into_response()
                })
            }
        }
        Err(e) => {
            let response = Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Access-Control-Allow-Origin", "*");
            response
                .body(Body::from(e.to_string()))
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Gateway error").into_response()
                })
        }
    }
}

async fn models_handler(AxumState(state): AxumState<Arc<AppState>>) -> impl IntoResponse {
    let models = get_model_registry();
    let downloaded = list_downloaded_models_internal(&state)
        .await
        .unwrap_or_default();

    let model_list: Vec<serde_json::Value> = models
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "object": "model",
                "owned_by": "local",
                "downloaded": downloaded.contains(&m.id),
            })
        })
        .collect();

    Json(serde_json::json!({ "object": "list", "data": model_list }))
}

// CORS preflight handler
async fn cors_preflight() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::ACCESS_CONTROL_ALLOW_METHODS, "POST, GET, OPTIONS"),
            (
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                "Content-Type, Authorization",
            ),
            (header::ACCESS_CONTROL_MAX_AGE, "86400"),
        ],
    )
}

// ============================================================================
// Thermal & Power Management
// ============================================================================

async fn thermal_monitor_loop(state: Arc<AppState>, handle: tauri::AppHandle) {
    let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds
                                                         // Note: a full idle timer with accurate wall-clock tracking can be added later.
                                                         // The idle_monitor_loop handles auto-unload based on generation activity.
    loop {
        interval.tick().await;

        // Get current settings
        let settings = state.settings.read().await.clone();
        if !settings.thermal_throttle_enabled {
            continue;
        }

        // Check CPU temperature and usage
        let (cpu_usage, cpu_temp) = {
            let mut sys = state.system.lock();
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            (
                sys.global_cpu_info().cpu_usage(),
                get_cpu_temperature().unwrap_or(0.0),
            )
        };

        // Determine thermal state
        let thermal_state = determine_thermal_state(cpu_temp, cpu_usage);
        let power_profile = settings.power_profile.clone();

        // Apply thermal throttling if needed
        if thermal_state == "throttling" || thermal_state == "hot" {
            apply_thermal_throttle(&state, &settings, thermal_state.clone()).await;
            // Under thermal load, treat as active to avoid conflicting idle unload logic.
        } else {
            // Check for idle state (no generation activity)
            // Track generation activity (future: use for finer-grained idle timing)
            let _ = *state.last_tokens_per_second.read().await;
        }

        // Emit thermal state to frontend
        let _ = handle.emit(
            "thermal:state",
            serde_json::json!({
                "thermal_state": thermal_state,
                "cpu_temp": cpu_temp,
                "cpu_usage": cpu_usage,
                "power_profile": power_profile,
            }),
        );
    }
}

async fn idle_monitor_loop(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds

    loop {
        interval.tick().await;

        let settings = state.settings.read().await.clone();
        if settings.auto_unload_idle_minutes == 0 {
            continue; // Disabled
        }

        let tps = *state.last_tokens_per_second.read().await;
        let is_idle = tps.is_none() || tps == Some(0.0);

        if is_idle {
            let _idle_duration = settings.auto_unload_idle_minutes as u64 * 60;
            // In a real implementation, we'd track actual idle time
            // For now, we'll check if model is loaded but no activity
            let model_loaded = {
                let model_state = state.model_state.lock();
                model_state.current_model.is_some()
            };
            if model_loaded {
                info!(
                    "Auto-unloading model due to idle timeout ({} min)",
                    settings.auto_unload_idle_minutes
                );
                // Inline unload without Tauri State wrapper - drop locks before await
                let process = {
                    let mut ms = state.model_state.lock();
                    ms.current_model = None;
                    ms.model_path = None;
                    ms.sender = None;
                    ms.llama_process.take()
                };
                if let Some(mut process) = process {
                    let _ = process.start_kill();
                }
                *state.last_tokens_per_second.write().await = None;
            }
        }
    }
}

fn get_cpu_temperature() -> Option<f32> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // Try WMI for CPU temperature
        if let Ok(output) = Command::new("wmic")
            .args([
                "/namespace:\\\\root\\wmi",
                "path",
                "MSAcpi_ThermalZoneTemperature",
                "get",
                "CurrentTemperature",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let line = line.trim();
                if let Ok(temp_k) = line.parse::<f32>() {
                    // Convert from tenths of Kelvin to Celsius
                    return Some((temp_k / 10.0) - 273.15);
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        // Read from thermal zone
        if let Ok(temp_str) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
            if let Ok(temp_millic) = temp_str.trim().parse::<f32>() {
                return Some(temp_millic / 1000.0);
            }
        }
    }
    None
}

fn determine_thermal_state(cpu_temp: f32, cpu_usage: f32) -> String {
    // Thermal state logic
    if cpu_temp > 90.0 || cpu_usage > 95.0 {
        "throttling"
    } else if cpu_temp > 80.0 || cpu_usage > 85.0 {
        "hot"
    } else if cpu_temp > 70.0 || cpu_usage > 70.0 {
        "warm"
    } else {
        "normal"
    }
    .to_string()
}

async fn apply_thermal_throttle(
    state: &Arc<AppState>,
    _settings: &Settings,
    thermal_state: String,
) {
    let mut model_state = state.model_state.lock();

    if let Some(process) = model_state.llama_process.as_mut() {
        match thermal_state.as_str() {
            "throttling" => {
                // Severe throttling: reduce threads, pause generation
                #[cfg(target_os = "windows")]
                {
                    use std::process::Command;
                    // Lower process priority
                    let _ = Command::new("wmic")
                        .args([
                            "process",
                            "where",
                            &format!("ProcessId={}", process.id().unwrap_or(0)),
                            "call",
                            "setpriority",
                            "16384",
                        ]) // BELOW_NORMAL_PRIORITY_CLASS
                        .output();
                }
                #[cfg(target_os = "linux")]
                {
                    use std::process::Command;
                    let _ = Command::new("renice")
                        .args(["10", "-p", &process.id().unwrap_or(0).to_string()])
                        .output();
                }
            }
            "hot" => {
                // Moderate: reduce CPU affinity or priority slightly
                #[cfg(target_os = "windows")]
                {
                    use std::process::Command;
                    let _ = Command::new("wmic")
                        .args([
                            "process",
                            "where",
                            &format!("ProcessId={}", process.id().unwrap_or(0)),
                            "call",
                            "setpriority",
                            "32768",
                        ]) // NORMAL_PRIORITY_CLASS
                        .output();
                }
            }
            _ => {}
        }
    }
}

async fn apply_power_profile_settings(state: &Arc<AppState>, profile: &str) {
    let mut settings = state.settings.write().await;

    match profile {
        "battery_saver" => {
            settings.max_cpu_percent = 50;
            settings.gpu_layers = 0;
            settings.batch_size = 128;
            settings.threads = 2;
            settings.quantize_kv_cache = true;
            settings.low_ram_mode = true;
        }
        "balanced" => {
            settings.max_cpu_percent = 80;
            settings.batch_size = 512;
            settings.threads = 0; // auto
            settings.quantize_kv_cache = true;
            settings.low_ram_mode = false;
        }
        "performance" => {
            settings.max_cpu_percent = 0; // unlimited
            settings.batch_size = 2048;
            settings.threads = 0; // auto
            settings.quantize_kv_cache = true;
            settings.low_ram_mode = false;
        }
        _ => {}
    }
}

// Tauri commands for thermal management
#[tauri::command]
async fn set_power_profile(state: State<'_, Arc<AppState>>, profile: String) -> Result<(), String> {
    let valid_profiles = ["battery_saver", "balanced", "performance"];
    if !valid_profiles.contains(&profile.as_str()) {
        return Err("Invalid power profile".to_string());
    }

    apply_power_profile_settings(&state, &profile).await;

    // Save to disk
    let settings = state.settings.read().await.clone();
    save_settings_to_disk(state.inner(), &settings).await?;

    info!("Power profile changed to: {}", profile);
    Ok(())
}

#[tauri::command]
async fn get_thermal_state(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let state_arc = state.inner().clone();

    // Drop the lock before any await to avoid Send issues
    let cpu_usage = {
        let mut sys = state_arc.system.lock();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        sys.global_cpu_info().cpu_usage()
    };
    let cpu_temp = get_cpu_temperature().unwrap_or(0.0);
    let thermal_state = determine_thermal_state(cpu_temp, cpu_usage);
    let settings = state_arc.settings.read().await.clone();

    Ok(serde_json::json!({
        "thermal_state": thermal_state,
        "cpu_temp": cpu_temp,
        "cpu_usage": cpu_usage,
        "power_profile": settings.power_profile,
        "max_cpu_percent": settings.max_cpu_percent,
    }))
}

#[tauri::command]
async fn set_thermal_throttle(
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), String> {
    let mut settings = state.settings.write().await;
    settings.thermal_throttle_enabled = enabled;
    drop(settings);

    let settings = state.settings.read().await.clone();
    save_settings_to_disk(state.inner(), &settings).await?;

    Ok(())
}

#[tauri::command]
async fn set_auto_unload(state: State<'_, Arc<AppState>>, minutes: u32) -> Result<(), String> {
    let mut settings = state.settings.write().await;
    settings.auto_unload_idle_minutes = minutes;
    drop(settings);

    let settings = state.settings.read().await.clone();
    save_settings_to_disk(state.inner(), &settings).await?;

    Ok(())
}

#[tauri::command]
async fn set_unload_on_minimize(
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), String> {
    let mut settings = state.settings.write().await;
    settings.unload_on_minimize = enabled;
    drop(settings);

    let settings = state.settings.read().await.clone();
    save_settings_to_disk(state.inner(), &settings).await?;

    Ok(())
}

#[tauri::command]
async fn set_max_cpu_percent(state: State<'_, Arc<AppState>>, percent: u32) -> Result<(), String> {
    let state_arc = state.inner().clone();
    let percent = percent.min(100);
    {
        let mut settings = state_arc.settings.write().await;
        settings.max_cpu_percent = percent;
        drop(settings);

        let settings = state_arc.settings.read().await.clone();
        save_settings_to_disk(&state_arc, &settings).await?;
    }

    // Apply to running llama process if any
    let pid = {
        let model_state = state_arc.model_state.lock();
        model_state.llama_process.as_ref().and_then(|p| p.id())
    };
    if let Some(pid) = pid {
        apply_cpu_limit(pid, percent).await;
    }

    Ok(())
}

async fn apply_cpu_limit(pid: u32, max_percent: u32) {
    if max_percent == 0 || max_percent >= 100 {
        return; // No limit
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // Use Process Governor or similar for CPU limiting
        // This is a simplified approach - in production use proper CPU limiting
        let _ = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Get-Process -Id {} | ForEach-Object {{ $_.PriorityClass = 'BelowNormal' }}",
                    pid
                ),
            ])
            .output();
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let _ = Command::new("cpulimit")
            .args(["-p", &pid.to_string(), "-l", &max_percent.to_string(), "-z"])
            .output();
    }
}

// ============================================================================
// Main
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let models_dir = data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aimitra")
        .join("models");

    let app_state = Arc::new(AppState::new(models_dir.clone()));

    // Load settings from disk (best-effort; the app can run with defaults)
    let state_clone = app_state.clone();
    std::thread::spawn(move || {
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            let _ = rt.block_on(load_settings_from_disk(&state_clone));
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .setup(|app| {
            let handle = app.handle();
            let state = handle.state::<Arc<AppState>>();

            // Start API server if enabled
            let settings = state.settings.blocking_read();
            if settings.enable_api {
                let port = settings.port;
                drop(settings);

                let state_clone = state.inner().clone();
                let api_handle = tauri::async_runtime::spawn(async move {
                    run_api_server(state_clone, port).await;
                });

                *state.api_server.lock() = Some(api_handle);
            }

            // System stats updater (includes thermal monitoring)
            let state_clone = state.inner().clone();
            let handle_clone = handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = interval(Duration::from_secs(2));
                loop {
                    interval.tick().await;
                    // Build stats without going through the Tauri State wrapper.
                    let (ram_used, ram_total, cpu_usage, cpu_temperature) = {
                        let mut sys = state_clone.system.lock();
                        sys.refresh_memory();
                        sys.refresh_cpu_usage();
                        let ram_used = sys.used_memory();
                        let ram_total = sys.total_memory();
                        let cpu_usage = sys.global_cpu_info().cpu_usage();
                        let cpu_temperature = get_cpu_temperature();
                        (ram_used, ram_total, cpu_usage, cpu_temperature)
                    };
                    let (model_loaded, model_name) = {
                        let model_state = state_clone.model_state.lock();
                        (
                            model_state.current_model.is_some(),
                            model_state.current_model.clone(),
                        )
                    };
                    let tokens_per_second = *state_clone.last_tokens_per_second.read().await;
                    let settings = state_clone.settings.read().await.clone();
                    let thermal_state =
                        determine_thermal_state(cpu_temperature.unwrap_or(0.0), cpu_usage);
                    let stats = SystemStats {
                        ram_used,
                        ram_total,
                        cpu_usage,
                        cpu_temperature,
                        model_loaded,
                        model_name,
                        tokens_per_second,
                        power_profile: settings.power_profile,
                        thermal_state,
                    };
                    let _ = handle_clone.emit("system:stats", stats);
                }
            });

            // Thermal monitoring & power management
            let state_clone = state.inner().clone();
            let handle_clone = handle.clone();
            let thermal_handle = tauri::async_runtime::spawn(async move {
                thermal_monitor_loop(state_clone, handle_clone).await;
            });
            *state.thermal_monitor.lock() = Some(thermal_handle);

            // Idle timer for auto-unload
            let state_clone = state.inner().clone();
            let idle_handle = tauri::async_runtime::spawn(async move {
                idle_monitor_loop(state_clone).await;
            });
            *state.idle_timer.lock() = Some(idle_handle);

            // Window event handler for unload on minimize
            let state_clone = state.inner().clone();
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(focused) = event {
                        let state = state_clone.clone();
                        let focused = *focused;
                        tauri::async_runtime::spawn(async move {
                            let settings = state.settings.read().await.clone();
                            if settings.unload_on_minimize && !focused {
                                let model_loaded = {
                                    let ms = state.model_state.lock();
                                    ms.current_model.is_some()
                                };
                                if model_loaded {
                                    info!("Auto-unloading model on window minimize");
                                    {
                                        let mut ms = state.model_state.lock();
                                        if let Some(mut process) = ms.llama_process.take() {
                                            let _ = process.start_kill();
                                        }
                                        ms.current_model = None;
                                        ms.model_path = None;
                                        ms.sender = None;
                                    }
                                    *state.last_tokens_per_second.write().await = None;
                                }
                            }
                        });
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_system_info,
            get_system_stats,
            check_internet,
            get_system_theme,
            list_downloaded_models,
            download_model,
            delete_model,
            load_model,
            unload_model,
            generate_completion,
            stop_generation,
            // Thermal management commands
            set_power_profile,
            get_thermal_state,
            set_thermal_throttle,
            set_auto_unload,
            set_unload_on_minimize,
            set_max_cpu_percent,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            error!("Tauri application error: {}", e);
            e
        })?;
    Ok(())
}

fn main() {
    run().expect("Application failed to start");
}
