use std::collections::HashMap;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;

pub struct DeploymentManager {
    targets: Vec<DeploymentTarget>,
    configurations: HashMap<String, DeploymentConfig>,
    build_cache: BuildCache,
}

#[derive(Debug, Clone)]
pub struct DeploymentTarget {
    pub name: String,
    pub platform: Platform,
    pub architecture: Architecture,
    pub runtime: Runtime,
    pub package_format: PackageFormat,
}

#[derive(Debug, Clone)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
    WebAssembly,
    Android,
    iOS,
    Docker,
    Cloud(CloudProvider),
}

#[derive(Debug, Clone)]
pub enum CloudProvider {
    AWS,
    Azure,
    GoogleCloud,
    Vercel,
    Netlify,
}

#[derive(Debug, Clone)]
pub enum Architecture {
    X86_64,
    ARM64,
    X86,
    ARM,
    WASM32,
}

#[derive(Debug, Clone)]
pub enum Runtime {
    Native,
    WASM,
    Container,
    Serverless,
    PWA,
}

#[derive(Debug, Clone)]
pub enum PackageFormat {
    Executable,
    Library,
    WebApp,
    Container,
    Mobile,
    CloudFunction,
}

#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    pub optimization_level: OptimizationLevel,
    pub include_debug_info: bool,
    pub compression: bool,
    pub minification: bool,
    pub tree_shaking: bool,
    pub bundle_dependencies: bool,
    pub environment_variables: HashMap<String, String>,
    pub build_flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Size,
    Speed,
    Ultra,
}

#[derive(Debug, Default)]
pub struct BuildCache {
    pub cached_builds: HashMap<String, CachedBuild>,
    pub dependency_graph: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CachedBuild {
    pub target: String,
    pub hash: String,
    pub timestamp: u64,
    pub size: usize,
    pub build_time: f64,
}

impl DeploymentManager {
    pub fn new() -> Self {
        let mut targets = Vec::new();
        
        // Pre-configured deployment targets
        targets.push(DeploymentTarget {
            name: "windows-x64".to_string(),
            platform: Platform::Windows,
            architecture: Architecture::X86_64,
            runtime: Runtime::Native,
            package_format: PackageFormat::Executable,
        });
        
        targets.push(DeploymentTarget {
            name: "linux-x64".to_string(),
            platform: Platform::Linux,
            architecture: Architecture::X86_64,
            runtime: Runtime::Native,
            package_format: PackageFormat::Executable,
        });
        
        targets.push(DeploymentTarget {
            name: "macos-arm64".to_string(),
            platform: Platform::MacOS,
            architecture: Architecture::ARM64,
            runtime: Runtime::Native,
            package_format: PackageFormat::Executable,
        });
        
        targets.push(DeploymentTarget {
            name: "web-wasm".to_string(),
            platform: Platform::WebAssembly,
            architecture: Architecture::WASM32,
            runtime: Runtime::WASM,
            package_format: PackageFormat::WebApp,
        });
        
        targets.push(DeploymentTarget {
            name: "docker-multi".to_string(),
            platform: Platform::Docker,
            architecture: Architecture::X86_64,
            runtime: Runtime::Container,
            package_format: PackageFormat::Container,
        });
        
        targets.push(DeploymentTarget {
            name: "aws-lambda".to_string(),
            platform: Platform::Cloud(CloudProvider::AWS),
            architecture: Architecture::X86_64,
            runtime: Runtime::Serverless,
            package_format: PackageFormat::CloudFunction,
        });
        
        let mut configurations = HashMap::new();
        
        // Default configurations for different environments
        configurations.insert("development".to_string(), DeploymentConfig {
            optimization_level: OptimizationLevel::Debug,
            include_debug_info: true,
            compression: false,
            minification: false,
            tree_shaking: false,
            bundle_dependencies: false,
            environment_variables: HashMap::new(),
            build_flags: vec!["--dev".to_string()],
        });
        
        configurations.insert("production".to_string(), DeploymentConfig {
            optimization_level: OptimizationLevel::Ultra,
            include_debug_info: false,
            compression: true,
            minification: true,
            tree_shaking: true,
            bundle_dependencies: true,
            environment_variables: HashMap::new(),
            build_flags: vec!["--release".to_string(), "--strip".to_string()],
        });
        
        Self {
            targets,
            configurations,
            build_cache: BuildCache::default(),
        }
    }
    
    pub fn deploy(&mut self, target_name: &str, config_name: &str, source_path: &Path) -> Result<DeploymentResult, String> {
        let target = self.find_target(target_name)?;
        let config = self.find_config(config_name)?;
        
        println!("Starting deployment to {} with {} configuration", target_name, config_name);
        
        // Check cache first
        if let Some(cached) = self.check_cache(target, source_path) {
            println!("Using cached build from {}", cached.timestamp);
            return Ok(DeploymentResult {
                target: target.name.clone(),
                success: true,
                build_time: 0.0,
                package_size: cached.size,
                output_path: self.get_output_path(target, source_path),
                warnings: Vec::new(),
            });
        }
        
        let start_time = std::time::Instant::now();
        
        // Platform-specific deployment
        let result = match &target.platform {
            Platform::Windows => self.deploy_windows(target, config, source_path)?,
            Platform::Linux => self.deploy_linux(target, config, source_path)?,
            Platform::MacOS => self.deploy_macos(target, config, source_path)?,
            Platform::WebAssembly => self.deploy_wasm(target, config, source_path)?,
            Platform::Docker => self.deploy_docker(target, config, source_path)?,
            Platform::Cloud(provider) => self.deploy_cloud(target, config, source_path, provider)?,
            _ => return Err("Unsupported platform".to_string()),
        };
        
        let build_time = start_time.elapsed().as_secs_f64();
        
        // Update cache
        self.update_cache(target, source_path, &result, build_time);
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time,
            package_size: result.package_size,
            output_path: result.output_path.clone(),
            warnings: result.warnings,
        })
    }
    
    fn deploy_windows(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Building for Windows x64...");
        
        let mut build_cmd = Command::new("cargo");
        build_cmd.arg("build");
        build_cmd.arg("--target").arg("x86_64-pc-windows-msvc");
        
        if matches!(config.optimization_level, OptimizationLevel::Release | OptimizationLevel::Ultra) {
            build_cmd.arg("--release");
        }
        
        let output = build_cmd
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Build failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        let exe_path = self.get_output_path(target, source_path);
        let package_size = if exe_path.exists() {
            fs::metadata(&exe_path).map(|m| m.len() as usize).unwrap_or(0)
        } else {
            0
        };
        
        // Additional Windows-specific steps
        if config.compression {
            self.compress_executable(&exe_path)?;
        }
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: 0.0,
            package_size,
            output_path: exe_path,
            warnings: vec!["Consider code signing for production distribution".to_string()],
        })
    }
    
    fn deploy_linux(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Building for Linux x64...");
        
        let mut build_cmd = Command::new("cargo");
        build_cmd.arg("build");
        build_cmd.arg("--target").arg("x86_64-unknown-linux-gnu");
        
        if matches!(config.optimization_level, OptimizationLevel::Release | OptimizationLevel::Ultra) {
            build_cmd.arg("--release");
        }
        
        let output = build_cmd
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Build failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        let exe_path = self.get_output_path(target, source_path);
        let package_size = if exe_path.exists() {
            fs::metadata(&exe_path).map(|m| m.len() as usize).unwrap_or(0)
        } else {
            0
        };
        
        // Strip debug symbols if requested
        if !config.include_debug_info {
            Command::new("strip")
                .arg(&exe_path)
                .output()
                .map_err(|e| format!("Failed to strip binary: {}", e))?;
        }
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: 0.0,
            package_size,
            output_path: exe_path,
            warnings: Vec::new(),
        })
    }
    
    fn deploy_macos(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Building for macOS ARM64...");
        
        let mut build_cmd = Command::new("cargo");
        build_cmd.arg("build");
        build_cmd.arg("--target").arg("aarch64-apple-darwin");
        
        if matches!(config.optimization_level, OptimizationLevel::Release | OptimizationLevel::Ultra) {
            build_cmd.arg("--release");
        }
        
        let output = build_cmd
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Build failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        let exe_path = self.get_output_path(target, source_path);
        let package_size = if exe_path.exists() {
            fs::metadata(&exe_path).map(|m| m.len() as usize).unwrap_or(0)
        } else {
            0
        };
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: 0.0,
            package_size,
            output_path: exe_path,
            warnings: vec!["Consider creating .app bundle for distribution".to_string()],
        })
    }
    
    fn deploy_wasm(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Building for WebAssembly...");
        
        // Install wasm-pack if not available
        let wasm_pack_check = Command::new("wasm-pack")
            .arg("--version")
            .output();
        
        if wasm_pack_check.is_err() {
            println!("Installing wasm-pack...");
            Command::new("cargo")
                .args(&["install", "wasm-pack"])
                .output()
                .map_err(|e| format!("Failed to install wasm-pack: {}", e))?;
        }
        
        let mut build_cmd = Command::new("wasm-pack");
        build_cmd.arg("build");
        build_cmd.arg("--target").arg("web");
        
        if matches!(config.optimization_level, OptimizationLevel::Release | OptimizationLevel::Ultra) {
            build_cmd.arg("--release");
        }
        
        let output = build_cmd
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to run wasm-pack: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("WASM build failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Generate HTML wrapper
        let html_content = self.generate_wasm_html(config)?;
        let html_path = source_path.join("pkg").join("index.html");
        fs::write(&html_path, html_content)
            .map_err(|e| format!("Failed to write HTML file: {}", e))?;
        
        let wasm_path = source_path.join("pkg");
        let package_size = self.calculate_directory_size(&wasm_path);
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: 0.0,
            package_size,
            output_path: wasm_path,
            warnings: vec!["Test in different browsers for compatibility".to_string()],
        })
    }
    
    fn deploy_docker(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Building Docker container...");
        
        // Generate Dockerfile
        let dockerfile_content = self.generate_dockerfile(config)?;
        let dockerfile_path = source_path.join("Dockerfile");
        fs::write(&dockerfile_path, dockerfile_content)
            .map_err(|e| format!("Failed to write Dockerfile: {}", e))?;
        
        // Build Docker image
        let image_name = format!("neksis-app:{}", chrono::Utc::now().timestamp());
        let mut build_cmd = Command::new("docker");
        build_cmd.arg("build");
        build_cmd.arg("-t").arg(&image_name);
        build_cmd.arg(".");
        
        let output = build_cmd
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to run docker build: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Docker build failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Get image size
        let size_cmd = Command::new("docker")
            .args(&["images", &image_name, "--format", "{{.Size}}"])
            .output()
            .map_err(|e| format!("Failed to get image size: {}", e))?;
        
        let size_str = String::from_utf8_lossy(&size_cmd.stdout);
        let package_size = self.parse_docker_size(&size_str);
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: 0.0,
            package_size,
            output_path: PathBuf::from(image_name),
            warnings: vec!["Consider multi-stage build for smaller image size".to_string()],
        })
    }
    
    fn deploy_cloud(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path, provider: &CloudProvider) -> Result<DeploymentResult, String> {
        match provider {
            CloudProvider::AWS => self.deploy_aws_lambda(target, config, source_path),
            CloudProvider::Azure => self.deploy_azure_functions(target, config, source_path),
            CloudProvider::Vercel => self.deploy_vercel(target, config, source_path),
            _ => Err("Cloud provider not yet supported".to_string()),
        }
    }
    
    fn deploy_aws_lambda(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Preparing AWS Lambda deployment...");
        
        // Create Lambda-compatible build
        let mut build_cmd = Command::new("cargo");
        build_cmd.arg("build");
        build_cmd.arg("--target").arg("x86_64-unknown-linux-gnu");
        build_cmd.arg("--release");
        
        let output = build_cmd
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to build for Lambda: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Lambda build failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Create deployment package
        let zip_path = source_path.join("lambda-deployment.zip");
        self.create_lambda_package(source_path, &zip_path)?;
        
        let package_size = fs::metadata(&zip_path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: 0.0,
            package_size,
            output_path: zip_path,
            warnings: vec![
                "Upload to AWS Lambda manually or use AWS CLI".to_string(),
                "Configure Lambda runtime and environment variables".to_string(),
            ],
        })
    }
    
    fn deploy_azure_functions(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Preparing Azure Functions deployment...");
        
        // Generate Azure Functions configuration
        let func_json = r#"{
            "scriptFile": "target/release/neksis",
            "entryPoint": "main",
            "bindings": [
                {
                    "authLevel": "function",
                    "type": "httpTrigger",
                    "direction": "in",
                    "name": "req"
                },
                {
                    "type": "http",
                    "direction": "out",
                    "name": "$return"
                }
            ]
        }"#;
        
        let func_dir = source_path.join("HttpTrigger");
        fs::create_dir_all(&func_dir)
            .map_err(|e| format!("Failed to create function directory: {}", e))?;
        
        fs::write(func_dir.join("function.json"), func_json)
            .map_err(|e| format!("Failed to write function.json: {}", e))?;
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: 0.0,
            package_size: 0,
            output_path: func_dir,
            warnings: vec!["Deploy using Azure Functions Core Tools".to_string()],
        })
    }
    
    fn deploy_vercel(&self, target: &DeploymentTarget, config: &DeploymentConfig, source_path: &Path) -> Result<DeploymentResult, String> {
        println!("Preparing Vercel deployment...");
        
        // First build as WASM
        let wasm_result = self.deploy_wasm(target, config, source_path)?;
        
        // Generate vercel.json
        let vercel_config = r#"{
            "version": 2,
            "builds": [
                {
                    "src": "pkg/**",
                    "use": "@vercel/static"
                }
            ],
            "routes": [
                {
                    "src": "/(.*)",
                    "dest": "/pkg/$1"
                }
            ]
        }"#;
        
        fs::write(source_path.join("vercel.json"), vercel_config)
            .map_err(|e| format!("Failed to write vercel.json: {}", e))?;
        
        Ok(DeploymentResult {
            target: target.name.clone(),
            success: true,
            build_time: wasm_result.build_time,
            package_size: wasm_result.package_size,
            output_path: wasm_result.output_path,
            warnings: vec!["Deploy using 'vercel deploy' command".to_string()],
        })
    }
    
    // Helper methods
    fn find_target(&self, name: &str) -> Result<&DeploymentTarget, String> {
        self.targets.iter()
            .find(|t| t.name == name)
            .ok_or_else(|| format!("Target '{}' not found", name))
    }
    
    fn find_config(&self, name: &str) -> Result<&DeploymentConfig, String> {
        self.configurations.get(name)
            .ok_or_else(|| format!("Configuration '{}' not found", name))
    }
    
    fn get_output_path(&self, target: &DeploymentTarget, source_path: &Path) -> PathBuf {
        let target_dir = match &target.platform {
            Platform::Windows => "target/x86_64-pc-windows-msvc/release",
            Platform::Linux => "target/x86_64-unknown-linux-gnu/release",
            Platform::MacOS => "target/aarch64-apple-darwin/release",
            Platform::WebAssembly => "pkg",
            _ => "target/release",
        };
        
        let binary_name = match &target.platform {
            Platform::Windows => "neksis.exe",
            Platform::WebAssembly => "neksis_bg.wasm",
            _ => "neksis",
        };
        
        source_path.join(target_dir).join(binary_name)
    }
    
    fn check_cache(&self, target: &DeploymentTarget, source_path: &Path) -> Option<&CachedBuild> {
        // Simple cache implementation
        self.build_cache.cached_builds.get(&target.name)
    }
    
    fn update_cache(&mut self, target: &DeploymentTarget, source_path: &Path, result: &DeploymentResult, build_time: f64) {
        let cached_build = CachedBuild {
            target: target.name.clone(),
            hash: self.calculate_source_hash(source_path),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            size: result.package_size,
            build_time,
        };
        
        self.build_cache.cached_builds.insert(target.name.clone(), cached_build);
    }
    
    fn calculate_source_hash(&self, _source_path: &Path) -> String {
        // Simplified hash calculation
        format!("{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs())
    }
    
    fn compress_executable(&self, _exe_path: &Path) -> Result<(), String> {
        // Placeholder for executable compression
        println!("Compressing executable...");
        Ok(())
    }
    
    fn generate_wasm_html(&self, config: &DeploymentConfig) -> Result<String, String> {
        let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Neksis WASM App</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        #output {{ background: #f0f0f0; padding: 20px; margin: 20px 0; border-radius: 5px; }}
        button {{ padding: 10px 20px; font-size: 16px; }}
    </style>
</head>
<body>
    <h1>Neksis WebAssembly Application</h1>
    <button onclick="runNeksis()">Run Neksis Code</button>
    <div id="output"></div>
    
    <script type="module">
        import init, {{ run_neksis }} from './neksis.js';
        
        async function run() {{
            await init();
            window.runNeksis = function() {{
                const output = document.getElementById('output');
                try {{
                    const result = run_neksis();
                    output.innerHTML = '<pre>' + result + '</pre>';
                }} catch (e) {{
                    output.innerHTML = '<pre style="color: red;">Error: ' + e + '</pre>';
                }}
            }};
        }}
        
        run();
    </script>
</body>
</html>"#);
        
        Ok(html)
    }
    
    fn generate_dockerfile(&self, config: &DeploymentConfig) -> Result<String, String> {
        let dockerfile = format!(r#"FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/neksis /usr/local/bin/neksis

EXPOSE 8080
CMD ["neksis"]
"#);
        
        Ok(dockerfile)
    }
    
    fn calculate_directory_size(&self, dir: &Path) -> usize {
        let mut size = 0;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    size += metadata.len() as usize;
                }
            }
        }
        size
    }
    
    fn parse_docker_size(&self, size_str: &str) -> usize {
        // Parse Docker size format (e.g., "45.2MB")
        let cleaned = size_str.trim().to_uppercase();
        if let Some(mb_pos) = cleaned.find("MB") {
            if let Ok(size) = cleaned[..mb_pos].parse::<f64>() {
                return (size * 1_000_000.0) as usize;
            }
        }
        0
    }
    
    fn create_lambda_package(&self, source_path: &Path, zip_path: &Path) -> Result<(), String> {
        // Create ZIP package for Lambda
        Command::new("zip")
            .args(&["-r", zip_path.to_str().unwrap(), "target/release/neksis", "bootstrap"])
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to create Lambda package: {}", e))?;
        
        Ok(())
    }
    
    pub fn list_targets(&self) -> Vec<&DeploymentTarget> {
        self.targets.iter().collect()
    }
    
    pub fn get_deployment_status(&self) -> String {
        let mut status = String::new();
        
        status.push_str("=== Deployment Status ===\n");
        status.push_str(&format!("Available targets: {}\n", self.targets.len()));
        status.push_str(&format!("Cached builds: {}\n", self.build_cache.cached_builds.len()));
        
        status.push_str("\nTargets:\n");
        for target in &self.targets {
            status.push_str(&format!("  {} - {:?} on {:?}\n", 
                target.name, target.runtime, target.platform));
        }
        
        status
    }
}

#[derive(Debug)]
pub struct DeploymentResult {
    pub target: String,
    pub success: bool,
    pub build_time: f64,
    pub package_size: usize,
    pub output_path: PathBuf,
    pub warnings: Vec<String>,
}
