use std::path::PathBuf;
fn main() {
    let root = PathBuf::from("Sample Projects/Neon & Nightmares");
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join("project.json")).unwrap()).unwrap();
    if let Some(pipelines) = manifest.get("pipelines") {
        for pipeline in pipelines.as_array().unwrap() {
            let id = pipeline.get("pipeline_id").unwrap().as_str().unwrap();
            std::fs::write(root.join("pipelines").join(format!("{}.json", id)), serde_json::to_string_pretty(pipeline).unwrap()).unwrap();
        }
    }
}
