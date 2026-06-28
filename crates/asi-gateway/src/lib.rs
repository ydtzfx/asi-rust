//! ASI Gateway — plugin registry + API gateway configuration.
pub mod plugin_registry;

/// Third-party plugin manifest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tools: Vec<ToolSpec>,
    pub hooks: Vec<HookSpec>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolSpec { pub name: String, pub description: String, pub parameters: serde_json::Value }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookSpec { pub event: String, pub endpoint: String }

/// Plugin registry — manages installed plugins.
pub struct PluginRegistry {
    plugins: std::sync::Mutex<Vec<PluginManifest>>,
}
impl PluginRegistry {
    pub fn new() -> Self { Self { plugins: std::sync::Mutex::new(Vec::new()) } }
    pub fn register(&self, plugin: PluginManifest) { self.plugins.lock().unwrap().push(plugin); }
    pub fn list(&self) -> Vec<PluginManifest> { self.plugins.lock().unwrap().clone() }
    pub fn find_tool(&self, name: &str) -> Option<ToolSpec> {
        self.plugins.lock().unwrap().iter().flat_map(|p| &p.tools).find(|t| t.name == name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_register_and_list() {
        let reg = PluginRegistry::new();
        reg.register(PluginManifest { name:"test".into(), version:"1.0".into(), description:"".into(), author:"".into(), tools:vec![], hooks:vec![] });
        assert_eq!(reg.list().len(), 1);
    }
}
