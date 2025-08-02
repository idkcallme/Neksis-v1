use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::CompilerError;

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
    pub entry_point: Option<String>,
}

#[derive(Debug)]
pub struct PackageManager {
    registry_url: String,
    packages_dir: PathBuf,
}

impl PackageManager {
    pub fn new() -> Result<Self, CompilerError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| CompilerError::runtime_error("Could not determine home directory"))?;
        
        let packages_dir = home_dir.join(".nexus").join("packages");
        
        fs::create_dir_all(&packages_dir)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to create packages directory: {}", e)))?;
        
        Ok(Self {
            registry_url: "https://registry.nexus-lang.org".to_string(),
            packages_dir,
        })
    }

    pub fn init_project(&self, project_name: &str) -> Result<(), CompilerError> {
        // Create project directory
        fs::create_dir_all(project_name)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to create project directory: {}", e)))?;
        
        // Change to project directory
        let project_path = Path::new(project_name);
        
        let manifest = PackageManifest {
            name: project_name.to_string(),
            version: "0.1.0".to_string(),
            description: Some("A neksis project".to_string()),
            author: None,
            license: Some("MIT".to_string()),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            entry_point: Some("src/main.nx".to_string()),
        };
        
        let manifest_content = serde_json::to_string_pretty(&manifest)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to serialize manifest: {}", e)))?;
        
        fs::write(project_path.join("nexus.json"), manifest_content)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to write manifest: {}", e)))?;
        
        // Create basic project structure
        fs::create_dir_all(project_path.join("src"))
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to create src directory: {}", e)))?;
        
        let main_content = r#"fn main() {
    println("Hello, neksis!");
}
"#;
        
        fs::write(project_path.join("src/main.nx"), main_content)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to create main.nx: {}", e)))?;
        
        // Create README.md
        let readme_content = format!("# {}\n\nA neksis project.\n\n## Getting Started\n\n```bash\nneksis run\n```\n", project_name);
        fs::write(project_path.join("README.md"), readme_content)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to create README.md: {}", e)))?;
        
        Ok(())
    }

    pub fn install_dependency(&self, package_name: &str, version: Option<&str>) -> Result<(), CompilerError> {
        let version = version.unwrap_or("latest");
        
        // Download package from registry
        let package_url = format!("{}/packages/{}/{}", self.registry_url, package_name, version);
        
        // TODO: Implement actual HTTP download
        println!("Would download package {} version {} from {}", package_name, version, package_url);
        
        // Extract and install
        let package_dir = self.packages_dir.join(package_name).join(version);
        fs::create_dir_all(&package_dir)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to create package directory: {}", e)))?;
        
        // Update manifest
        self.update_manifest_dependency(package_name, version)?;
        
        Ok(())
    }

    pub fn update_manifest_dependency(&self, package_name: &str, version: &str) -> Result<(), CompilerError> {
        let manifest_path = Path::new("nexus.json");
        
        if !manifest_path.exists() {
            return Err(CompilerError::runtime_error("No nexus.json found. Run 'nexus init' first."));
        }
        
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read manifest: {}", e)))?;
        
        let mut manifest: PackageManifest = serde_json::from_str(&content)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to parse manifest: {}", e)))?;
        
        manifest.dependencies.insert(package_name.to_string(), version.to_string());
        
        let updated_content = serde_json::to_string_pretty(&manifest)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to serialize manifest: {}", e)))?;
        
        fs::write(manifest_path, updated_content)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to write manifest: {}", e)))?;
        
        Ok(())
    }

    pub fn build_project(&self) -> Result<(), CompilerError> {
        let manifest_path = Path::new("nexus.json");
        
        if !manifest_path.exists() {
            return Err(CompilerError::runtime_error("No nexus.json found. Run 'nexus init' first."));
        }
        
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read manifest: {}", e)))?;
        
        let manifest: PackageManifest = serde_json::from_str(&content)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to parse manifest: {}", e)))?;
        
        let entry_point = manifest.entry_point
            .ok_or_else(|| CompilerError::runtime_error("No entry point specified in manifest"))?;
        
        // Compile the project
        let compiler = crate::compiler::FastCompiler::new(crate::compiler::CompilerOptions::default());
        compiler.compile_file(&entry_point)?;
        
        println!("Build completed successfully!");
        Ok(())
    }

    pub fn run_project(&self) -> Result<(), CompilerError> {
        let manifest_path = Path::new("nexus.json");
        
        if !manifest_path.exists() {
            return Err(CompilerError::runtime_error("No nexus.json found. Run 'nexus init' first."));
        }
        
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read manifest: {}", e)))?;
        
        let manifest: PackageManifest = serde_json::from_str(&content)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to parse manifest: {}", e)))?;
        
        let entry_point = manifest.entry_point
            .ok_or_else(|| CompilerError::runtime_error("No entry point specified in manifest"))?;
        
        // Run the project
        let compiler = crate::compiler::FastCompiler::new(crate::compiler::CompilerOptions::default());
        let result = compiler.compile_file(&entry_point)?;
        println!("{}", result);
        
        Ok(())
    }
} 